use entities::*;
use game::prelude::*;
use enum_map::EnumMap;
use common::reflect::*;
use common::hex::*;
use common::Color;
use common::prelude::*;
use entities::actions::*;
use entities::reactions::*;
use game::ModifierReference;
use std::collections::HashSet;
use std::collections::HashMap;


impl GraphicsData {
    pub const graphical_position: Field<GraphicsData, Option<CartVec>> = Field::new(stringify!( graphical_position ), |t| &t.graphical_position, |t| &mut t.graphical_position, |t, v| { t.graphical_position = v; });
    pub const color: Field<GraphicsData, Color> = Field::new(stringify!( color ), |t| &t.color, |t| &mut t.color, |t, v| { t.color = v; });
}

impl CharacterData {
    pub const faction: Field<CharacterData, Entity> = Field::new(stringify!( faction ), |t| &t.faction, |t| &mut t.faction, |t, v| { t.faction = v; });
    pub const health: Field<CharacterData, Reduceable<i32>> = Field::new(stringify!( health ), |t| &t.health, |t| &mut t.health, |t, v| { t.health = v; });
    pub const action_points: Field<CharacterData, Reduceable<i32>> = Field::new(stringify!( action_points ), |t| &t.action_points, |t| &mut t.action_points, |t, v| { t.action_points = v; });
    pub const move_speed: Field<CharacterData, Sext> = Field::new(stringify!( move_speed ), |t| &t.move_speed, |t| &mut t.move_speed, |t, v| { t.move_speed = v; });
    pub const moves: Field<CharacterData, Sext> = Field::new(stringify!( moves ), |t| &t.moves, |t| &mut t.moves, |t, v| { t.moves = v; });
    pub const stamina: Field<CharacterData, Reduceable<Sext>> = Field::new(stringify!( stamina ), |t| &t.stamina, |t| &mut t.stamina, |t, v| { t.stamina = v; });
    pub const stamina_recovery: Field<CharacterData, Sext> = Field::new(stringify!( stamina_recovery ), |t| &t.stamina_recovery, |t| &mut t.stamina_recovery, |t, v| { t.stamina_recovery = v; });
    pub const sprite: Field<CharacterData, String> = Field::new(stringify!( sprite ), |t| &t.sprite, |t| &mut t.sprite, |t, v| { t.sprite = v; });
    pub const name: Field<CharacterData, String> = Field::new(stringify!( name ), |t| &t.name, |t| &mut t.name, |t, v| { t.name = v; });
}

impl CombatData {
    pub const active_attack: Field<CombatData, AttackReference> = Field::new(stringify!( active_attack ), |t| &t.active_attack, |t| &mut t.active_attack, |t, v| { t.active_attack = v; });
    pub const natural_attacks: Field<CombatData, Vec<Attack>> = Field::new(stringify!( natural_attacks ), |t| &t.natural_attacks, |t| &mut t.natural_attacks, |t, v| { t.natural_attacks = v; });
    pub const counters_remaining: Field<CombatData, Reduceable<i32>> = Field::new(stringify!( counters_remaining ), |t| &t.counters_remaining, |t| &mut t.counters_remaining, |t, v| { t.counters_remaining = v; });
    pub const counters_per_event: Field<CombatData, i32> = Field::new(stringify!( counters_per_event ), |t| &t.counters_per_event, |t| &mut t.counters_per_event, |t, v| { t.counters_per_event = v; });
    pub const melee_accuracy_bonus: Field<CombatData, i32> = Field::new(stringify!( melee_accuracy_bonus ), |t| &t.melee_accuracy_bonus, |t| &mut t.melee_accuracy_bonus, |t, v| { t.melee_accuracy_bonus = v; });
    pub const ranged_accuracy_bonus: Field<CombatData, i32> = Field::new(stringify!( ranged_accuracy_bonus ), |t| &t.ranged_accuracy_bonus, |t| &mut t.ranged_accuracy_bonus, |t, v| { t.ranged_accuracy_bonus = v; });
    pub const melee_damage_bonus: Field<CombatData, i32> = Field::new(stringify!( melee_damage_bonus ), |t| &t.melee_damage_bonus, |t| &mut t.melee_damage_bonus, |t, v| { t.melee_damage_bonus = v; });
    pub const ranged_damage_bonus: Field<CombatData, i32> = Field::new(stringify!( ranged_damage_bonus ), |t| &t.ranged_damage_bonus, |t| &mut t.ranged_damage_bonus, |t, v| { t.ranged_damage_bonus = v; });
    pub const dodge_bonus: Field<CombatData, i32> = Field::new(stringify!( dodge_bonus ), |t| &t.dodge_bonus, |t| &mut t.dodge_bonus, |t, v| { t.dodge_bonus = v; });
    pub const defense_bonus: Field<CombatData, i32> = Field::new(stringify!( defense_bonus ), |t| &t.defense_bonus, |t| &mut t.defense_bonus, |t, v| { t.defense_bonus = v; });
    pub const block_bonus: Field<CombatData, i32> = Field::new(stringify!( block_bonus ), |t| &t.block_bonus, |t| &mut t.block_bonus, |t, v| { t.block_bonus = v; });
}

