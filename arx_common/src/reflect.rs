use prelude::*;
use std::fmt::Formatter;
use std::fmt::Debug;
use std::fmt::Error;

pub struct Field<E, T : 'static> {
    pub name : Str,
    pub setter : fn(&mut E, T),
    pub getter : fn(&E) -> &T,
    pub getter_mut : fn(&mut E) -> &mut T,
}

impl <E, T : 'static> Debug for Field<E,T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.name)
    }
}

impl <E, T : 'static, U : 'static> PartialEq<Field<E,U>> for Field<E,T> {
    fn eq(&self, other: &Field<E, U>) -> bool {
        self.name == other.name
    }
}

impl <E,T : 'static> Field<E,T> {
    pub const fn new (name : Str, /*index : u32, */ getter : fn(&E) -> &T, getter_mut : fn(&mut E) -> &mut T, setter : fn(&mut E, T)) -> Field<E,T> {
        Field { name, /*index, */ getter, getter_mut, setter }
    }
}
