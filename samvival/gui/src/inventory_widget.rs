use common::prelude::*;
use gui::prelude::*;
use gui::ListWidget;
use common::Color;
use game::prelude::*;
use game::entities::IdentityData;
use gui::TabWidget;

// we actually want two things here, one to just display an inventory, and another to display multiple inventories together
// to allow moving items back and forth between them.


pub struct InventoryDisplay {
    body: Widget,
    main_inventory: InventoryDisplayWidget,
    other_inventories_tabs: Option<TabWidget>,
    other_inventory_widgets: Vec<InventoryDisplayWidget>,

    last_other_inv_count: usize,
    main_inventory_data : InventoryDisplayData,
    other_inventory_data : Vec<InventoryDisplayData>,
}

impl DelegateToWidget for InventoryDisplay {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

#[derive(Clone)]
pub struct InventoryDisplayData {
    pub name: String,
    pub items: Vec<Entity>,
}
impl InventoryDisplayData {
    pub fn new<S : Into<String>>(items : Vec<Entity>, name : S) -> InventoryDisplayData {
        InventoryDisplayData {
            items,
            name : name.into()
        }
    }
}

impl InventoryDisplay {
    pub fn new(main_inv_name: String, parent: &Widget) -> InventoryDisplay {
        let body = Widget::div().color(Color::new(1.0,0.5,0.5,0.5)).margin(5.px()).centered().named("Inventory display parent div").parent(parent);
        InventoryDisplay {
            main_inventory: InventoryDisplayWidget::new(main_inv_name.clone(), &body),
            other_inventories_tabs: None,
            other_inventory_widgets: Vec::new(),
            last_other_inv_count: 0,
            body,
            main_inventory_data : InventoryDisplayData::new(Vec::new(), main_inv_name),
            other_inventory_data : Vec::new()
        }
    }

    pub fn update(&mut self, gui: &mut GUI, world : &WorldView, main_inventory: InventoryDisplayData, other_inventories: Vec<InventoryDisplayData>) {
        self.main_inventory_data = main_inventory.clone();
        self.other_inventory_data = other_inventories.clone();

        if self.body.showing {
            self.main_inventory.update(gui, world, &main_inventory.items, None);

            if self.last_other_inv_count != other_inventories.len() {
                self.last_other_inv_count = other_inventories.len();
                if let Some(other_inventories) = &mut self.other_inventories_tabs {
                    gui.remove_widget(other_inventories);

                    self.other_inventory_widgets.clear();
                }
                if other_inventories.non_empty() {
                    let other_inv_tabs = TabWidget::new(other_inventories.map(|i| i.name.clone()))
                        .parent(&self.body)
                        .below(&self.main_inventory, 4.ux())
//                        .y(34.ux())
                        .size(40.ux(), 30.ux())
                        .named("Other inventories tab")
                        .apply(gui);

                    for inv in &other_inventories {
                        let parent_tab = other_inv_tabs.tab_named(inv.name.clone());
                        let inv_widget = InventoryDisplayWidget::new(inv.name.clone(), parent_tab)
                            .size(Sizing::match_parent(), Sizing::match_parent())
                            .apply(gui);
                        self.other_inventory_widgets.push(inv_widget);
                    }
                    self.other_inventories_tabs = Some(other_inv_tabs);
                }
            }

            for (i,inv) in other_inventories.iter().enumerate() {
                if let Some(inv_widget) = self.other_inventory_widgets.get_mut(i) {
                    inv_widget.update(gui, world, &inv.items, None);
                } else {
                    warn!("inventory widget enumeration mismatch");
                }
            }
        }
    }
}


struct InventoryDisplayWidget {
    pub body: Widget,
    pub name_display: Widget,
    pub inventory_list: ListWidget<ItemNameDisplay>,
}

//impl WidgetContainer for InventoryDisplayWidget {
//    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
//        (func)(&self.body);
//        (func)(&self.name_display);
//        self.inventory_list.for_all_widgets(func);
//    }
//}
impl DelegateToWidget for InventoryDisplayWidget {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

#[derive(WidgetContainer)]
struct ItemNameDisplay {
    pub name: Widget,
    pub picked_up_indicator: Widget,
}

impl Default for ItemNameDisplay {
    fn default() -> Self {
        ItemNameDisplay {
            name: Widget::text("Test", 14).x(32.px()).y(0.px()),
            picked_up_indicator: Widget::image("ui/hand_icon", Color::white(), 1).size(Sizing::constant(30.px()), Sizing::constant(30.px())).x(0.px()).y(Positioning::centered()),
        }
    }
}

impl InventoryDisplayWidget {
    pub fn new(name: String, parent: &Widget) -> InventoryDisplayWidget {
        let body = Widget::window(Color::greyscale(0.8), 2)
            .width(40.ux())
            .height(30.ux())
            .margin(3.px())
            .parent(parent);
        let name_display = Widget::text(name, 16).parent(&body).x(Positioning::centered());

        let inventory_list_row = Widget::window(Color::greyscale(0.9), 1).size(Sizing::match_parent(), Sizing::constant(30.px()));
        let inventory_list = ListWidget::custom(inventory_list_row, 0.px())
            .border_width(0)
            .color(Color::clear())
            .width(Sizing::match_parent())
            .below(&name_display, 1.ux())
            .parent(&body);
        InventoryDisplayWidget {
            body,
            inventory_list,
            name_display,
        }
    }

    pub fn update(&mut self, gui: &mut GUI, world: &WorldView, items: &Vec<Entity>, selected_item: Option<Entity>) {
        self.body.reapply(gui);
        if self.body.showing {
            self.name_display.reapply(gui);

            self.inventory_list.update(gui, &items, |widget, item| {
                if let Some(ident) = world.data_opt::<IdentityData>(*item) {
                    widget.name.set_text(ident.effective_name());
                } else {
                    widget.name.set_text("unknown entity");
                }
                widget.name.set_showing(true);
            });
        }
    }
}