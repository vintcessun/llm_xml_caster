use crate::{Cache, LlmPrompt};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[derive(Deserialize)]
struct Entry<K, V> {
    key: K,
    value: V,
}

#[derive(Deserialize)]
struct XmlMap<K, V> {
    #[serde(rename = "entry", default = "Vec::new")]
    entries: Vec<Entry<K, V>>,
}

pub struct BTreeMapParser<K, V>(PhantomData<(K, V)>)
where
    K: DeserializeOwned + Ord,
    V: DeserializeOwned;

impl<K, V> BTreeMapParser<K, V>
where
    K: DeserializeOwned + Ord,
    V: DeserializeOwned,
{
    pub fn custom_btreemap_parser<'de, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match XmlMap::<K, V>::deserialize(deserializer) {
            Ok(wrapper) => {
                let map = wrapper
                    .entries
                    .into_iter()
                    .map(|e| (e.key, e.value))
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

impl<K, V> LlmPrompt for BTreeMap<K, V>
where
    K: LlmPrompt + Ord + 'static,
    V: LlmPrompt + 'static,
{
    fn get_prompt_schema() -> &'static str {
        let key_schema = K::get_prompt_schema();
        let val_schema = V::get_prompt_schema();
        let cache = Cache::<BTreeMap<K, V>>::get();
        cache.prompt_schema.get_or_init(|| {
            format!("a sequence of key-value pairs, where each key is {} and each value is {}. The XML format should be: <entry><key>{{key}}</key><value>{{value}}</value></entry>, and this structure can be repeated multiple times.", key_schema, val_schema)
        })
    }

    fn root_name() -> &'static str {
        let key_name = K::root_name();
        let val_name = V::root_name();
        let cache = Cache::<BTreeMap<K, V>>::get();
        cache
            .root_name
            .get_or_init(|| format!("BTreeMap<{}, {}>", key_name, val_name))
    }
}
