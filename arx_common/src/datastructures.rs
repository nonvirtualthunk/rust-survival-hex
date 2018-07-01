use std::hash::Hash;
use std::hash::Hasher;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::collections::hash_map::DefaultHasher;
use std::marker::PhantomData;
use std::iter::Iterator;

pub trait PerfectHashable {
    fn hash(&self) -> usize;
}

#[derive(Default)]
pub struct PerfectHashMap<K : PerfectHashable, V> {
    pub keys : Vec<K>,
    pub values : Vec<Option<V>>,
    pub sentinel : V,
//    hasher_builder : S,
    _phantom_marker : PhantomData<K>
}

impl <K : PerfectHashable, V> PerfectHashMap<K, V> {
    pub fn new (sentinel : V) -> PerfectHashMap<K, V> {
        PerfectHashMap {
            keys : vec![],
            values : vec![],
            sentinel,
//            hasher_builder : RandomState::new(),
            _phantom_marker : PhantomData
        }
    }

    pub fn get(&self, k :&K) -> &V {
        let idx = self.hash(k);

        if idx >= self.values.len() {
            &self.sentinel
        } else {
            let raw : &Option<V> = self.values.get(idx).unwrap_or(&None);
            raw.as_ref().unwrap_or_else(|| &self.sentinel)
        }
    }

    pub fn get_if_present(&self, k :&K) -> Option<&V> {
        let idx = self.hash(k);

        if idx >= self.values.len() {
            None
        } else {
            self.values.get(idx).unwrap_or(&None).as_ref()
        }
    }

    pub fn get_mut(&mut self, k :&K) -> &mut V {
        let idx = self.hash(k);

        if idx >= self.values.len() {
            panic!("attempted to get_mut on non-present key");
        } else {
            let raw : Option<&mut Option<V>> = self.values.get_mut(idx);
            raw.unwrap().as_mut().unwrap()
        }
    }


    pub fn put(&mut self, k : K, v : V) {
        let idx = self.hash(&k);
        if idx >= self.values.len() {
            self.values.resize_default(idx+1);
        }
        if self.values[idx].is_none() {
            self.keys.push(k);
        }
        self.values[idx] = Some(v);
    }

    fn hash(&self, k : &K) -> usize {
//        let mut hasher = self.hasher_builder.build_hasher();
//        k.hash(&mut hasher);
//        hasher.finish() as usize
        k.hash()
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K,V> {
        Iter {
            map: self,
            pos: 0
        }
    }
}

impl <'a, K : PerfectHashable, V> IntoIterator for &'a PerfectHashMap<K,V> {
    type Item = (&'a K,&'a V);
    type IntoIter = Iter<'a, K,V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map : &self,
            pos : 0
        }
    }
}

pub struct Iter<'a, K : PerfectHashable + 'a, V : 'a>  {
    map : &'a PerfectHashMap<K,V>,
    pos : usize
}

impl <'a, K : PerfectHashable, V> Iterator for Iter<'a, K,V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.map.keys.len() > self.pos {
            let key : &'a K = unsafe { self.map.keys.get_unchecked(self.pos) };
            self.pos += 1;
            Some((key, self.map.get(key)))
        } else {
            None
        }
    }
}


#[derive(Clone,Default,Eq,PartialEq,Debug)]
struct TestStruct {
    txt : &'static str
}

#[derive(Clone,Copy,Default,Eq,PartialEq,Debug)]
struct TestRef(u32);
impl PerfectHashable for TestRef {
    fn hash(&self) -> usize {
        self.0 as usize
    }
}

#[test]
pub fn test_perfect_hash_map () {
    let test_ref = TestRef(1);
    let test_struct = TestStruct {
        txt : "hello"
    };
    let sentinel = TestStruct {
        txt : "sentinel"
    };

    let mut map = PerfectHashMap::new(sentinel.clone());

    map.put(test_ref, test_struct);

    map.put(TestRef(2), TestStruct { txt : "two" });

    assert_eq!(*map.get(&TestRef(1)), TestStruct { txt : "hello" });

    assert_eq!(*map.get(&TestRef(2)), TestStruct { txt : "two" });

    assert_eq!(*map.get(&TestRef(3)), TestStruct { txt : "sentinel" });

    let mut keys = vec![];
    let mut values = vec![];
    for (k,v) in map.iter() {
        keys.push(k.clone());
        values.push(v.clone());
    }

    assert_eq!(keys.len(), 2);
    assert_eq!(TestRef(1), *keys.get(0).unwrap());
    assert_eq!(TestRef(2), *keys.get(1).unwrap());

    assert_eq!(values.len(), 2);
    assert_eq!(TestStruct { txt : "hello" }, *values.get(0).unwrap());
    assert_eq!(TestStruct { txt : "two" }, *values.get(1).unwrap());
}