use prelude::*;

use string_interner::StringInterner;
use string_interner;
use string_interner::Sym;
use std::sync::Mutex;
use std::fmt;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::de::Visitor;
use serde;
use string_interner::Symbol;

//lazy_static! {
//    static ref INTERNER : Mutex<StringInterner<Sym>> = Mutex::new(StringInterner::<Sym>::new());
//}

#[derive(PartialEq,Eq,Hash,Clone,Copy,Default,Debug)]
pub struct InternedString(Option<string_interner::Sym>);
pub type IStr = InternedString;

impl InternedString {
    pub fn resolve<'a, 'b>(&'a self, interner : &'b StringInterner<Sym>) -> &'b str {
        if let Some(sym) = self.0 {
            interner.resolve(sym).unwrap_or("")
        } else {
            ""
        }
    }
}

impl Serialize for InternedString {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        if let Some(sym) = self.0 {
            serializer.serialize_some(&(sym.to_usize() as u32))
        } else {
            serializer.serialize_none()
        }
    }
}

impl <'de> Deserialize<'de> for InternedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        struct IStrVisitor;
        impl <'de> Visitor<'de> for IStrVisitor {
            type Value = InternedString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt:: Error> {
                write!(formatter, "An interned string")
            }

            fn visit_u32<E>(self, v: u32) -> Result<<Self as Visitor<'de>>::Value, E> where E: serde::de::Error, {
                Ok(InternedString(Some(Sym::from_usize(v as usize))))
            }
        }
        deserializer.deserialize_option(IStrVisitor)
    }
}
//impl Into<InternedString> for Str {
//    fn into(self) -> InternedString {
//        InternedString(Some(INTERNER.lock().unwrap().get_or_intern(self)))
//    }
//}
//impl Into<InternedString> for String {
//    fn into(self) -> InternedString {
//        InternedString(Some(INTERNER.lock().unwrap().get_or_intern(self)))
//    }
//}
//impl Into<String> for InternedString {
//    fn into(self) -> String {
//        if let Some(sym) = self.0 {
//            String::from(INTERNER.lock().unwrap().resolve(sym).expect("interned string somehow absent"))
//        } else {
//            String::from("")
//        }
//    }
//}
//impl <'a> Into<String> for &'a InternedString {
//    fn into(self) -> String {
//        if let Some(sym) = self.0 {
//            String::from(INTERNER.lock().unwrap().resolve(sym).expect("interned string somehow absent"))
//        } else {
//            String::from("")
//        }
//    }
//}
//
//
//pub fn deserialize_interned_strings<'de, D : Deserializer<'de>>(deserializer : D) {
//    INTERNER.lock().unwrap().StringInterner::<Sym>::deserialize(deserializer)
//}
//
//
////impl From<String> for InternedString {
////    fn from(string: String) -> Self {
////        InternedString(Some(INTERNER.lock().unwrap().get_or_intern(string)))
////    }
////}
//
//impl fmt::Debug for InternedString {
//    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//        let string : String = self.into();
//        write!(f, "{}", string)
//    }
//}