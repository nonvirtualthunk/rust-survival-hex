use common::prelude::*;
use game::prelude::*;

use archetype::EntityArchetype;
use entities::selectors::EntitySelector;
use entities::common_entities::Taxon;
use std::collections::HashMap;
use multimap::MultiMap;

use game::EntityData;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Ingredient {
    pub ingredient_selector: EntitySelector,
    // i.e. Wood, Mineral, PlantBasedMaterial
    pub amount_required: i32, // how many entities of this kind must be used
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default, Fields)]
pub struct Recipe {
    // i.e. MithrilDoomAxe -> DoomAxe -> Axe, so we can choose the most specific recipe
    pub parent_recipe: Option<Entity>,
    pub ingredients_by_kind: HashMap<Taxon, Ingredient>,
    pub result: EntityArchetype,
}

impl EntityData for Recipe {}


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ChildRecipe {
    recipe: Entity,
    depth: i32,
}

impl Recipe {
    pub fn recipe(ingredients_by_kind: HashMap<Taxon, Ingredient>, result: EntityArchetype) -> Recipe {
        Recipe { parent_recipe: None, ingredients_by_kind, result }
    }

    pub fn sub_recipe(parent_recipe: Entity, ingredients_by_kind: HashMap<Taxon, Ingredient>, result: EntityArchetype) -> Recipe {
        Recipe { parent_recipe: Some(parent_recipe), ingredients_by_kind, result }
    }
}


pub struct RecipeCatalogView {
    recipes_by_parent: MultiMap<Entity, Entity>,
}

impl RecipeCatalogView {
    pub fn from(world: &WorldView) -> RecipeCatalogView {
        let mut recipes_by_parent : MultiMap<Entity, Entity> = MultiMap::new();
        for (ent, recipe) in world.entities_with_data::<Recipe>() {
            recipes_by_parent.insert(recipe.parent_recipe.unwrap_or(Entity::sentinel()), *ent);
        }
        RecipeCatalogView {
            recipes_by_parent
        }
    }

    pub fn child_recipes_of(&self, recipe: Entity) -> Vec<ChildRecipe> {
        let mut ret = Vec::new();
        if let Some(children) = self.recipes_by_parent.get_vec(&recipe) {
            for child in children {
                let child_children = self.child_recipes_of(*child);
                ret.push(ChildRecipe { recipe: *child, depth: 1 });
                ret.extend(child_children.into_iter().map(|cr| ChildRecipe { recipe: cr.recipe, depth: cr.depth + 1 }))
            }
        }

        ret
    }
}