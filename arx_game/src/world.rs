use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::collections::hash_map;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::hash::Hash;
use entities::*;
use entity_base::*;
use common::hex::*;
use core::*;
use events::*;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::sync::Mutex;
use lazy_static;
use common::datastructures::PerfectHashMap;
use common::datastructures::PerfectHashable;
use std::collections::hash_map::RandomState;
use pathfinding::astar;
use noisy_float::prelude::*;


//lazy_static! {
///// world views, organized by the world id they are views of. Only one per world id because they
///// are all supposed to be kept up to date (snapshot world views are not kept here) and therefore
///// they are all going to be identical
//    static ref WORLD_VIEWS : Mutex<hash_map::HashMap<usize,Arc<Mutex<WorldView>>>> = {
//        Mutex::new(hash_map::HashMap::new())
//    };
//}

pub struct EntityContainer<KeyType : Eq + Hash + PerfectHashable, GameDataType: Clone> {
    pub entities: PerfectHashMap<KeyType, Entity<GameDataType>>
}

impl<KeyType : Eq + Hash + PerfectHashable, GameDataType: Clone> EntityContainer<KeyType, GameDataType> {
    pub fn new(sentinel: Entity<GameDataType>) -> EntityContainer<KeyType, GameDataType> {
        EntityContainer {
            //            modifiers: vec![],
            entities: PerfectHashMap::new(sentinel)
        }
    }
}


pub struct World {
    pub id: usize,
    pub characters: EntityContainer<CharacterRef, CharacterData>,
    pub tiles: EntityContainer<AxialCoord, TileData>,
    pub items: EntityContainer<ItemRef, ItemData>,
    pub factions: EntityContainer<FactionRef, FactionData>,
    pub event_clock: GameEventClock, // monotonically increasing time clock, incremented by occurrences
    pub events: Vec<GameEventWrapper>,
    pub min_tile: AxialCoord,
    pub max_tile: AxialCoord,
    raw_data: WorldData,
    pub modifiers : Vec<ModifierContainer<WorldData>>,

    view : UnsafeCell<WorldView>
}

#[derive(Default,Clone,Copy)]
pub struct WorldData {
    pub turn_number: u32
}


#[derive(Default)]
pub struct WorldView {
    pub from_id: usize,
    pub keep_up_to_date: bool,
    pub characters: PerfectHashMap<CharacterRef, CharacterData>,
    pub tiles: PerfectHashMap<AxialCoord, TileData>,
    pub items: PerfectHashMap<ItemRef, ItemData>,
    pub factions: PerfectHashMap<FactionRef, FactionData>,
    pub event_clock: GameEventClock,
    pub events: Vec<GameEventWrapper>,
    pub min_tile: AxialCoord,
    pub max_tile: AxialCoord,
    pub data : WorldData
}

impl WorldView {
    pub fn character(&self, cref : CharacterRef) -> &CharacterData {
        self.characters.get(&cref)
    }

    pub fn character_mut(&mut self, cref : CharacterRef) -> &mut CharacterData {
        self.characters.get_mut(&cref)
    }

    pub fn faction(&self, cref : FactionRef) -> &FactionData {
        self.factions.get(&cref)
    }

    pub fn tile(&self, coord : AxialCoord) -> &TileData {
        self.tiles.get(&coord)
    }

    pub fn item(&self, iref : ItemRef) -> &ItemData {
        self.items.get(&iref)
    }

    fn catch_up_entities<KeyType : Eq + Hash + PerfectHashable + Clone, GameDataType: Clone>(
        map : &mut PerfectHashMap<KeyType, GameDataType>,
        ent_container: &EntityContainer<KeyType, GameDataType>,
        world : &World,) {
        let new_clock = world.event_clock;
        let cloned_keys : Vec<KeyType> = map.keys.iter().cloned().collect::<Vec<KeyType>>();
        for key in &cloned_keys {
            let mut cur = map.get_mut(key);
            let modifiers : &Vec<ModifierContainer<GameDataType>> = &ent_container.entities.get(key).modifiers;
            for modifier in modifiers {
                modifier.modifier.apply(cur, world, new_clock);
            }
        }
    }

