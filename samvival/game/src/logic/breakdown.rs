use common::prelude::*;
use std::ops;
use game::EntityData;
use game::reflect::*;
use game::modifiers::FieldLogs;
use common::reflect::Field;

#[derive(Default)]
pub struct Breakdown<T: Default> {
    pub total: T,
    pub components: Vec<(String, String)>,
}

impl<T: Default> Breakdown<T> where T: ops::Add<Output=T> + Clone + ToStringWithSign {
    pub fn add_field<S1: Into<String>, E: EntityData, U: ToStringWithSign + Clone>(&mut self, net_value: T, logs: &FieldLogs<E>, field: &'static Field<E, U>, descriptor: S1) {
        self.total = self.total.clone() + net_value;
        let base_value = (field.getter)(&logs.base_value);
        let base_value_str = base_value.to_string_with_sign();
        self.components.push((base_value_str, format!("base {}", descriptor.into())));
        for field_mod in logs.modifications_for(field) {
            let mut modification_str = field_mod.modification.to_string();
            modification_str.retain(|c| !c.is_whitespace());
            self.components.push((modification_str, field_mod.description.clone().unwrap_or_else(||String::from(""))))
        }
    }

    pub fn add<S1: Into<String>>(&mut self, value: T, descriptor: S1) {
        self.total = self.total.clone() + value.clone();
        self.components.push((value.to_string_with_sign(), descriptor.into()));
    }
    pub fn new() -> Breakdown<T> {
        Breakdown::default()
    }
}