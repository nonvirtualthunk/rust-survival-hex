use common::prelude::*;
use game::World;
use graphics::core::GraphicsWrapper;
use conrod;
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


use game::prelude::*;
use game::entities::combat::DamageType;
use game::entities::*;
use game::logic;
use game::reflect::*;
use game::GameEvent;
use cgmath::InnerSpace;

//use graphics::core::Context as ArxContext;
use graphics::core::GraphicsResources;


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
        let mut raw_world = World::new();
        let world = &mut raw_world;
        // -------- entity data --------------
        world.register::<TileData>();
        world.register::<CharacterData>();
        world.register::<CombatData>();
        world.register::<InventoryData>();
        world.register::<SkillData>();
        world.register::<ItemData>();
        world.register::<FactionData>();
        world.register::<PositionData>();
        world.register::<GraphicsData>();
        // -------- world data ---------------
        world.register::<MapData>();
        world.register::<TurnData>();

        world.register_index::<AxialCoord>();

        world.register_event_type::<GameEvent>();

        world.attach_world_data(&MapData {
            min_tile_bound: AxialCoord::new(-30, -30),
            max_tile_bound: AxialCoord::new(30, 30),
        });

        for x in -50..50 {
            for y in -50..50 {
                let coord = AxialCoord::new(x, y);
                if coord.as_cart_vec().magnitude2() < 30.0 * 30.0 {
                    let tile = EntityBuilder::new()
                        .with(TileData {
                            position: coord,
                            name: "grass",
                            move_cost: Sext::of(1),
                            cover: 0,
                            occupied_by: None,
                            elevation: 0,
                        }).create(world);
                    world.index_entity(tile, coord);
                }
            }
        }

        let player_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Player"),
                color: Color::new(1.1, 0.3, 0.3, 1.0),
            }).create(world);

        world.attach_world_data(&TurnData {
            turn_number: 0,
            active_faction: player_faction,
        });


        let enemy_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Enemy"),
                color: Color::new(0.3, 0.3, 0.9, 1.0),

            }).create(world);


        let bow = EntityBuilder::new()
            .with(ItemData {
                primary_attack: Some(Attack {
                    name: "bowshot",
                    ap_cost: 4,
                    damage_dice: DicePool {
                        die: 8,
                        count: 2,
                    },
                    damage_bonus: 1,
                    to_hit_bonus: 3,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 10,
                    min_range: 2,
                }),
                ..Default::default()
            }).create(world);

        let archer = EntityBuilder::new()
            .with(CharacterData {
                faction: player_faction,
                sprite: String::from("elf/archer"),
                name: String::from("Archer"),
                move_speed: Sext::of_parts(1, 2), // one and 2 sixths
                health: Reduceable::new(25),
                ..Default::default()
            })
            .with(CombatData {
                ranged_accuracy_bonus : 2,
                ..Default::default()
            })
            .with(SkillData::default())
            .with(InventoryData::default())
            .with(PositionData::default())
            .with(GraphicsData::default())
            .create(world);
        let bow_attack_ref = AttackReference::of_attack(world.view(), archer, world.view().item(bow).primary_attack.as_ref().unwrap());
        world.modify(archer, CombatData::active_attack.set_to(bow_attack_ref), "equipped");

        world.modify(archer, CombatData::ranged_accuracy_bonus.add(1), "well rested");
        world.modify(archer, CombatData::ranged_accuracy_bonus.add(3), "careful aim");

        logic::movement::place_entity_in_world(world, archer, AxialCoord::new(0, 0));

        logic::items::equip_item(world, archer, bow);

        let create_monster_at = |world_in: &mut World, pos: AxialCoord| {
            let monster = EntityBuilder::new()
                .with(CharacterData {
                    faction: enemy_faction,
                    sprite: String::from("void/monster"),
                    name: String::from("Monster"),
                    move_speed: Sext::of_rounded(0.75),
                    action_points: Reduceable::new(6),
                    health: Reduceable::new(22),
                    ..Default::default()
                })
                .with(PositionData {
                    hex : pos
                })
                .with(CombatData {
                    natural_attacks: vec![Attack {
                        name: "slam",
                        damage_dice: DicePool {
                            count: 1,
                            die: 4,
                        },
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .with(SkillData::default())
                .with(InventoryData::default())
                .with(GraphicsData::default())
                .create(world_in);

            logic::movement::place_entity_in_world(world_in, monster, pos);

            monster
        };

        let monster1 = create_monster_at(world, AxialCoord::new(4, 0));
        let monster2 = create_monster_at(world, AxialCoord::new(0, 4));

        world.modify(monster1, CombatData::dodge_bonus.add(1), "speed monster");

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