impl FactionData {
    pub const name: Field<FactionData, String> = Field::new(stringify!( name ), |t| &t.name, |t| &mut t.name, |t, v| { t.name = v; });
    pub const color: Field<FactionData, Color> = Field::new(stringify!( color ), |t| &t.color, |t| &mut t.color, |t, v| { t.color = v; });
}

impl InventoryData {
    pub const equipped: Field<InventoryData, Vec<Entity>> = Field::new(stringify!( equipped ), |t| &t.equipped, |t| &mut t.equipped, |t, v| { t.equipped = v; });
    pub const inventory: Field<InventoryData, Vec<Entity>> = Field::new(stringify!( inventory ), |t| &t.inventory, |t| &mut t.inventory, |t, v| { t.inventory = v; });
}

impl ItemData {
    pub const attacks: Field<ItemData, Vec<Attack>> = Field::new(stringify!( attacks ), |t| &t.attacks, |t| &mut t.attacks, |t, v| { t.attacks = v; });
    pub const held_by: Field<ItemData, Option<Entity>> = Field::new(stringify!( held_by ), |t| &t.held_by, |t| &mut t.held_by, |t, v| { t.held_by = v; });
}

impl MapData {
    pub const min_tile_bound: Field<MapData, AxialCoord> = Field::new(stringify!( min_tile_bound ), |t| &t.min_tile_bound, |t| &mut t.min_tile_bound, |t, v| { t.min_tile_bound = v; });
    pub const max_tile_bound: Field<MapData, AxialCoord> = Field::new(stringify!( max_tile_bound ), |t| &t.max_tile_bound, |t| &mut t.max_tile_bound, |t, v| { t.max_tile_bound = v; });
}

impl SkillData {
    pub const skill_bonuses: Field<SkillData, EnumMap<Skill, u32>> = Field::new(stringify!( skill_bonuses ), |t| &t.skill_bonuses, |t| &mut t.skill_bonuses, |t, v| { t.skill_bonuses = v; });
    pub const skill_xp: Field<SkillData, EnumMap<Skill, u32>> = Field::new(stringify!( skill_xp ), |t| &t.skill_xp, |t| &mut t.skill_xp, |t, v| { t.skill_xp = v; });
}

impl TileData {
    pub const name: Field<TileData, Str> = Field::new(stringify!( name ), |t| &t.name, |t| &mut t.name, |t, v| { t.name = v; });
    pub const position: Field<TileData, AxialCoord> = Field::new(stringify!( position ), |t| &t.position, |t| &mut t.position, |t, v| { t.position = v; });
    pub const move_cost: Field<TileData, Sext> = Field::new(stringify!( move_cost ), |t| &t.move_cost, |t| &mut t.move_cost, |t, v| { t.move_cost = v; });
    pub const cover: Field<TileData, i8> = Field::new(stringify!( cover ), |t| &t.cover, |t| &mut t.cover, |t, v| { t.cover = v; });
    pub const occupied_by: Field<TileData, Option<Entity>> = Field::new(stringify!( occupied_by ), |t| &t.occupied_by, |t| &mut t.occupied_by, |t, v| { t.occupied_by = v; });
    pub const elevation: Field<TileData, i8> = Field::new(stringify!( elevation ), |t| &t.elevation, |t| &mut t.elevation, |t, v| { t.elevation = v; });
}

impl TurnData {
    pub const turn_number: Field<TurnData, u32> = Field::new(stringify!( turn_number ), |t| &t.turn_number, |t| &mut t.turn_number, |t, v| { t.turn_number = v; });
    pub const active_faction: Field<TurnData, Entity> = Field::new(stringify!( active_faction ), |t| &t.active_faction, |t| &mut t.active_faction, |t, v| { t.active_faction = v; });
}

impl PositionData { pub const hex: Field<PositionData, AxialCoord> = Field::new(stringify!( hex ), |t| &t.hex, |t| &mut t.hex, |t, v| { t.hex = v; }); }

impl IdentityData {
    pub const name: Field<IdentityData, Option<String>> = Field::new(stringify!( name ), |t| &t.name, |t| &mut t.name, |t, v| { t.name = v; });
    pub const kind: Field<IdentityData, Taxon> = Field::new(stringify!( kind ), |t| &t.kind, |t| &mut t.kind, |t, v| { t.kind = v; });
}

impl ActionData {
    pub const active_action: Field<ActionData, Option<Action>> = Field::new(stringify!( active_action ), |t| &t.active_action, |t| &mut t.active_action, |t, v| { t.active_action = v; });
    pub const active_reaction: Field<ActionData, ReactionType> = Field::new(stringify!( active_reaction ), |t| &t.active_reaction, |t| &mut t.active_reaction, |t, v| { t.active_reaction = v; });
    pub const available_action_types: Field<ActionData, HashSet<ActionType>> = Field::new(stringify!( available_action_types ), |t| &t.available_action_types, |t| &mut t.available_action_types, |t, v| { t.available_action_types = v; });
}

impl ModifierTrackingData { pub const modifiers_by_key: Field<ModifierTrackingData, HashMap<String, ModifierReference>> = Field::new(stringify!( modifiers_by_key ), |t| &t.modifiers_by_key, |t| &mut t.modifiers_by_key, |t, v| { t.modifiers_by_key = v; }); }