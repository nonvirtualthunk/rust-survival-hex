use serde::Serialize;
use ron;
use bincode;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use anymap::any::CloneAny;
use anymap::Map;
use std::collections::HashMap;
use serde::Deserializer;
use serialize::SerializableError::InvalidDataFormat;
use serde::Serializer;
use serde::de::Visitor;
use std::fmt::Formatter;
use std::fmt;
use serde::de::MapAccess;

#[derive(Clone,Debug)]
pub enum SerializableError {
    Error,
    InvalidDataFormat
}