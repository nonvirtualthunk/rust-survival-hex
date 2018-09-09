use common::prelude::*;
use prelude::*;
use entities::tile::*;
use logic;
use logic::breakdown::Breakdown;
use entities::ItemData;
use entities::actions::*;
use entities::InventoryData;
use entities::ToolData;

pub fn harvestables_at(world : &WorldView, coord : AxialCoord) -> Vec<Entity> {
    world.terrain(coord).harvestables.values()
        .chain(world.vegetation(coord).harvestables.values())
        .filter(|h| world.has_data::<Harvestable>(**h))
        .cloned()
        .collect()
}

pub fn can_harvest(world: &WorldView, character : Entity, harvestable: Entity) -> (bool, String) {
    if let Some(harvestable_data) = world.data_opt::<Harvestable>(harvestable) {
        if harvestable_data.amount.cur_value().as_u32_or_0() == 0 {
            (false, strf("nothing left to harvest"))
        } else if ! harvestable_data.character_requirements.matches(world, character) {
            (false, strf("character does not have the necessary skills or abilities"))
        } else if harvestable_data.requires_tool() && harvest_tool_to_use_for(world, character, &harvestable).is_none() {
            (false, strf("character does not have an appropriate tool equipped"))
        } else {
            (true, harvestable_data.action_name.clone())
        }
    } else {
        (false, strf("nothing to harvest"))
    }
}

pub fn harvest_tool_to_use_for(world: &WorldView, character: Entity, harvestable: &impl IntoHarvestable) -> Option<Entity> {
//    let harvestable_data = world.data::<Harvestable>(harvestable);
    let harvestable_data = harvestable.harvestable_data(world);
    for item in logic::item::equipped_items(world, character) {
        if harvestable_data.tool.matches(world, item) {
            return Some(item);
        }
    }
    None
}


pub struct HarvestBreakdown {
    pub harvestable : Entity,
    pub resource : Entity,
    pub harvest_from : Entity,
    pub harvester : Entity,
    pub tool : Option<Entity>,
    pub difficulty_reason : Option<String>, // if present indicates why this is more difficult to harvest than usual (generally lack of tool)
    pub dice_amount_harvested : Breakdown<DicePool>,
    pub fixed_amount_harvested : Breakdown<i32>,
    pub harvest_limit : i32,
    pub amount_remaining : i32,
    pub inventory_limit : Option<i32>,
    pub ap_to_harvest : Breakdown<i32>,

}

pub fn compute_harvest_breakdown(world: &World, view: &WorldView, character : Entity, from : AxialCoord, harvestable : Entity, preserve_renewable : bool) -> Result<HarvestBreakdown, String> {
    if let Some(tile) = view.tile_ent_opt(from) {
        let (can_harvest, reason) = can_harvest(view, character, harvestable);
        if can_harvest {
            let harvestable_data = world.data::<Harvestable>(harvestable);
            let tool_opt = harvest_tool_to_use_for(world, character, harvestable_data);
            let mut difficulty_reason = None;

            let harvestable_field_logs = world.field_logs_for::<Harvestable>(harvestable);
            let mut dice_amount_harvested = Breakdown::new();
            dice_amount_harvested.add_field(harvestable_data.dice_amount_per_harvest.clone(), &harvestable_field_logs, &Harvestable::dice_amount_per_harvest, "harvest");

            let mut fixed_amount_harvested = Breakdown::new();
            fixed_amount_harvested.add_field(harvestable_data.fixed_amount_per_harvest.clone(), &harvestable_field_logs, &Harvestable::fixed_amount_per_harvest, "harvest");

            let mut ap_to_harvest = Breakdown::new();
            ap_to_harvest.add_field(harvestable_data.ap_per_harvest as i32, &harvestable_field_logs, &Harvestable::ap_per_harvest, "ap to harvest");
            if let Some(tool) = tool_opt.and_then(|t| world.data_opt::<ToolData>(t)) {
                ap_to_harvest.add(-tool.tool_speed_bonus, "tool speed");
                dice_amount_harvested.add(tool.tool_harvest_dice_bonus.clone(), "tool bonus");
                fixed_amount_harvested.add(tool.tool_harvest_fixed_bonus, "tool bonus");
            }

            let mut harvest_limit = harvestable_data.amount.cur_value().as_i32();
            if preserve_renewable && harvestable_data.renew_rate.is_some() {
                harvest_limit -= 1;
            }
            harvest_limit = harvest_limit.max(0);

            let inventory_limit = logic::item::inventory_limit_remaining_for(view, character, harvestable_data.resource);

            if tool_opt.is_none() {
                if let ToolUse::DifficultWithout { amount_limit, ap_increase } = harvestable_data.tool_use {
                    difficulty_reason = Some(format!("difficult without {}", harvestable_data.tool.article_string(view)));
                    if let Some(amount_limit) = amount_limit {
                        harvest_limit = harvest_limit.min(amount_limit);
                    }
                    if let Some(ap_increase) = ap_increase {
                        ap_to_harvest.add(ap_increase, "no tool");
                    }
                }
            }

            Ok(HarvestBreakdown {
                harvestable,
                resource : harvestable_data.resource,
                tool : tool_opt,
                difficulty_reason,
                harvest_from : tile.entity,
                harvester : character,
                dice_amount_harvested,
                fixed_amount_harvested,
                ap_to_harvest,
                harvest_limit,
                inventory_limit,
                amount_remaining : harvestable_data.amount.cur_value().as_i32()
            })
        } else { Err(reason) }
    } else { Err(strf("invalid tile")) }
}

