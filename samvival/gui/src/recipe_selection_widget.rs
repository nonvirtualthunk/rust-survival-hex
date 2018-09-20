use common::prelude::*;
use game::entities::recipes::*;
use game::entities::EntitySelector;
use game::archetype::EntityArchetype;
use game::entities::item::*;
use game::prelude::*;

use gui::*;
use gui::compound_widgets::TextDisplayWidget;
use common::color::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaseRecipeSelected { pub recipe : Entity }

pub struct BaseRecipeSelector {
    body: Widget,
    label : Widget,
    recipe_list : ListWidget<Button>,
    catalog : Option<RecipeCatalogView>
}
impl DelegateToWidget for BaseRecipeSelector {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

impl BaseRecipeSelector {
    pub fn new(parent : &Widget) -> Self {
        let body = Widget::segmented_window("ui/window/fancy")
            .color(Color::greyscale(0.8))
            .width(20.ux())
            .height(40.ux())
//            .margin(2.px())
            .parent(parent);

        let label = Widget::text("Recipes", FontSize::Large).x(Positioning::centered()).parent(&body);

        let recipe_list = ListWidget::featherweight().x(0.px()).width(Sizing::ExtendToParentEdge).surround_children_v().below(&label, 1.ux()).parent(&body);

        BaseRecipeSelector {
            label,
            recipe_list,
            body,
            catalog : None,
        }
    }

    pub fn update(&mut self, view : &WorldView, gui : &mut GUI) {
        self.body.reapply(gui);
        if self.catalog.is_none() {
            let catalog = RecipeCatalogView::of(view);

            self.label.reapply(gui);

            println!("Number of root recipes: {}", catalog.root_recipes().len());
            self.recipe_list.update(gui, catalog.root_recipes(), |widget, recipe| {
                widget.set_font_size(FontSize::HeadingMajor)
                    .set_width(Sizing::match_parent());
                match view.data::<Recipe>(*recipe).result {
                    EntityArchetype::Archetype(arch) => widget.set_text(view.identity(arch).effective_name().to_string().capitalized()),
                    EntityArchetype::CopyEnitity(entity) => widget.set_text(view.identity(entity).effective_name().to_string().capitalized()),
                    EntityArchetype::Sentinel => widget.set_text("Sentinel"),
                };

                let widget_id = widget.id();
                let recipe = *recipe;
                widget.add_on_click(move |ctxt, evt| {
                    println!("On click triggered");
                   ctxt.trigger_event(UIEvent::custom_event(BaseRecipeSelected { recipe }, widget_id));
                });
            });

            self.catalog = Some(catalog);
        }
    }
}