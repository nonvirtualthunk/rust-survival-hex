use common::prelude::*;
use gui::prelude::*;
use gui::ListWidget;
use common::Color;
use game::prelude::*;
use game::entities::IdentityData;
use game::entities::EquipmentData;
use gui::TabWidget;
use state::ControlContext;
use control_events::*;
use std::collections::HashSet;

// we actually want two things here, one to just display an inventory, and another to display multiple inventories together
// to allow moving items back and forth between them.


pub struct InventoryDisplay {
    body: Widget,
    main_inventories: InventoryDisplayWidget,
    other_inventories: InventoryDisplayWidget,
    selected_item : Option<Entity>,
}

impl DelegateToWidget for InventoryDisplay {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

#[derive(Clone, PartialEq)]
pub struct InventoryDisplayData {
    pub name: String,
    pub items: Vec<Entity>,
    pub from_entities: Vec<Entity>,
    pub equippable: bool,
    pub size: Option<u32>
}
impl InventoryDisplayData {
    pub fn new<S : Into<String>>(items : Vec<Entity>, name : S, from_entities : Vec<Entity>, equippable : bool, size : Option<u32>) -> InventoryDisplayData {
        InventoryDisplayData {
            items,
            name : name.into(),
            from_entities,
            equippable,
            size
        }
    }
}

impl InventoryDisplay {
    pub fn new(main_inv_name: String, parent: &Widget) -> InventoryDisplay {
        let body = Widget::div().centered().named("Inventory display parent div").parent(parent);
        let main_inventories = InventoryDisplayWidget::new(&body);
        let other_inventories = InventoryDisplayWidget::new(&body)
            .below(main_inventories.as_widget_immut(), 4.ux());

        InventoryDisplay {
            other_inventories,
            main_inventories,
            body,
            selected_item : None,
        }
    }


    fn all_items(&self) -> impl Iterator<Item=&Entity> {
        self.main_inventories.inventories.iter().flat_map(|idd| idd.items.iter())
            .chain(self.other_inventories.inventories.iter().flat_map(|idd| idd.items.iter()))
    }

    pub fn update(&mut self, gui: &mut GUI, world : &WorldView, main_inventories: Vec<InventoryDisplayData>, other_inventories: Vec<InventoryDisplayData>, control : &mut ControlContext) {

        if let Some(selected) = self.selected_item {
            if ! self.all_items().any(|item| item == &selected) {
                self.selected_item = None;
                info!("Clearing selected inventory item since it is no longer part of the inventories");
            }
        }

        for event in gui.events_for(&self.body) {
            if let Some((item_selected, origin)) = event.as_custom_event::<InventoryItemSelected>() {
                let targeted_inventories = if origin == self.main_inventories.id() { &self.main_inventories } else { &self.other_inventories };

                if let Some(targeted_item) = targeted_inventories.inventories.get(item_selected.inventory_index).and_then(|idd| idd.items.get(item_selected.item_index)) {
                    if self.selected_item.as_ref() == Some(targeted_item) {
                        self.selected_item = None;
                    } else {
                        self.selected_item = Some(*targeted_item);
                    }
                }
            }
            if let Some((InventoryItemToggleEquip { item }, origin)) = event.as_custom_event::<InventoryItemToggleEquip>() {
                let entities = self.main_inventories.active_inventory_data(gui).from_entities.clone();
                if entities.len() != 1 {
                    error!("Our inventory setup kind of assumes one from-entity for equipping");
                }
                if let Some(entity) = entities.first() {
                    control.trigger_event(ControlEvents::EquipItemRequested { item, equip_on : *entity });
                }
            }
        }

        let mut transfer_to : Option<&InventoryDisplayWidget> = None;
        for event in gui.events_for(self.main_inventories.as_widget_immut()) {
            if let UIEvent::MouseRelease { pos , .. } = event {
                transfer_to = Some(&self.main_inventories);
            }
        }
        for event in gui.events_for(self.other_inventories.as_widget_immut()) {
            if let UIEvent::MouseRelease { pos , .. } = event {
                transfer_to = Some(&self.other_inventories);
            }
        }

        if let Some(transfer_to) = transfer_to {
            if let Some(selected) = self.selected_item {
                if transfer_to.inventories.any_match(|idd| idd.items.contains(&selected)) {
                    info!("not going to transfer an item from an inventory to itself")
                } else {
                    let transfer_from = if transfer_to == &self.main_inventories { &self.other_inventories } else { &self.main_inventories };
                    let from_entities = transfer_from.inventories.iter().find(|idd| idd.items.contains(&selected)).map(|idd| idd.from_entities.clone()).unwrap_or(Vec::new());
                    let to_entities = transfer_to.active_inventory_data(gui).from_entities.clone();
                    if from_entities.non_empty() && to_entities.non_empty() {
                        control.trigger_event(ControlEvents::ItemTransferRequested { item : selected, from : from_entities, to: to_entities });
                        self.selected_item = None;
                    } else {
                        warn!("From/to entities weren't valid for inventory transfer: {:?}, {:?}", from_entities, to_entities);
                    }
                }
            }
        }

        if self.body.showing {
            self.main_inventories.update(gui, world, &main_inventories, self.selected_item);

            self.other_inventories.update(gui, world, &other_inventories, self.selected_item);
            if other_inventories.is_empty() {
                self.other_inventories.set_showing(false).reapply(gui);
            } else {
                self.other_inventories.set_showing(true).reapply(gui);
            }
        }
    }

