use common::prelude::*;
use gui::*;
use game::entities::{Attack, IdentityData, ItemArchetype};
use attack_descriptions::*;
use itertools::Either;
use game::prelude::*;

#[derive(WidgetContainer)]
pub struct ItemDisplayWidget {
    pub body : Widget,
    pub name_display : Widget,
    pub attack_displays : ListWidget<AttackDescriptionWidget>,
}
impl DelegateToWidget for ItemDisplayWidget {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

impl ItemDisplayWidget {
    pub fn new() -> ItemDisplayWidget {
        let body = Widget::segmented_window("ui/window/minimalist")
            .width(30.ux())
            .height(40.ux());

        let name_display = Widget::text("", FontSize::HeadingMajor)
            .centered_horizontally()
            .parent(&body);

        let attack_displays = ListWidget::featherweight()
            .parent(&body)
            .item_gap(1.ux())
            .width(Sizing::ExtendToParentEdge)
            .surround_children_v()
            .below(&name_display, 1.ux());

        ItemDisplayWidget { body, attack_displays, name_display }
    }

    pub fn update(&mut self, view : &WorldView, gui : &mut GUI, item : Either<(&IdentityData,&ItemArchetype), Entity>) {
        self.body.reapply(gui);
        match item {
            Either::Left((identity, archetype)) => {
                self.name_display.set_text(identity.effective_name().to_string().capitalized()).reapply(gui);
                self.attack_displays.update(gui, &archetype.attacks, |widget, (identity,attack)| {
                    widget.update(attack, identity, false, false);
                });

            },
            Either::Right(entity) => {
                warn!("Entity item display not implemented yet")
            }
        }
    }
}