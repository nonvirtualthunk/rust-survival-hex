use common::prelude::*;
use std::collections::HashMap;
use entities::common_entities::IdentityData;
use game::prelude::*;
use game::EntityData;

pub struct Catalog {
    pub entities : HashMap<String, Entity>,
    pub default : Entity
}
impl Catalog {
    pub fn of<T : EntityData>(world_view: &WorldView, default : Entity) -> Catalog {
        let mut ret_map = HashMap::new();
        let all_ident_data = world_view.all_data_of_type::<IdentityData>();
        for (ent, data) in world_view.entities_with_data::<T>() {
            if let Some(ident) = all_ident_data.data_opt(*ent) {
                ret_map.insert(ident.effective_name().to_string(), *ent);
            }
        }

        Catalog {
            entities : ret_map,
            default
        }
    }

    pub fn entity_with_name<'a, S : Into<&'a str>>(&self, name : S) -> Entity {
        *self.entities.get(name.into()).unwrap_or(&self.default)
    }
}