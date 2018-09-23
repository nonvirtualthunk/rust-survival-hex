use common::prelude::*;
use gui::*;
use gui::text_display_widget::TextDisplayWidget;
use game::entities::{IdentityData,recipes::*,ItemArchetype,Taxon};
use std::collections::HashMap;
use common::color::Color;
use game::prelude::*;
use std::rc::Rc;

#[derive(Clone)]
pub enum IngredientAssignmentEvent {
    SelectIngredientSlot(Taxon),
    ClearIngredientSlot(Taxon),
}

pub struct IngredientAssignmentWidget {
    body : Widget,
    label : Widget,
    ingredient_assignments : ListWidget<IngredientAssignmentSubWidget>
}
impl DelegateToWidget for IngredientAssignmentWidget {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}


impl IngredientAssignmentWidget {
    pub fn new() -> IngredientAssignmentWidget {
        let body = Widget::segmented_window("ui/window/fancy")
            .color(Color::greyscale(0.8))
            .width(30.ux())
            .height(50.ux());

        let label = Widget::text("Assign Ingredients", FontSize::HeadingMajor)
            .centered_horizontally()
            .parent(&body);

        let ingredient_assignments = ListWidget::featherweight()
            .parent(&body)
            .item_gap(4.ux())
            .width(Sizing::match_parent())
            .surround_children_v()
//            .height(40.ux())
            .below(&label, 1.ux())
            .with_ui_callback(|ctxt, evt| {
                if let UIEvent::WidgetEvent { event : WidgetEvent::ListItemClicked(index, button), .. } = evt {
                    if let Some(taxon) = ctxt.custom_data::<Taxon>() {
                        println!("ingredient assignment selected : {:?}", taxon);
                        if button == &MouseButton::Left {
                            ctxt.trigger_custom_event(IngredientAssignmentEvent::SelectIngredientSlot((*taxon).clone()));
                        } else {
                            ctxt.trigger_custom_event(IngredientAssignmentEvent::ClearIngredientSlot((*taxon).clone()));
                        }
                    } else {
                        if ! ctxt.has_custom_data() {
                            println!("No custom data!");
                        } else { println!("Wrong kind of custom data!") }
                    }
                }
            })
        ;

        IngredientAssignmentWidget { body, label, ingredient_assignments }
    }


    pub fn update(&mut self, view : &WorldView, gui : &mut GUI, requirements : &HashMap<Taxon, Ingredient>, ingredients : &HashMap<Taxon, Vec<Entity>>) {
        self.body.reapply(gui);
        self.label.reapply(gui);

        let mut data : Vec<(Taxon, (Ingredient, Option<Vec<Entity>>))> = Vec::new();
        for (kind, ingredient) in requirements {
            let corresponding_entities = ingredients.get(kind);
            data.push((kind.clone(), (ingredient.clone(), corresponding_entities.cloned())));
        }
        self.ingredient_assignments.update_with_row(gui, &data, |widget, (kind, (ingredient, assigned)), row| {
            row.set_custom_data(kind.clone());
            widget.name_display.set_text(kind.name().to_string().capitalized());
            if let Some(first_assigned) = assigned.clone().and_then(|a| a.first().cloned()) {
                widget.selector_display.hide();
                let ident = view.identity(first_assigned);
                let current_assigned_count = assigned.iter().map(|a| a.len()).next().unwrap_or(0) as i32;
                if current_assigned_count < ingredient.amount_required && current_assigned_count != 0 {
                    widget.item_display.set_color(Color::new(0.8,0.2,0.2,1.0));
                }
                let amount_str = if current_assigned_count >= ingredient.amount_required {
                    format!("{}",current_assigned_count)
                } else {
                    format!("{}/{}", current_assigned_count, ingredient.amount_required)
                };
                widget.item_display.show()
                    .set_text(format!("{} {}", amount_str, ident.effective_name().to_string().capitalized()));
            } else {
                widget.item_display.hide();
                let selector_str = ingredient.ingredient_selector.to_string_general(view);
                let amount = ingredient.amount_required;
                widget.selector_display.show()
                    .set_text(format!("{} {}", amount, selector_str));
            }
        });
    }
}

#[derive(WidgetContainer)]
struct IngredientAssignmentSubWidget {
    pub name_display : Widget,
    pub selector_display : TextDisplayWidget,
    pub item_display : TextDisplayWidget,
}
//impl WidgetContainer for IngredientAssignmentSubWidget {
//    fn for_each_widget<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
//        (func)(&mut self.name_display);
//        (func)(self.selector_display.as_widget());
//        (func)(self.item_display.as_widget());
//    }
//}

impl Default for IngredientAssignmentSubWidget {
    fn default() -> Self {
        let name_display = Widget::text("", FontSize::HeadingMinor).centered_horizontally();

        let selector_display = TextDisplayWidget::new("A", FontSize::Standard, "ui/window/minimalist", ImageSegmentation::All)
            .color(Color::greyscale(0.9))
            .width(Sizing::ExtendToParentEdge)
            .below(&name_display, 1.ux())
            .centered_text();

        let item_display = TextDisplayWidget::new("B", FontSize::Standard, "ui/window/minimalist", ImageSegmentation::All)
            .color(Color::greyscale(0.9))
            .width(Sizing::ExtendToParentEdge)
            .below(&name_display, 1.ux())
            .centered_text()
        ;

        IngredientAssignmentSubWidget { name_display, selector_display, item_display }
    }
}