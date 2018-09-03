use world::World;
use std::collections::HashMap;

#[derive(Serialize,Deserialize,Default)]
pub struct Universe {
    pub worlds : Vec<World>
}

#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize,Copy)]
pub struct WorldRef(pub usize);

impl Universe {
    pub fn register_world(&mut self, world : World) -> WorldRef {
        let world_ref = WorldRef(self.worlds.len());
        self.worlds.push(world);
        world_ref
    }

    pub fn world(&mut self, world_ref : WorldRef) -> &mut World {
        &mut self.worlds[world_ref.0]
    }

    pub fn new() -> Universe {
        Universe::default()
    }
}