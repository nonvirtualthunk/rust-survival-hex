use common::prelude::*;
use game::prelude::*;

use archetype::EntityArchetype;
use entities::selectors::EntitySelector;
use entities::common_entities::Taxon;
use std::collections::HashMap;
use multimap::MultiMap;

use game::EntityData;
use entities::skill::Skill;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Ingredient {
    pub ingredient_selector: EntitySelector,
    // i.e. Wood, Mineral, PlantBasedMaterial
    pub amount_required: i32, // how many entities of this kind must be used
}
impl Ingredient {
    pub fn new(selector : EntitySelector, amount : i32) -> Ingredient {
        Ingredient { ingredient_selector : selector, amount_required : amount }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecipeToolUse {
    Required, // must have a tool matching the given selector in order to make at all
    DifficultWithout { ap_increase : Option<i32>, quality_penalty : Option<i32> }, // difficult to make without a tool,
}
impl Default for RecipeToolUse {
    fn default() -> Self {
        RecipeToolUse::Required
    }
}


#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default, Fields)]
pub struct Recipe {
    // i.e. MithrilDoomAxe -> DoomAxe -> Axe, so we can choose the most specific recipe
    pub parent_recipe: Option<Entity>,
    pub ingredients_by_kind: HashMap<Taxon, Ingredient>,
    pub name_from_ingredient : Option<Taxon>,
    pub result: EntityArchetype,
    pub tools_used: Vec<(EntitySelector, RecipeToolUse)>,
    pub skills_used : Vec<SkillUse>,
}

impl EntityData for Recipe {}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkillUse {
    pub skill : Skill,
    pub required_level : Option<i32>,
    pub proportion : Sext
}


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ChildRecipe {
    pub recipe: Entity,
    pub depth: i32,
}

impl Recipe {
    pub fn new(result : EntityArchetype) -> Recipe {
        Recipe {
            parent_recipe : None,
            name_from_ingredient : None,
            ingredients_by_kind : HashMap::new(),
            result,
            skills_used : Vec::new(),
            tools_used: Vec::new(),
        }
    }

    pub fn new_child(result : EntityArchetype, parent : Entity) -> Recipe {
        Recipe {
            parent_recipe : Some(parent),
            name_from_ingredient : None,
            ingredients_by_kind : HashMap::new(),
            result,
            skills_used : Vec::new(),
            tools_used: Vec::new(),
        }
    }

    pub fn name_from<T : Into<Taxon>>(mut self, ingredient_type : T) -> Self {
        self.name_from_ingredient = Some(ingredient_type.into());
        self
    }

    pub fn effective_ingredients_by_kind(&self, view: &WorldView) -> HashMap<Taxon, Ingredient> {
        let mut ret = HashMap::new();
        let mut cur_opt = Some(self);
        while let Some(cur) = cur_opt {
            for (kind, ingredient) in &cur.ingredients_by_kind {
                if ! ret.contains_key(kind) {
                    ret.insert(kind.clone(), ingredient.clone());
                }
            }
            cur_opt = cur.parent_recipe.map(|r| view.data::<Recipe>(r));
        }
        ret
    }

//    pub fn recipe(ingredients_by_kind: HashMap<Taxon, Ingredient>, result: EntityArchetype) -> Recipe {
//        Recipe { parent_recipe: None, ingredients_by_kind, result }
//    }

//    pub fn sub_recipe(parent_recipe: Entity, ingredients_by_kind: HashMap<Taxon, Ingredient>, result: EntityArchetype) -> Recipe {
//        Recipe { parent_recipe: Some(parent_recipe), ingredients_by_kind, result }
//    }
//
    pub fn with_ingredient<T : Into<Taxon>>(mut self, ingredient_type : T, ingredient_selector : EntitySelector, amount : i32) -> Self {
        self.ingredients_by_kind.insert(ingredient_type.into(), Ingredient::new(ingredient_selector, amount));
        self
    }

    pub fn with_tool_use(mut self, which_tool : EntitySelector, tool_use : RecipeToolUse) -> Self {
        self.tools_used.push((which_tool, tool_use));
        self
    }

    pub fn with_skill_use<I : Into<Option<i32>>>(mut self, skill : Skill, proportion: Sext, lvl_required : I) -> Self {
        self.skills_used.push(SkillUse { skill, proportion, required_level : lvl_required.into() });
        self
    }

    pub fn with_single_skill<I : Into<Option<i32>>>(mut self, skill : Skill, lvl_required : I) -> Self {
        self.skills_used.push(SkillUse { skill, proportion : Sext::of(1), required_level : lvl_required.into() });
        self
    }

}


pub struct RecipeCatalogView {
    pub recipes_by_parent: MultiMap<Entity, Entity>,
    root_recipes: Vec<Entity>,
}

impl RecipeCatalogView {
    pub fn of(world: &WorldView) -> RecipeCatalogView {
        let mut recipes_by_parent : MultiMap<Entity, Entity> = MultiMap::new();
        let mut root_recipes = Vec::new();
        for (ent, recipe) in world.entities_with_data::<Recipe>() {
            recipes_by_parent.insert(recipe.parent_recipe.unwrap_or(Entity::sentinel()), *ent);
            if recipe.parent_recipe.is_none() {
                root_recipes.push(*ent);
            }
        }
        RecipeCatalogView {
            recipes_by_parent,
            root_recipes,
        }
    }

    pub fn self_and_child_recipes_of(&self, recipe: Entity) -> Vec<ChildRecipe> {
        let mut children = self.child_recipes_of(recipe);
        children.push(ChildRecipe { recipe, depth : 0 });
        children
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

    pub fn root_recipes(&self) -> &Vec<Entity> {
        &self.root_recipes
    }
}