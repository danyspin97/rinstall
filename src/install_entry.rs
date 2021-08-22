use std::{fmt, marker::PhantomData, path::PathBuf, str::FromStr};

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use void::Void;

#[derive(Deserialize)]
pub struct InstallEntry {
    #[serde(rename(deserialize = "src"))]
    pub source: PathBuf,
    #[serde(rename(deserialize = "dst"))]
    pub destination: Option<PathBuf>,
    #[serde(default, rename(deserialize = "tmpl"))]
    pub templating: bool,
}

impl InstallEntry {
    pub fn new_with_source(source: PathBuf) -> Self {
        InstallEntry {
            source,
            destination: None,
            templating: false,
        }
    }
}

impl FromStr for InstallEntry {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(InstallEntry::new_with_source(PathBuf::from(s)))
    }
}

// https://serde.rs/string-or-struct.html
pub fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(
            &self,
            formatter: &mut fmt::Formatter,
        ) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(
            self,
            value: &str,
        ) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(
            self,
            map: M,
        ) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
