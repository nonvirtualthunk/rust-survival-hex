use std::collections::hash_map;
use std::hash::Hash;
use entities::*;
use common::hex::*;
use core::*;
use events::*;

pub struct World {
    pub characters: EntityContainer<usize, CharacterData>,
    pub tiles: EntityContainer<AxialCoord, TileData>,
    pub event_clock: GameEventClock, // monotonically increasing time clock, incremented by occurrences
    pub events: Vec<GameEventWrapper>,
    pub min_tile: AxialCoord,
    pub max_tile: AxialCoord
}

pub struct WorldView {
    pub characters: hash_map::HashMap<usize, CharacterData>,
    pub tiles: hash_map::HashMap<AxialCoord, TileData>,
    pub event_clock: GameEventClock,
    pub events: Vec<GameEventWrapper>,
    pub min_tile: AxialCoord,
    pub max_tile: AxialCoord
}

impl WorldView {
    pub fn character(&self, cref : CharacterRef) -> &CharacterData {
        self.characters.get(&cref.0).unwrap()
    }

    pub fn tile(&self, coord : AxialCoord) -> &TileData {
        self.tiles.get(&coord).unwrap()
    }
}

impl World {
    fn view_of<KeyType : Eq + Hash + Clone, GameDataType: Clone>(&self, container : &EntityContainer<KeyType, GameDataType>, at_time : GameEventClock) -> hash_map::HashMap<KeyType, GameDataType> {
        let mut ret = hash_map::HashMap::new();
        for (key, ent) in &container.entities {
            let new_key : KeyType = key.clone();
            ret.insert(new_key, ent.data_at_time(self, at_time));
        }
        ret
    }

    pub fn view_at_time(&self, at_time : GameEventClock) -> WorldView {
        WorldView {
            characters : self.view_of(&self.characters, at_time),
            tiles : self.view_of(&self.tiles, at_time),
            event_clock : at_time,
            events : self.events.iter().filter(|&e| e.occurred_at < at_time).cloned().collect::<Vec<GameEventWrapper>>(),
            min_tile : self.min_tile,
            max_tile : self.max_tile
        }
    }
    pub fn add_character_modifier(&mut self, cref: CharacterRef, modifier: Box<Modifier<CharacterData>>)  {
        let event_clock = self.event_clock;
        let character = self.character_mut(cref);
        character.add_modifier(ModifierContainer {
            modifier,
            applied_at : event_clock
        });
    }

    pub fn character(&self, cref: CharacterRef) -> CharacterData {
        self.characters.entities.get(&cref.0).unwrap().data(self)
    }

    pub fn character_at_time(&self, cref: CharacterRef, at_time : GameEventClock) -> CharacterData {
        self.characters.entities.get(&cref.0).unwrap().data_at_time(self, at_time)
    }

    pub fn character_mut(&mut self, cref: CharacterRef) -> &mut Character {
        self.characters.entities.get_mut(&cref.0).unwrap_or(&mut self.characters.sentinel)
    }

    pub fn characters<'a>(&'a self) -> hash_map::Values<usize, Character> {
        self.characters.entities.values()
    }

    pub fn add_character(&mut self, new_char: Character) -> CharacterRef {
        let id = new_char.id;
        self.characters.entities.insert(id, new_char);
        CharacterRef(id)
    }

    pub fn add_tile(&mut self, tile : Tile) {
        let pos = tile.intern_data.position;
        self.min_tile = AxialCoord::new(self.min_tile.q.min(pos.q), self.min_tile.r.min(pos.r));
        self.max_tile = AxialCoord::new(self.max_tile.q.max(pos.q), self.max_tile.r.max(pos.r));
        self.tiles.entities.insert(pos, tile);
    }

    pub fn tile(&self, coord: &AxialCoord) -> TileData {
        match self.tiles.entities.get(coord) {
            Some(value) => value.data(self),
            None => self.tiles.sentinel.data(self)
        }
    }

    pub fn tile_mut(&mut self, coord: &AxialCoord) -> &mut Tile {
        self.tiles.entities.get_mut(coord).unwrap_or(&mut self.tiles.sentinel)
    }

    pub fn tiles<'a>(&'a self) -> &'a hash_map::HashMap<AxialCoord, Tile> {
        & self.tiles.entities
    }

    pub fn add_event(&mut self, event : GameEvent) {
        self.events.push(GameEventWrapper { occurred_at : self.event_clock, data : event });
        self.event_clock += 1;
    }

    pub fn new() -> World {
        World {
            characters: EntityContainer::new(Character {
                id: 0,
                intern_data: CharacterData {
                    ..Default::default()
                },
                modifiers: vec![]
            }),
            tiles: EntityContainer::new(Tile {
                id: 0,
                intern_data: TileData {
                    name: "void",
                    position: AxialCoord::new(0, 0),
                    move_cost: 10000
                },
                modifiers: vec![]
            }),
            event_clock: 1,
            events: vec![GameEventWrapper { occurred_at : 0, data : GameEvent::WorldStart }],
            min_tile: AxialCoord::new(0,0),
            max_tile: AxialCoord::new(0,0)
        }
    }
}