pub fn harvest(world: &mut World, character : Entity, from : AxialCoord, harvestable : Entity, preserve_renewable : bool, progress : Option<i32>) {
    if let Ok(breakdown) = compute_harvest_breakdown(world, world.view(), character, from, harvestable, preserve_renewable) {
        let cdata = world.view().data::<CharacterData>(character);

        let action_type = ActionType::Harvest { from, harvestable, preserve_renewable };

        // Harvesting can be a multi-turn action, so we first want to figure out if we're actually resolving the harvest, or just making progress
        // toward its eventual completion
        let ap_so_far = progress.unwrap_or(0);
        let ap_required = breakdown.ap_to_harvest.total;
        let ap_remaining = cdata.action_points.cur_value();
        // if our progress so far, plus the ap we can contribute this turn are still less than what's required, update our action-in-progress
        if ap_required > ap_remaining + ap_so_far {
            world.modify(character, CharacterData::action_points.reduce_by(ap_remaining));
            let in_progress_action = Action {
                action_type : action_type.clone(),
                ap : Progress::new(ap_remaining + ap_so_far, ap_required)
            };
            world.modify(character, ActionData::active_action.set_to(Some(in_progress_action.clone())));
            let event = GameEvent::ActionTaken { entity : character, action : in_progress_action };
            if progress.is_none() {
                world.start_event(event);
            } else {
                world.continue_event(event);
            }
        } else {
            // if we're here then we can actually complete the harvest action
            world.modify(character, CharacterData::action_points.reduce_by(ap_required - ap_so_far));

            let mut rng = world.random(9221);
            let amount_harvested = (breakdown.dice_amount_harvested.total.roll(&mut rng).total_result as i32 + breakdown.fixed_amount_harvested.total)
                .min(breakdown.harvest_limit)
                .min(breakdown.inventory_limit.unwrap_or(1000000));
            let harvestable_data = world.view().data::<Harvestable>(harvestable);
            world.modify(harvestable, Harvestable::amount.reduce_by(Sext::of(amount_harvested)));

            for i in 0 .. amount_harvested {
                let new_entity = world.clone_entity(harvestable_data.resource);
                logic::item::put_item_in_inventory(world, new_entity, character);
            }

            let completed_action = Action { action_type : action_type.clone(), ap : Progress::new(ap_required, ap_required) };
            if world.data::<ActionData>(character).active_action.is_some() { // clear the active action, if any
                world.modify(character, ActionData::active_action.set_to(None));
            }
            let event = GameEvent::ActionTaken { entity : character, action : completed_action };
            if progress.is_some() { world.end_event(event); } else { world.add_event(event); }
        }
    }
}