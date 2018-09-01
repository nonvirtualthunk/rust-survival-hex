use common::prelude::*;
use game::World;
use graphics::core::GraphicsWrapper;
use piston_window::*;
use game::entities::TileData;
use common::hex::*;
use gfx_device_gl;
use tactical::TacticalMode;
use gui::GUI;
use gui::Wid;
use gui::Widget;
use gui::WidgetType;
use gui::Sizing;
use gui::UIEvent;
use common::Color;
use game::entities::reactions::reaction_types;
use game::DebugData;

use game::prelude::*;
use game::entities::combat::DamageType;
use game::entities::*;
use game::logic;
use game::reflect::*;
use game::GameEvent;
use cgmath::InnerSpace;
use game::archetypes::*;
use game::entities::taxonomy;
use game::terrain;
use game::entities::reactions::ReactionTypeRef;

//use graphics::core::Context as ArxContext;
use graphics::core::GraphicsResources;
use std::collections::HashMap;


//pub static mut GLOBAL_MODIFIERS : Modifiers = Modifiers {
//    alt : false,
//    ctrl : false,
//    shift : false
//};
//
////pub static mut MOUSE_POSITION : Vec2f = Vec2f {
////    x : 0.0,
////    y : 0.0
////};
//
//pub fn get_key_modifiers() -> Modifiers {
//    unsafe {
//        GLOBAL_MODIFIERS.clone()
//    }
//}
//pub fn set_key_modifiers(modifiers : Modifiers) {
//    unsafe {
//        GLOBAL_MODIFIERS = modifiers;
//    }
//}

pub trait GameMode {
    fn enter(&mut self, world: &mut World);
    fn update(&mut self, world: &mut World, dt: f64);
    fn update_gui(&mut self, world: &mut World, ui: &mut GUI, frame_id: Option<Wid>);
    fn draw(&mut self, world: &mut World, g: &mut GraphicsWrapper);
    fn on_event<'a, 'b>(&'a mut self, world: &mut World, ui: &'b mut GUI, event: &UIEvent);
    fn handle_event(&mut self, world: &mut World, gui: &mut GUI, event: &UIEvent);
}

pub struct Game {
    pub world: World,
    pub resources: GraphicsResources,
    pub active_mode: Box<GameMode>,
    pub viewport: Viewport,
    pub gui: GUI,
}