    pub fn selected_item(&self) -> Option<Entity> {
        self.selected_item
    }
}


struct InventoryDisplayWidget {
    pub body: TabWidget,
    pub inventory_lists: Vec<ListWidget<ItemNameDisplay>>,
    pub inventories : Vec<InventoryDisplayData>,
    placeholder_data : InventoryDisplayData,
    last_selected : Option<Entity>
}
impl PartialEq<InventoryDisplayWidget> for InventoryDisplayWidget {
    fn eq(&self, other: &'_ InventoryDisplayWidget) -> bool {
        self.id() == other.id()
    }
}
impl DelegateToWidget for InventoryDisplayWidget {
    fn as_widget(&mut self) -> &mut Widget { self.body.as_widget() }
    fn as_widget_immut(&self) -> &Widget { self.body.as_widget_immut() }
}

#[derive(WidgetContainer)]
struct ItemNameDisplay {
    pub name: Widget,
    pub picked_up_indicator: Widget,
    pub equip_button: Button
}

#[derive(Clone)]
pub struct InventoryItemSelected { inventory_index : usize, item_index : usize }
#[derive(Clone)]
pub struct InventoryItemToggleEquip { item : Entity }

impl Default for ItemNameDisplay {
    fn default() -> Self {
        ItemNameDisplay {
            name: Widget::text("Test", 14).x(32.px()).y(Positioning::centered()),
            picked_up_indicator: Widget::image("ui/hand_icon", Color::white(), 1)
                .size(30.px(), 30.px())
                .y(Positioning::centered())
                .border_sides(BorderSides::one_side(Alignment::Right))
                .showing(false),
            equip_button: Button::new("test").showing(false).y(Positioning::centered()).align_right().height(30.px())
                .border_width(1).border_sides(BorderSides::one_side(Alignment::Left))
        }
    }
}

impl InventoryDisplayWidget {
    pub fn new(parent: &Widget) -> InventoryDisplayWidget {
        let body = TabWidget::new(Vec::<String>::new())
            .width(40.ux())
            .height(30.ux())
            .margin(3.px())
            .parent(parent)
            .and_consume(EventConsumption::mouse_events());

        InventoryDisplayWidget {
            body,
            inventory_lists : Vec::new(),
            inventories : Vec::new(),
            placeholder_data : InventoryDisplayData {
                items : Vec::new(),
                name : String::from("sentinel"),
                from_entities : Vec::new(),
                equippable: false,
                size: None,
            },
            last_selected : None
        }
    }

