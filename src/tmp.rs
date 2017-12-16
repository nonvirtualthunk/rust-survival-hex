use std::collections::HashMap;

//#[derive(Clone)]
//struct CharacterData {
//    health: i32
//}
//
//struct Modifier {
//
//}
//
//impl Modifier {
//    fn modify(&self, data: &mut CharacterData, world: &World) {
//        data.health = data.health + world.characters.len() as i32;
//    }
//}
//
//struct Character {
//    id: i32,
//    data: CharacterData,
//    modifiers: Vec<Modifier>
//}
//impl Character {
//    fn eff_data(&self, world : &World) -> CharacterData {
//        let mut new_data = self.data.clone();
//        for modifier in &self.modifiers {
//            modifier.modify(&mut new_data, world);
//        }
//        new_data
//    }
//}
//
//
//struct World {
//    characters: HashMap<i32, Character>
//}
//
//impl World {
//    fn add_character(&mut self, new_char: Character) -> i32 {
//        let id = new_char.id;
//        self.characters.insert(id, new_char);
//        id
//    }
//
//    fn get_modified(&self, i: i32) -> CharacterData {
//        let char1 = self.characters.get(&i).unwrap();
//        let mut new_data = char1.data.clone();
//        for modifier in &char1.modifiers {
//            modifier.modify(&mut new_data, self);
//        }
//        new_data
//    }
//
//    fn get_character(&self, i: i32) -> &Character {
//        self.characters.get(&i).unwrap()
//    }
//
//    fn get_character_mut(&mut self, i: i32) -> &mut Character {
//        self.characters.get_mut(&i).unwrap()
//    }
//}
//
//fn process_world(world: &mut World) {
//    let char1_id = world.add_character(Character {
//        id: 1,
//        data: CharacterData {
//            health: 3
//        },
//        modifiers: vec![]
//    });
//
//    let char2_id = world.add_character(Character {
//        id: 2,
//        data: CharacterData {
//            health: 20
//        },
//        modifiers: vec![]
//    });
//
//    let sum;
//    {
//        let char1 = world.get_modified(char1_id);
//        let char2 = world.get_character(char2_id).eff_data(world);
//        sum = char1.health + char2.health;
//    }
//
//    {
//        let char1 = world.get_character_mut(1);
//        char1.data.health = sum;
//    }
//
//    print!("health sum: {}", world.get_character(1).data.health);
//}
//
//#[test]
//pub fn testbed() {
//    let world = World {
//        characters: HashMap::new()
//    };
//    print!("Everything's fine")
//}