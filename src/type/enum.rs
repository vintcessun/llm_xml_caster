use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::marker::PhantomData;

pub struct EnumParser<T: DeserializeOwned>(PhantomData<T>);

#[derive(Deserialize)]
struct EnumWrapper<T> {
    #[serde(rename = "$value")]
    content: T,
}

impl<T: DeserializeOwned> EnumParser<T> {
    pub fn custom_enum_parser<'de, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        EnumWrapper::<T>::deserialize(deserializer).map(|w| w.content)
    }
}
