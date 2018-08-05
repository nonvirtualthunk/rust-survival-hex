use world::Entity;
use world::EntityData;
use world::WorldView;
use core::*;
use std::ops::Deref;
use common::prelude::*;
use common::hex::*;

#[derive(Clone, Default, Debug)]
pub struct TileData {
    pub name : Str,
    pub position: AxialCoord,
    pub move_cost: Oct,
    pub cover: i8,
    pub occupied_by : Option<Entity>,
    pub elevation: i8
}
impl EntityData for TileData {}


pub trait TileStore {
    fn tile (&self, coord : AxialCoord) -> &TileData;
    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData>;
    fn tile_ent(&self, coord: AxialCoord) -> TileEntity;
    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity>;
}

impl TileStore for WorldView {
    fn tile(&self, coord: AxialCoord) -> &TileData {
        let tile_ent = self.entity_by_key(&coord).expect("Tile is expected to exist");
        self.data::<TileData>(tile_ent)
    }

    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData> {
        self.entity_by_key(&coord).map(|e| self.data::<TileData>(e))
    }

    fn tile_ent(&self, coord: AxialCoord) -> TileEntity {
        self.tile_ent_opt(coord).expect("tile is expected to exist")
    }

    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity> {
        if let Some(entity) = self.entity_by_key(&coord) {
            Some(TileEntity { entity, data : self.data::<TileData>(entity) })
        } else {
            None
        }
    }
}

pub struct TileEntity<'a> { pub entity : Entity, pub data : &'a TileData }
impl <'a> Deref for TileEntity<'a> {
    type Target = TileData;

    fn deref(&self) -> &TileData {
        self.data
    }
}