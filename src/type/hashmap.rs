use crate::{Cache, LlmPrompt};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Deserialize)]
struct EntryWrapper<T> {
    #[serde(rename = "$value")]
    val: T,
}

#[derive(Deserialize)]
struct Entry<K, V> {
    key: EntryWrapper<K>,
    value: EntryWrapper<V>,
}

#[derive(Deserialize)]
struct XmlMap<K, V> {
    #[serde(rename = "entry", default = "Vec::new")]
    entries: Vec<Entry<K, V>>,
}

pub struct HashMapParser<K, V>(PhantomData<(K, V)>)
where
    K: DeserializeOwned + Eq + Hash,
    V: DeserializeOwned;

impl<K, V> HashMapParser<K, V>
where
    K: DeserializeOwned + Eq + Hash,
    V: DeserializeOwned,
{
    pub fn custom_hashmap_parser<'de, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match XmlMap::<K, V>::deserialize(deserializer) {
            Ok(wrapper) => {
                let map = wrapper
                    .entries
                    .into_iter()
                    .map(|e| (e.key.val, e.value.val))
                    .collect();
                Ok(map)
            }
            Err(e) => Err(serde::de::Error::custom(format!(
                "The XML structure is invalid. The sequence must consist of <entry> elements, each containing a <key> and a <value>. Details: {}",
                e
            ))),
        }
    }
}

impl<K, V> LlmPrompt for HashMap<K, V>
where
    K: LlmPrompt + Eq + Hash + 'static,
    V: LlmPrompt + 'static,
{
    fn get_prompt_schema() -> &'static str {
        let key_schema = K::get_prompt_schema();
        let val_schema = V::get_prompt_schema();
        let cache = Cache::<HashMap<K, V>>::get();
        cache.prompt_schema.get_or_init(|| {
            format!("a sequence of key-value pairs, where each key is {} and each value is {}. The XML format should be: <entry><key>{{key}}</key><value>{{value}}</value></entry>, and this structure can be repeated multiple times.", key_schema, val_schema)
        })
    }

    fn root_name() -> &'static str {
        let key_name = K::root_name();
        let val_name = V::root_name();
        let cache = Cache::<HashMap<K, V>>::get();
        cache
            .root_name
            .get_or_init(|| format!("HashMap<{}, {}>", key_name, val_name))
    }

    const IS_ENUM: bool = false;
}
