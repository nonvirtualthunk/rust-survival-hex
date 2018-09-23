use common::prelude::*;
use game::prelude::*;
use game::core::Reduceable;
use scenario::Scenario;
use samvival_core::create_world;
use terrain;
use common::Color;
use archetypes::*;
use entities::*;
use game::EntityBuilder;
use logic;
use prelude::GameEvent;
use entities::reactions::ReactionTypeRef;
use game::DebugData;
use archetypes::weapons::create_weapon_archetypes;


#[derive(Clone)]
pub struct FirstEverScenario {}
impl Scenario for FirstEverScenario {
    fn initialize_scenario_world(&self) -> World {
        let mut raw_world = create_world();
        {
            let world = &mut raw_world;
            ::samvival_core::initialize_world(world);

            create_weapon_archetypes(world);
            let item_catalog = Catalog::of::<ItemArchetype>(world.view(), Entity::sentinel());

            for tile in terrain::generator::generate(world, 70) {
                let tile = tile.with(DebugData { name: strf("world tile") }).create(world);
                let pos = world.data::<TileData>(tile).position;
                world.index_entity(tile, pos);
            }

            let player_faction = EntityBuilder::new()
                .with(FactionData {
                    name: String::from("Player"),
                    color: Color::new(1.1, 0.3, 0.3, 1.0),
                    player_faction: true,
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
                    player_faction: false,
                })
                .with(DebugData { name: strf("enemy faction") })
                .create(world);

//            let weapon_archetypes = weapon_archetypes();

            let character_archetypes = character_archetypes();

            let bow = logic::crafting::craft_without_materials(world, item_catalog.entity_with_name("longbow"));


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

            logic::item::put_item_in_inventory(world, bow, archer);
            logic::item::equip_item(world, bow, archer, true);

            let resources = world.view().world_data::<Resources>();
            for i in 0..5 {
                let iron = world.clone_entity(resources.main.iron);
                world.attach_data::<EntityMetadata>(iron, EntityMetadata {archetype : resources.main.iron});
                world.add_event(GameEvent::EntityCreated { entity : iron });
                logic::item::put_item_in_inventory(world, iron, archer);
            }


            world.modify_with_desc(archer, CombatData::ranged_accuracy_bonus.add(1), "well rested");
            world.modify_with_desc(archer, CombatData::ranged_accuracy_bonus.add(3), "careful aim");

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

            let spear = logic::crafting::craft_without_materials(world, item_catalog.entity_with_name("longspear"));
            logic::item::put_item_in_inventory(world, spear, spearman);
            logic::item::equip_item(world, spear, spearman, true);
            logic::movement::place_entity_in_world(world, spearman, AxialCoord::new(1, -1));

            let special_attack = EntityBuilder::new()
                .with(DerivedAttackData {
                    character_condition: EntitySelector::Any,
                    weapon_condition: EntitySelector::is_a(&taxonomy::weapons::ReachWeapon),
                    attack_condition: EntitySelector::is_a(&taxonomy::attacks::StabbingAttack).and(EntitySelector::is_a(&taxonomy::attacks::ReachAttack)),
                    kind: DerivedAttackKind::PiercingStrike,
                }).create(world);

            world.modify_with_desc(spearman, CombatData::special_attacks.append(special_attack), None);


            let peasant = char_base("axflar")
                .with(CharacterData {
                    sprite: String::from("human/peasant"),
                    name: String::from("Peasant"),
                    health: Reduceable::new(45),
                    action_points: Reduceable::new(8),
                    ..Default::default()
                })
                .with(AllegianceData { faction: player_faction })
                .with(ActionData {
                    active_reaction: ReactionTypeRef::Defend,
                    ..Default::default()
                })
                .with(DebugData { name: strf("peasant") })
                .create(world);

            let hatchet = logic::crafting::craft_without_materials(world, item_catalog.entity_with_name("hatchet"));
            logic::item::put_item_in_inventory(world, hatchet, peasant);
            logic::item::equip_item(world, hatchet, peasant, true);

            let pickaxe = logic::crafting::craft_without_materials(world, item_catalog.entity_with_name("pickaxe"));
            logic::item::put_item_in_inventory(world, pickaxe, peasant);
            logic::item::equip_item(world, pickaxe, peasant, true);

            logic::movement::place_entity_in_world(world, peasant, AxialCoord::new(1, -2));


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
                .with_creator(|world|IdentityData::of_kind(Taxon::new(world, "summoning stone", &taxonomy::Monster)))
                .create(world);
            logic::movement::place_entity_in_world(world, spawner, AxialCoord::new(10, 0));

            world.add_event(GameEvent::WorldStart);
        }

        raw_world
    }
}