    pub fn catch_up_to(&mut self, world: &World) {
        if self.event_clock < world.event_clock {
            // TODO: Handle entities created since last updated
            WorldView::catch_up_entities(&mut self.characters, &world.characters, world);
            WorldView::catch_up_entities(&mut self.tiles, &world.tiles, world);
            WorldView::catch_up_entities(&mut self.items, &world.items, world);
            WorldView::catch_up_entities(&mut self.factions, &world.factions, world);

            self.event_clock = world.event_clock;
            self.events = world.events.clone();
            self.data = world.data().clone();
        }
    }
}

//impl Default for WorldView {
//    fn default() -> Self {
//        WorldView {
//            characters : PerfectHashMap::new(CharacterData::default())
//        }
//    }
//}

impl World {
    fn view_of<KeyType : Eq + Hash + PerfectHashable + Clone, GameDataType: Clone>(&self, container : &EntityContainer<KeyType, GameDataType>, at_time : GameEventClock) -> PerfectHashMap<KeyType, GameDataType> {
        let existing_sentinel : &Entity<GameDataType> = &container.entities.sentinel;
        let cloned_sentinel = existing_sentinel.intern_data.clone();
        let mut ret = PerfectHashMap::new(cloned_sentinel);
        for (key, ent) in &container.entities {
            let new_key : KeyType = key.clone();
            ret.put(new_key, ent.data_at_time(self, at_time));
        }
        ret
    }

    /// Returns a view of this world that will be kept continuously up to date
    pub fn view<'a, 'b>(&'a self) -> &'b WorldView {
        unsafe { &*self.view.get() }
    }

    fn mut_view(&self) -> &mut WorldView {
        unsafe { &mut *self.view.get() }
    }

    /// Returns a snapshot view of this world at a given time point
    pub fn view_at_time(&self, at_time : GameEventClock) -> WorldView {
        let mut data_view = self.raw_data.clone();

        for modifier in &self.modifiers {
            if modifier.applied_at <= at_time {
                modifier.modifier.apply(&mut data_view, self, at_time);
            }
        }

        WorldView {
            keep_up_to_date : false,
            from_id : self.id,
            characters : self.view_of(&self.characters, at_time),
            tiles : self.view_of(&self.tiles, at_time),
            items : self.view_of(&self.items, at_time),
            factions : self.view_of(&self.factions, at_time),
            event_clock : at_time,
            events : self.events.iter().filter(|&e| e.occurred_at < at_time).cloned().collect::<Vec<GameEventWrapper>>(),
            min_tile : self.min_tile,
            max_tile : self.max_tile,
            data : data_view
        }
    }

    pub fn data(&self) -> WorldData {
        let mut data_view = self.raw_data.clone();

        for modifier in &self.modifiers {
            modifier.modifier.apply(&mut data_view, self, self.event_clock);
        }
        data_view
    }

    pub fn add_modifier(&mut self, modifier : Box<Modifier<WorldData>>) {
        {
            self.modifiers.push(ModifierContainer {
                modifier,
                applied_at : self.event_clock
            })
        }
        self.mut_view().data = self.data();
    }

    pub fn modify<F : Fn(&mut WorldData) + 'static>(&mut self, modifier_func : F) {
        {
            self.modifiers.push(ModifierContainer {
                modifier : Box::new(GenericModifier::new(move |wd: &mut WorldData, _ : &World, _ : GameEventClock| modifier_func(wd))),
                applied_at : self.event_clock
            })
        }
        self.mut_view().data = self.data();
    }

    pub fn add_character_modifier(&mut self, cref: CharacterRef, modifier: Box<Modifier<CharacterData>>)  {
        {
            let event_clock = self.event_clock;
            let character = self.characters.entities.get_mut(&cref);
            character.add_modifier(ModifierContainer {
                modifier,
                applied_at : event_clock
            });
        }
        self.mut_view().characters.put(cref, self.character(cref));
    }


    pub fn add_item_modifier(&mut self, cref: ItemRef, modifier: Box<Modifier<ItemData>>)  {
        {
            let event_clock = self.event_clock;
            let item = self.items.entities.get_mut(&cref);
            item.add_modifier(ModifierContainer {
                modifier,
                applied_at : event_clock
            });
        }
        self.mut_view().items.put(cref, self.item(cref));
    }


    pub fn character(&self, cref: CharacterRef) -> CharacterData {
        self.characters.entities.get(&cref).data(self)
    }

    pub fn character_at_time(&self, cref: CharacterRef, at_time : GameEventClock) -> CharacterData {
        self.characters.entities.get(&cref).data_at_time(self, at_time)
    }

