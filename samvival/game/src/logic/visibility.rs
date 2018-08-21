use common::prelude::*;
use common::hex;
use prelude::*;

use entities::Visibility;
use entities::VisibilityData;
use entities::character::CharacterData;
use entities::common::*;
use game::EntityData;


//pub struct

#[derive(Clone,Debug,Default)]
pub struct VisibilityComputor {

}
impl EntityData for VisibilityComputor {}


impl VisibilityComputor {
    pub fn new() -> VisibilityComputor {
        VisibilityComputor {

        }
    }


    pub fn register(world: &mut World) {
        world.register::<VisibilityComputor>();

        world.add_callback(|world,event_w| {
            match event_w.event {
                GameEvent::WorldStart => {

                },
                GameEvent::Move { character, from, to, .. } => {
                    let world_view = world.view();
                    let faction = world_view.character(character).faction;
                    let computor = world.world_data_mut::<VisibilityComputor>();
                    let vis = computor.recompute_visibility(world_view, faction, None);

//                    world.modify()
                },
                _ => ()
            }
        });
    }

    pub fn recompute_visibility(&self, world : &WorldView, faction : Entity, moved_entities_ : Option<Vec<Entity>>) -> Visibility {
//        let current_visibility = world.world_data::<VisibilityData>().visibility_by_faction.get(faction).unwrap_or_else(|| Visibility::new());

        // for the moment, recompute from scratch every time

        let mut visibility = Visibility::new();

        for (ent,cdata) in world.entities_with_data::<CharacterData>() {
            if cdata.faction == faction {
                self.compute_character_visibility(world, *ent, cdata, &mut visibility);
            }
        }

        visibility
    }


    fn compute_character_visibility(&self, world : &WorldView, ent : Entity, cdata : &CharacterData, visibility : &mut Visibility) {
        let center : AxialCoord = world.data::<PositionData>(ent).hex;
        let center_cube = center.as_cube_coord();

        visibility.visible_hexes.insert(center);
        for r in 1 .. 6 {
            let mut cur = center_cube + hex::CUBE_DELTAS[4] * r;

            for j in 0 .. r {
                visibility.visible_hexes.insert(cur.as_axial_coord());
                cur = cur + hex::CUBE_DELTAS[j as usize];
            }
        }
    }
}