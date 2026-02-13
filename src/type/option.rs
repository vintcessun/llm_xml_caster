use super::Cache;
use crate::LlmPrompt;
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::marker::PhantomData;

pub struct OptionParser<T: DeserializeOwned>(PhantomData<T>);

#[derive(Deserialize)]
#[serde(transparent)]
struct XmlOption<T>(Option<T>);

impl<T: DeserializeOwned> OptionParser<T> {
    pub fn custom_option_parser<'de, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match XmlOption::<T>::deserialize(deserializer) {
            Ok(wrapper) => Ok(wrapper.0),
            Err(e) => Err(serde::de::Error::custom(format!(
                "The XML structure is invalid. Reason: {}",
                e
            ))),
        }
    }
}

impl<T: LlmPrompt + 'static> LlmPrompt for Option<T> {
    fn get_prompt_schema() -> &'static str {
        let sub_schema = T::get_prompt_schema();
        let cache = Cache::<Option<T>>::get();
        cache.prompt_schema.get_or_init(|| {
            format!("Optional. if not provided, do not include any tags. If provided, the format is: {}", sub_schema)
        })
    }

    fn root_name() -> &'static str {
        let sub_root_name = T::root_name();
        let cache = Cache::<Option<T>>::get();
        cache
            .root_name
            .get_or_init(|| format!("Option<{}>", sub_root_name))
    }

    const IS_ENUM: bool = false;
}
