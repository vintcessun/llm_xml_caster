use crate::{Cache, LlmPrompt};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::marker::PhantomData;

pub struct VecParser<T>(PhantomData<T>);

#[derive(Deserialize)]
struct ItemWrapper<T> {
    #[serde(rename = "$value")]
    content: T,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct XmlSeq<T> {
    #[serde(rename = "item", default = "Vec::new")]
    items: Vec<ItemWrapper<T>>,
}

impl<T> VecParser<T>
where
    T: DeserializeOwned,
{
    pub fn custom_vector_parser<'de, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match XmlSeq::<T>::deserialize(deserializer) {
            Ok(wrapper) => Ok(wrapper.items.into_iter().map(|w| w.content).collect()),
            Err(e) => Err(serde::de::Error::custom(format!(
                "The XML structure is invalid. It must be a sequence of <item> elements, each containing the value. Details: {}",
                e
            ))),
        }
    }
}

impl<T: LlmPrompt + 'static> LlmPrompt for Vec<T> {
    fn get_prompt_schema() -> &'static str {
        let sub_schema = T::get_prompt_schema();
        let cache = Cache::<Vec<T>>::get();
        cache.prompt_schema.get_or_init(|| {
            format!("A series(0 or more elements) of items where each item has the following format:<item>{}</item>\nNOTICE: Even a single item must be enclosed within <item></item> tags.", sub_schema)
        })
    }

    fn root_name() -> &'static str {
        let sub_root_name = T::root_name();
        let cache = Cache::<Vec<T>>::get();
        cache
            .root_name
            .get_or_init(|| format!("Vec<{}>", sub_root_name))
    }

    const IS_ENUM: bool = false;
}
