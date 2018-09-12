use gui::*;
use game::logic::combat::*;
use common::prelude::*;
use game::prelude::*;
use common::color::Color;
use game::logic::harvest;
use game::entities::Harvestable;
use state::ControlContext;
use control_events::TacticalEvents;
use common::hex::AxialCoord;
use game::logic::breakdown::Breakdown;
use game::logic::harvest::HarvestBreakdown;
use graphics::renderers::ItemRenderer;
use graphics::GraphicsResources;

pub struct HarvestSummaryWidget {
    harvestable_summaries: ListWidget<HarvestableSummary>
}

impl DelegateToWidget for HarvestSummaryWidget {
    fn as_widget(&mut self) -> &mut Widget { self.harvestable_summaries.as_widget() }
    fn as_widget_immut(&self) -> &Widget { self.harvestable_summaries.as_widget_immut() }
}

impl HarvestSummaryWidget {
    pub fn new() -> HarvestSummaryWidget {
        HarvestSummaryWidget {
            harvestable_summaries: ListWidget::custom(Widget::div(), 1.ux()).surround_children()
        }
    }
}

#[derive(WidgetContainer, DelegateToWidget)]
struct HarvestableSummary {
    pub body: Widget,
    pub resource_icon: Widget,
    pub action_description: Widget,
    pub harvest_amount: Widget,
    pub time: Widget,
}

impl Default for HarvestableSummary {
    fn default() -> Self {
        let body = Widget::div();

        let resource_icon = Widget::image("ui/blank", Color::white(), 1).parent(&body).named("resource_icon");
        let action_description = Widget::text("", FontSize::HeadingMinor).parent(&body).right_of(&resource_icon, 4.px()).named("action_description");
        let time = Widget::text("", FontSize::HeadingMinor).parent(&body).right_of(&action_description, 18.px()).named("harvest_time");
        let harvest_amount = Widget::text("", FontSize::HeadingMinor).parent(&body).below(&action_description, 4.px()).right_of(&resource_icon, 4.px()).named("harvest_amount");


        HarvestableSummary {
            body,
            resource_icon,
            action_description,
            harvest_amount,
            time,
        }
    }
}

impl HarvestableSummary {
    pub fn update(&mut self, view: &WorldView, graphics: &mut GraphicsResources, breakdown: &HarvestBreakdown, greyed_out: bool) {
        let harvestable_data = view.data::<Harvestable>(breakdown.harvestable);
        let resource_ident = view.data::<IdentityData>(breakdown.resource);
        let resource_img = ItemRenderer::image_for(graphics, resource_ident.main_kind());
        self.resource_icon.set_widget_type(WidgetType::image(resource_img));
        self.action_description.set_text(harvestable_data.action_name.capitalized());


        let fixed_limit = breakdown.harvest_limit.min(breakdown.inventory_limit.unwrap_or(100000000));
        let min_yield = (breakdown.dice_amount_harvested.total.min_roll() as i32 + breakdown.fixed_amount_harvested.total).min(fixed_limit);
        let max_yield = (breakdown.dice_amount_harvested.total.max_roll() as i32 + breakdown.fixed_amount_harvested.total).min(fixed_limit);

//        let combined_dice_str = breakdown.dice_amount_harvested.total.to_d20_string();
//        let combined_fixed = match breakdown.fixed_amount_harvested.total {
//            0 => String::new(),
//            other => format!(" {} {}", other.sign_str(), other.abs()),
//        };
        let resource_name = resource_ident.effective_name();
//        self.harvest_amount.set_text(format!("{}{} {}", combined_dice_str, combined_fixed, resource_name));
        let harvest_str = if breakdown.inventory_limit == Some(0) {
            strf("Not enough inventory space")
        } else {
            let yield_str = if min_yield == max_yield { format!("{}", min_yield) } else { format!("{}-{}", min_yield, max_yield) };
            format!("{} {}", yield_str, resource_name)
        };

        self.harvest_amount.set_text(harvest_str);
        self.time.set_text(format!("{} AP", breakdown.ap_to_harvest.total));

        let body_id = self.body.id();
        self.for_all_widgets(|w: &mut Widget| {
            if w.id() != body_id {
                w.set_color(if greyed_out { Color::new(0.5, 0.5, 0.5, 1.0) } else {
                    if let WidgetType::Text { .. } = w.widget_type {
                        Color::black()
                    } else {
                        Color::white()
                    }
                });
            }
        })
    }
}

impl HarvestSummaryWidget {
    pub fn update(&mut self, gui: &mut GUI, world: &World, view: &WorldView, gsrc: &mut GraphicsResources, pixel_pos: Vec2f, harvester: Entity, harvest_from: AxialCoord, preserve_renewable: bool, greyed_out: bool) {
        let harvest_breakdowns = harvest::harvestables_sorted_by_desirability_at(view, harvester, harvest_from).iter().flat_map(|harvestable|
            harvest::compute_harvest_breakdown(world, view, harvester, harvest_from, *harvestable, preserve_renewable)).collect_vec();
        if !harvest_breakdowns.is_empty() {
            self.harvestable_summaries.update(gui, &harvest_breakdowns, |widget, breakdown| {
                widget.update(view, gsrc, breakdown, greyed_out)
            });

            self.set_showing(true).set_position(Positioning::constant((pixel_pos.x + 20.0).px()), Positioning::constant(pixel_pos.y.px())).reapply(gui);
        } else {
            self.set_showing(false).reapply(gui);
        }

        self.reapply(gui);
    }
}