    pub fn active_inventory_data(&self, gui : &GUI) -> &InventoryDisplayData {
        self.body.active_tab(gui).and_then(|at| self.inventories.get(at as usize)).unwrap_or(&self.placeholder_data)
    }


    pub fn update(&mut self, gui: &mut GUI, world: &WorldView, inventories: &Vec<InventoryDisplayData>, selected_item: Option<Entity>) {
        self.body.reapply(gui);
        let new_inventory_names = inventories.map(|idd| idd.name.clone());
        let self_id = self.id();
        if self.inventories.map(|idd| idd.name.clone()) != new_inventory_names {
            while self.inventory_lists.len() > inventories.len() {
                if let Some(mut inv) = self.inventory_lists.pop() {
                    gui.remove_widget(&mut inv);
                }
            }

            self.body.set_tabs(new_inventory_names);

            while self.inventory_lists.len() < inventories.len() {
                let inventory_index = self.inventory_lists.len();
                let tab = self.body.tab_at_index(inventory_index);
                let inventory_list_row = Widget::window(Color::greyscale(0.9), 1).size(Sizing::match_parent(), Sizing::constant(32.px()));
                let mut inventory_list = ListWidget::custom(inventory_list_row, 4.px())
                    .border_width(0)
                    .color(Color::clear())
                    .width(Sizing::match_parent())
                    .parent(tab);
                inventory_list.add_callback(move |ctxt : &mut WidgetContext, event : &UIEvent| {
                   if let UIEvent::WidgetEvent { event : WidgetEvent::ListItemClicked(index, button), .. } = event {
                       ctxt.trigger_event(UIEvent::custom_event(InventoryItemSelected { inventory_index , item_index : *index }, self_id));
                   }
                });
                self.inventory_lists.push(inventory_list);
            }
            self.body.reapply_all(gui);
        }

        let inv_clone = inventories.clone();
        if self.last_selected != selected_item || self.inventories != inv_clone {
            self.last_selected = selected_item;
            self.inventories = inv_clone;

            for (inventory_list, inventory) in self.inventory_lists.iter_mut().zip(inventories.iter()) {
                let equippable = inventory.equippable;

                let all_equipped_items : HashSet<Entity> = inventory.from_entities.iter()
                    .flat_map(|ent| world.data_opt::<EquipmentData>(*ent).map(|eq| eq.equipped.clone()).unwrap_or(Vec::new()))
                    .collect();

                let empty_slots = (0 .. (inventory.size.unwrap_or(0) as i32 - inventory.items.len() as i32).as_u32_or_0()).map(|i| None);
                let item_or_slot = inventory.items.iter().map(|i| Some(i)).chain(empty_slots).collect_vec();
                inventory_list.update(gui, &item_or_slot, |widget, item_or_slot| {
                    if let Some(item) = item_or_slot {
                        let item = *item;
                        if let Some(ident) = world.data_opt::<IdentityData>(*item) {
                            widget.name.set_showing(true).set_text(ident.effective_name());
                        } else {
                            widget.name.set_showing(true).set_text("unknown entity");
                        }
                        widget.picked_up_indicator.set_showing(Some(*item) == selected_item);
                        if equippable {
                            let text = if all_equipped_items.contains(item) { "Unequip" } else { "Equip" };

                            let item_copy = *item;
                            widget.equip_button.set_showing(true).set_text(text).clear_callbacks().add_callback(move |ctxt : &mut WidgetContext, event : &UIEvent| {
                                if let UIEvent::WidgetEvent { event : WidgetEvent::ButtonClicked(_), .. } = event {
                                    println!("Executing callback");
                                    ctxt.trigger_event(UIEvent::custom_event(InventoryItemToggleEquip { item : item_copy }, self_id));
                                }
                            });
                        } else {
                            widget.equip_button.set_showing(false);
                        }
                    } else {
                        widget.name.set_showing(false);
                        widget.picked_up_indicator.set_showing(false);
                        widget.equip_button.set_showing(false);
                    }
                });
            }
        }
    }
}