use crate::LlmPrompt;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::marker::PhantomData;

pub struct OrderedFloatParser<T: DeserializeOwned>(PhantomData<T>);

#[derive(Deserialize)]
#[serde(transparent)]
struct XmlOrderedFloat<T>(T);

impl<T: DeserializeOwned> OrderedFloatParser<T> {
    pub fn custom_ordered_float_parser<'de, D>(deserializer: D) -> Result<OrderedFloat<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match XmlOrderedFloat::<T>::deserialize(deserializer) {
            Ok(wrapper) => Ok(OrderedFloat(wrapper.0)),
            Err(e) => Err(serde::de::Error::custom(format!(
                "The XML structure is invalid. Reason: {}",
                e
            ))),
        }
    }
}

impl<T: LlmPrompt + 'static> LlmPrompt for OrderedFloat<T> {
    fn get_prompt_schema() -> &'static str {
        T::get_prompt_schema()
    }
    fn root_name() -> &'static str {
        T::root_name()
    }
}