impl Game {
    pub fn new(factory: gfx_device_gl::Factory) -> Game {
        let mut gui = GUI::new();

        let (world, player_faction) = Game::init_world();
        let tactical_mode = TacticalMode::new(&mut gui, &world, player_faction);

        Game {
            world,
            resources: GraphicsResources::new(factory, "survival"),
            active_mode: Box::new(tactical_mode),
            gui,
            viewport: Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256],
            },
        }
    }


    pub fn init_world() -> (World, Entity) {
        let mut raw_world = create_world();
        let world = &mut raw_world;

        for tile in terrain::generator::generate(70) {
            let tile = tile.with(DebugData { name: strf("world tile") }).create(world);
            let pos = world.data::<TileData>(tile).position;
            world.index_entity(tile, pos);
        }

        let player_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Player"),
                color: Color::new(1.1, 0.3, 0.3, 1.0),
            })
            .with(DebugData { name: strf("player faction") })
            .create(world);

        world.attach_world_data(TurnData {
            turn_number: 0,
            active_faction: player_faction,
        });


        let enemy_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Enemy"),
                color: Color::new(0.3, 0.3, 0.9, 1.0),

            })
            .with(DebugData { name: strf("enemy faction") })
            .create(world);


        let weapon_archetypes = weapon_archetypes();

        let character_archetypes = character_archetypes();

        let bow = weapon_archetypes.with_name("longbow").create(world);


        let char_base = |name: Str| character_archetypes.with_name("human").clone()
            .with(IdentityData::new(name, &taxonomy::Person));

        let archer = char_base("gunnar")
            .with(CharacterData {
                sprite: String::from("elf/archer"),
                name: String::from("Archer"),
                health: Reduceable::new(25),
                action_points: Reduceable::new(8),
                ..Default::default()
            })
            .with(AllegianceData { faction: player_faction })
            .with(ActionData {
                active_reaction: ReactionTypeRef::Dodge,
                ..Default::default()
            })
            .with(DebugData { name: strf("archer") })
            .create(world);

        logic::item::equip_item(world, bow, archer, true);

        world.modify(archer, CombatData::ranged_accuracy_bonus.add(1), "well rested");
        world.modify(archer, CombatData::ranged_accuracy_bonus.add(3), "careful aim");

        logic::movement::place_entity_in_world(world, archer, AxialCoord::new(0, 0));


        let spearman = char_base("haftdar")
            .with(CharacterData {
                sprite: String::from("human/spearman"),
                name: String::from("Spearman"),
                health: Reduceable::new(45),
                action_points: Reduceable::new(8),
                ..Default::default()
            })
            .with(AllegianceData { faction: player_faction })
            .with(ActionData {
                active_reaction: ReactionTypeRef::Counterattack,
                ..Default::default()
            })
            .with(DebugData { name: strf("spearman") })
            .create(world);

        let spear = weapon_archetypes.with_name("longspear").create(world);
        logic::item::equip_item(world, spear, spearman, true);
        logic::movement::place_entity_in_world(world, spearman, AxialCoord::new(1, -1));

        let special_attack = EntityBuilder::new()
            .with(DerivedAttackData {
                character_condition: EntitySelectors::Any,
                weapon_condition: EntitySelectors::is_a(&taxonomy::weapons::ReachWeapon),
                attack_condition: EntitySelectors::is_a(&taxonomy::attacks::StabbingAttack).and(EntitySelectors::is_a(&taxonomy::attacks::ReachAttack)),
                kind: DerivedAttackKind::PiercingStrike,
            }).create(world);

        world.modify(spearman, CombatData::special_attacks.append(special_attack), None);


        let monster_base = character_archetypes.with_name("mud monster").clone()
            .with(AllegianceData { faction: enemy_faction })
            .with(DebugData { name: strf("monster") });

        let create_monster_at = |world_in: &mut World, pos: AxialCoord| {
            let monster = monster_base.clone().create(world_in);

            logic::movement::place_entity_in_world(world_in, monster, pos);

            monster
        };

        let monster1 = create_monster_at(world, AxialCoord::new(4, 0));
        let monster2 = create_monster_at(world, AxialCoord::new(0, 4));

        let spawner = EntityBuilder::new()
            .with(CharacterData {
                sprite: strf("void/summoner_monolith"),
                name: strf("Summoning Stone"),
                action_points: Reduceable::new(1),
                health: Reduceable::new(100),
                ..Default::default()
            })
            .with(MovementData { move_speed: Sext::of(0), ..Default::default() })
            .with(AllegianceData { faction: enemy_faction })
            .with(PositionData::default())
            .with(CombatData { dodge_bonus: -10, ..Default::default() })
            .with(MonsterSpawnerData {
                spawns: vec![
                    Spawn {
                        entity: SpawnEntity::Character(strf("mud monster")),
                        start_spawn_turn: 1,
                        turns_between_spawns: 4,
                    }]
            })
            .with(IdentityData::of_kind(taxon("summoning stone", &taxonomy::Monster)))
            .create(world);
        logic::movement::place_entity_in_world(world, spawner, AxialCoord::new(10, 0));

        world.add_event(GameEvent::WorldStart);

        (raw_world, player_faction)
    }

    pub fn on_load(&mut self, _: &mut PistonWindow) {}
    pub fn on_update(&mut self, upd: UpdateArgs) {
        self.active_mode.update(&mut self.world, upd.dt);

        self.active_mode.update_gui(&mut self.world, &mut self.gui, None);

        self.gui.reset_events();
    }

    pub fn on_draw<'a>(&'a mut self, c: Context, g: &'a mut G2d) {
        if let Some(v) = c.viewport {
            self.viewport = v;
        }

        c.reset();

        clear([0.8, 0.8, 0.8, 1.0], g);

        let mut wrapper = GraphicsWrapper::new(c, &mut self.resources, g);

        self.active_mode.draw(&mut self.world, &mut wrapper);

        self.gui.draw(&mut wrapper);
        //        self.player.render(g, center);
    }

    pub fn on_event(&mut self, event: &Event) {
        if let Some(ui_event) = self.gui.convert_event(event.clone()) {
            if !self.gui.handle_ui_event_for_self(&ui_event) {
                self.active_mode.handle_event(&mut self.world, &mut self.gui, &ui_event);
            }
            self.active_mode.on_event(&mut self.world, &mut self.gui, &ui_event);
        }
    }
}


pub fn normalize_screen_pos(screen_pos: Vec2f, viewport: &Viewport) -> Vec2f {
    let in_x = screen_pos.x;
    let in_y = viewport.window_size[1] as f32 - screen_pos.y - 1.0;

    let centered_x = in_x - (viewport.window_size[0] / 2) as f32;
    let centered_y = in_y - (viewport.window_size[1] / 2) as f32;

    let norm_x = centered_x / viewport.window_size[0] as f32;
    let norm_y = centered_y / viewport.window_size[1] as f32;

    let scale_factor = viewport.draw_size[0] as f32 / viewport.window_size[0] as f32;

    let scaled_x = norm_x * scale_factor;
    let scaled_y = norm_y * scale_factor;

    v2(scaled_x, scaled_y)
}