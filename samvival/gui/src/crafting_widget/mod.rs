pub mod crafting_output_widget;
pub mod ingredient_assignment_widget;

use common::prelude::*;
use game::entities::recipes::*;
use game::entities::{EntitySelector,InventoryData,ActionData,ItemArchetype,IdentityData,StackData};
use game::logic::item;
use game::logic::crafting;
use game::archetype::EntityArchetype;
use game::entities::item::*;
use game::prelude::*;

use gui::*;
use common::color::Color;
use recipe_selection_widget::*;
use inventory_widget::*;
use state::ControlContext;
use state::GameState;
use attack_descriptions::*;
use itertools::Either;
use std::collections::HashMap;
use item_display_widget::ItemDisplayWidget;
use crafting_widget::crafting_output_widget::CraftingOutputWidget;
use crafting_widget::ingredient_assignment_widget::*;
use std::collections::HashSet;

#[derive(Clone,Debug)]
pub(crate) enum CraftWidgetInternalEvent {
    Craft,
    Cancel
}

#[derive(DelegateToWidget)]
pub struct CraftingWidget {
    body : Widget,
    recipe_selector : BaseRecipeSelector,
    inventory_widget : InventoryDisplayWidget,
    output_widget : CraftingOutputWidget,
    ingredient_assignment_widget : IngredientAssignmentWidget,
    selected_item : Option<Entity>,
    selected_base_recipe: Option<Entity>,
    ingredient_assignments: HashMap<Taxon, Vec<Entity>>,
}

impl CraftingWidget {
    pub fn new(parent : &Widget) -> Self {
        let body = Widget::segmented_window("ui/window/green_minor_styled")
            .width(80.ux())
            .height(80.ux())
//            .margin(1.ux())
            .showing(false)
            .centered()
            .parent(parent);

        let recipe_selector = BaseRecipeSelector::new(&body)
            .width(38.ux())
            .height(50.ux())
            .position(0.px(), 0.px());

        let inventory_widget = InventoryDisplayWidget::new(&body)
            .width(Sizing::match_parent())
            .height(28.ux())
            .below(&recipe_selector, 1.ux());

        let output_widget = CraftingOutputWidget::new()
            .parent(&body)
            .width(38.ux())
            .height(50.ux())
            .align_right()
            .position(0.px(), 0.px());

        let ingredient_assignment_widget = IngredientAssignmentWidget::new()
            .parent(&body)
            .width(38.ux())
            .height(50.ux())
            .position(0.px(), 0.px());

        CraftingWidget {
            body,
            recipe_selector,
            inventory_widget,
            output_widget,
            selected_item : None,
            selected_base_recipe: None,
            ingredient_assignments: HashMap::new(),
            ingredient_assignment_widget,
        }
    }

    pub fn toggle(&mut self, view : &WorldView, gui : &mut GUI) {
        self.body.reapply(gui);

        self.recipe_selector.update(view, gui);
        if self.body.showing {
            self.body.hide().reapply(gui);
        } else {
            self.body.show().reapply(gui);
        }
    }

    pub fn update(&mut self, view : &WorldView, gui : &mut GUI, game_state : &GameState, control : &mut ControlContext) {
        self.body.reapply(gui);

        if let Some(selected) = game_state.selected_character {
            let inv_data = view.data::<InventoryData>(selected);
            let items = &inv_data.items;
            let all_assigned_items : HashSet<Entity> = self.ingredient_assignments.values().flat_map(|v| v.iter().cloned()).collect();
            let destacked = item::items_in_inventory(view, selected);
            let main_inv = vec![InventoryDisplayData::new(items.clone(), destacked, all_assigned_items, "Character Inventory", vec![selected], true, inv_data.inventory_size)];

            self.inventory_widget.update(gui, view, &main_inv, self.selected_item);

            if let Some(recipe) = self.selected_base_recipe {
                self.recipe_selector.hide().reapply(gui);
                let recipe_dat = view.data::<Recipe>(recipe);
                self.ingredient_assignment_widget.show();
                self.ingredient_assignment_widget.update(view, gui, &recipe_dat.ingredients_by_kind, &self.ingredient_assignments);
            } else {
                self.recipe_selector.show().reapply(gui);
                self.ingredient_assignment_widget.hide().reapply(gui);
            }

            for evt in gui.events_for(&self.body) {
                if let Some(InventoryItemSelected { inventory_index, item_index }) = evt.as_custom_event_no_origin() {
                    let new_selected = main_inv.get(inventory_index).and_then(|idd : &InventoryDisplayData| idd.items.get(item_index)).cloned();
                    if self.selected_item == new_selected {
                        self.selected_item = None;
                    } else {
                        self.selected_item = new_selected;
                    }
                } else if let Some(BaseRecipeSelected { recipe }) = evt.as_custom_event_no_origin() {
                    self.selected_base_recipe = Some(recipe);
                } else if let Some(IngredientAssignmentEvent::SelectIngredientSlot(kind)) = evt.as_custom_event_no_origin() {
                    if let Some(item) = self.selected_item {
                        if self.assign_item_to_ingredient_slot(view, selected, &kind, item) {
                            self.selected_item = None;
                        }
                    } else {
                        info!("no item selected, so no assignment made");
                    }
                } else if let Some(IngredientAssignmentEvent::ClearIngredientSlot(kind)) = evt.as_custom_event_no_origin() {
                    self.ingredient_assignments.remove(&kind);
                } else if let Some(CraftWidgetInternalEvent::Cancel) = evt.as_custom_event_no_origin() {
                    self.selected_base_recipe = None;
                    self.ingredient_assignments.clear();
                } else if let Some(CraftWidgetInternalEvent::Craft) = evt.as_custom_event_no_origin() {
                    println!("PERFORM CRAFT HERE");
                }
            }

            self.output_widget.update(view, gui, selected, self.selected_base_recipe, &self.ingredient_assignments);
        }
    }

    pub fn assign_item_to_ingredient_slot(&mut self, view : &WorldView, selected_character : Entity, kind : &Taxon, item : Entity) -> bool {
        if let Some(base_recipe) = self.selected_base_recipe {
            let item = item::item_or_first_in_stack(view, item);
            // create a hypothetical where we're sure we haven't yet assigned this kind
            let mut hypothetical_assignments = self.ingredient_assignments.clone();
            hypothetical_assignments.remove(kind);

            if crafting::can_item_be_used_in_craft(view, base_recipe, item, kind, &hypothetical_assignments) {
                info!("Assigning items to slot");
                let amount_required = view.data::<Recipe>(base_recipe).ingredients_by_kind.get(kind).map(|i| i.amount_required).unwrap_or(1);
                let destacked_items = item::items_in_inventory(view, selected_character);
                let already_assigned : HashSet<Entity> = hypothetical_assignments.values().flat_map(|v| v.iter().cloned()).collect();
                let all_valid = destacked_items.iter()
                    .filter(|e| ! already_assigned.contains(*e))
                    .filter(|e| item::can_items_stack_together(view, item, **e))
                    .cloned()
                    .take(amount_required as usize).collect();
                self.ingredient_assignments.insert(kind.clone(), all_valid);
                true
            } else {
                info!("Item could not be used in craft");
                false
            }
        } else {
            warn!("No base recipe selected, how are we assigning items?");
            false
        }
    }
}