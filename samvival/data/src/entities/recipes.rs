use common::prelude::*;
use game::prelude::*;

use archetype::EntityArchetype;
use entities::selectors::EntitySelector;
use entities::common_entities::Taxon;
use std::collections::HashMap;
use multimap::MultiMap;


#[derive(PartialEq,Hash,Serialize,Deserialize,Debug,Clone)]
pub struct Ingredient {
    pub part_kind : Taxon, // i.e. Shaft, Haft, Fletching
    pub ingredient_selector : EntitySelector, // i.e. Wood, Mineral, PlantBasedMaterial
    pub amount_required : i32, // how many entities of this kind must be used
}

#[derive(PartialEq,Hash,Serialize,Deserialize,Debug,Clone,Fields)]
pub struct Recipe {
    parent_recipe : Option<Entity>, // i.e. MithrilDoomAxe -> DoomAxe -> Axe, so we can choose the most specific recipe
    ingredients : Vec<Ingredient>,
    result : EntityArchetype,
}


#[derive(Clone,Copy,PartialEq,Debug)]
pub struct ChildRecipe {
    recipe : Entity,
    depth : i32
}

impl Recipe {
    pub fn recipe(ingredients : Vec<Ingredient>, result : EntityArchetype) -> Recipe {
        Recipe { parent_recipe : None, ingredients, result }
    }

    pub fn sub_recipe(parent_recipe : Entity, ingredients : Vec<Ingredient>, result : EntityArchetype) -> Recipe {
        Recipe { parent_recipe, ingredients, result}
    }
}


pub struct RecipeLibrary {
    recipes_by_parent : MultiMap<Entity, Entity>,
}

impl RecipeLibrary {
    pub fn new(world : &WorldView) -> RecipeLibrary {
        let mut recipes_by_parent = MultiMap::new();
        for (ent,recipe) in world.entities_with_data::<Recipe>() {
            recipes_by_parent.insert(recipe.parent_recipe.cloned().unwrap_or(Entity::sentinel()), ent);
        }
        RecipeLibrary {
            recipes_by_parent
        }
    }

    pub fn child_recipes_of(&self, recipe : Entity) -> Vec<ChildRecipe> {
        let mut ret = Vec::new();
        if let Some(children) = self.recipes_by_parent.get_vec(&recipe) {
            for child in children {
                let child_children = child_recipes_of(*child);
                ret.append(ChildRecipe { recipe : *child, depth : 1 });
                ret.extend(child_children.into_iter().map(|cr| ChildRecipe { recipe : cr.recipe , depth : cr.depth + 1 }))
            }
        }

        ret
    }
}