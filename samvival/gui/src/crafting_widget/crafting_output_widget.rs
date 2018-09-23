use common::prelude::*;

use item_display_widget::ItemDisplayWidget;
use gui::*;
use common::Color;
use crafting_widget::CraftWidgetInternalEvent;
use std::collections::HashMap;
use itertools::Either;
use game::logic::crafting;
use game::prelude::*;


#[derive(WidgetContainer)]
pub struct CraftingOutputWidget {
    pub body : Widget,
    pub item_display : ItemDisplayWidget,
    pub craft_button : Button,
    pub cancel_button : Button,
    pub reason_display : Widget,
}
impl DelegateToWidget for CraftingOutputWidget {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

impl CraftingOutputWidget {
    pub fn new() -> CraftingOutputWidget {
        let body = Widget::segmented_window("ui/window/fancy")
            .color(Color::greyscale(0.8))
            .width(30.ux())
            .height(50.ux());

        let item_display = ItemDisplayWidget::new()
            .width(Sizing::match_parent())
            .parent(&body);

        let craft_button = Button::segmented("Craft", "ui/window/grey_button")
            .parent(&body)
            .with_on_click(|ctxt, evt| { ctxt.trigger_custom_event(CraftWidgetInternalEvent::Craft)} )
            .font_size(FontSize::HeadingMinor)
            .align_bottom()
            .align_right()
            .x(1.ux())
            .y(1.ux());

        let cancel_button = Button::segmented("Cancel", "ui/window/grey_button")
            .parent(&body)
            .with_on_click(|ctxt, evt| { ctxt.trigger_custom_event(CraftWidgetInternalEvent::Cancel)} )
            .font_size(FontSize::HeadingMinor)
            .align_bottom()
            .x(1.ux())
            .y(1.ux());

        let reason_display = Widget::wrapped_text("", FontSize::HeadingMinor, TextWrap::WithinParent)
            .parent(&body)
            .hidden()
            .centered();

        CraftingOutputWidget { body, item_display, craft_button, cancel_button, reason_display }
    }

    pub fn update(&mut self, world : &mut World, view : &WorldView, gui : &mut GUI, crafter : Entity, base_recipe : Option<Entity>, ingredients : &HashMap<Taxon, Vec<Entity>>) {
        self.body.reapply(gui);
        if let Some(base_recipe) = base_recipe {
            match crafting::compute_crafting_breakdown(world, view, crafter, ingredients, base_recipe) {
                Ok(breakdown) => {
                    self.reason_display.hide().reapply(gui);
                    self.craft_button.show().reapply(gui);
                    self.item_display.show().update(view, gui, Either::Left((&breakdown.result_identity, &breakdown.effective_archetype)));
                },
                Err(reason) => {
                    self.reason_display.show()
                        .set_text(reason)
                        .reapply(gui);
                    self.craft_button.hide().reapply(gui);
                    self.item_display.hide().reapply(gui);
                },
            }
            self.cancel_button.show().reapply(gui);
        } else {
            self.item_display.hide().reapply(gui);
            self.craft_button.hide().reapply(gui);
            self.cancel_button.hide().reapply(gui);
            self.reason_display.hide().reapply(gui);
        }
    }
}