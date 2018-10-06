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
use color::Color;

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


#[derive(Clone,Copy,PartialEq,Eq,Debug,Serialize,Deserialize)]
pub struct RichStringStyle {
    pub bold : bool,
    pub color : Color
}
#[allow(non_upper_case_globals)]
impl RichStringStyle {
    const BoldStruct : RichStringStyle = RichStringStyle { bold : true, color : Color([0.0,0.0,0.0,1.0]) };
    const PlainStruct : RichStringStyle = RichStringStyle { bold : false, color : Color([1.0,1.0,1.0,1.01])};

    pub const Bold : &'static RichStringStyle = &RichStringStyle::BoldStruct;
    pub const Plain : &'static RichStringStyle = &RichStringStyle::PlainStruct;
}

#[derive(Clone,PartialEq,Eq,Debug,Serialize,Deserialize)]
pub struct RichStringSection {
    pub string : String,
    pub style : RichStringStyle
}

#[derive(Clone,PartialEq,Eq,Debug,Serialize,Deserialize)]
pub struct RichString {
    sections : Vec<RichStringSection>
}

impl RichString {
    pub fn new () -> RichString { RichString { sections : Vec::new() } }

    pub fn sections(&self) -> &[RichStringSection] { &self.sections }

    pub fn append<S : Into<String>>(&mut self, string : S, style : &RichStringStyle) -> &mut Self {
        self.sections.push(RichStringSection {
            string : string.into(),
            style : style.clone()
        });
        self
    }

    pub fn with_appended<S : Into<String>>(mut self, string : S, style : &RichStringStyle) -> Self {
        self.append(string, style);
        self
    }

    pub fn as_plain_string(&self) -> String {
        self.sections.map(|s| s.string.clone()).join("")
    }

//    pub fn from(raw_string : &str) -> RichString {
//        use regex;
//        let pattern = regex::Regex::new(r"(+>\(.*?\)|->).*?").unwrap();
//        for raw_section in pattern.find_iter().map(|m| m.text) {
//            if raw_section.starts_with("+>(") {
//                raw_section.split
//            }
//        }
//    }
}