    pub fn add_faction(&mut self, new_faction: FactionData) -> FactionRef {
        let faction = Faction::new(new_faction.clone());
        let id = faction.id;
        self.factions.entities.put(FactionRef(id), faction);
        self.mut_view().factions.put(FactionRef(id), new_faction);
        FactionRef(id)
    }

    pub fn add_character(&mut self, char_data: CharacterData) -> CharacterRef {
        let character = Character::new(char_data.clone());
        let id = character.id;
        self.characters.entities.put(CharacterRef(id), character);
        self.mut_view().characters.put(CharacterRef(id), char_data);

        CharacterRef(id)
    }

    pub fn add_tile(&mut self, tile_data : TileData) {
        let tile = Tile::new(tile_data.clone());
        let pos = tile_data.position;
        self.min_tile = AxialCoord::new(self.min_tile.q.min(pos.q), self.min_tile.r.min(pos.r));
        self.max_tile = AxialCoord::new(self.max_tile.q.max(pos.q), self.max_tile.r.max(pos.r));
        self.tiles.entities.put(pos, tile);
        self.mut_view().tiles.put(pos, tile_data);
    }

    pub fn add_item(&mut self, item_data : ItemData) -> ItemRef {
        let item = Item::new(item_data.clone());
        let id = item.id;
        self.items.entities.put(ItemRef(id), item);
        self.mut_view().items.put(ItemRef(id), item_data);

        ItemRef(id)
    }

    pub fn item(&self, iref: ItemRef) -> ItemData {
        self.items.entities.get(&iref).data(self)
    }

    pub fn item_at_time(&self, iref: ItemRef, at_time : GameEventClock) -> ItemData {
        self.items.entities.get(&iref).data_at_time(self, at_time)
    }

    pub fn tile(&self, coord: &AxialCoord) -> TileData {
        self.tiles.entities.get(coord).data(self)
    }

    pub fn tile_mut(&mut self, coord: &AxialCoord) -> &mut Tile {
        self.tiles.entities.get_mut(coord)
    }

    pub fn add_event(&mut self, event : GameEvent) {
        self.events.push(GameEventWrapper { occurred_at : self.event_clock, data : event });
        self.event_clock += 1;

//        println!("EVT {:?}", event);
    }

    pub fn event_at(&self, event_clock : GameEventClock) -> Option<GameEvent> {
        self.events.get(event_clock as usize).map(|e| e.data)
    }

    pub fn new() -> World {
        let world = World {
            id: WORLD_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
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
                    move_cost: 10000,
                    cover: 0.0
                },
                modifiers: vec![]
            }),
            items: EntityContainer::new(Item {
                id: 0,
                intern_data: ItemData {
                    position : None,
                    held_by : None,
                    primary_attack : None,
                    secondary_attack : None
                },
                modifiers: vec![]
            }),
            factions: EntityContainer::new(Faction {
               id: 0,
                intern_data: FactionData {
                    name : String::from("No Faction"),
                    color : [0.5,0.5,0.5,0.5]
                },
                modifiers: vec![]
            }),
            event_clock: 1,
            events: vec![GameEventWrapper { occurred_at : 0, data : GameEvent::WorldStart }],
            min_tile: AxialCoord::new(0,0),
            max_tile: AxialCoord::new(0,0),
            raw_data: WorldData::default(),
            modifiers : vec![],
            view : UnsafeCell::new(WorldView::default())
        };

        {
            let view = world.mut_view();
            view.from_id = world.id;
            view.events = world.events.clone();
        }

        world
    }
}

impl <'a, 'b> Into<&'b WorldView> for &'a World {
    fn into(self) -> &'b WorldView {
        self.view()
    }
}

impl WorldView {
    pub fn character_at(&self, coord : AxialCoord) -> Option<(CharacterRef, &CharacterData)> {
        for (cref, cdata) in &self.characters {
            if cdata.position == coord && cdata.is_alive() {
                return Some((*cref, cdata));
            }
        }
        None
    }

    pub fn path(&self, from: AxialCoord, to: AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
        astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(1.0))), |c| c.distance(&to), |c| *c == to)
    }

    pub fn path_any_v(&self, from: AxialCoord, to: &Vec<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
        let mut set = HashSet::new();
        set.extend(to.iter());
        self.path_any(from, &set, heuristical_center)
    }

    pub fn path_any(&self, from: AxialCoord, to: &HashSet<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
        astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(1.0))), |c| c.distance(&heuristical_center), |c| to.contains(c))
    }
}