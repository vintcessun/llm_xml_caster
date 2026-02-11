use crate::LlmPrompt;
use serde::{Deserialize, Deserializer};

pub fn custom_bool_parser<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    match s.trim().to_lowercase().as_str() {
        // the true values bucket
        "true" | "1" | "yes" | "y" | "t" | "on" | "真" | "checked" | "selected" => Ok(true),
        // the false values bucket
        "false" | "0" | "no" | "n" | "f" | "off" | "假" | "null" | "none" | "" => Ok(false),
        // if the LLM outputs other nonsense, default to error
        _ => Err(serde::de::Error::custom(format!(
            "can not parse '{}' as a boolean value",
            s
        ))),
    }
}

impl LlmPrompt for bool {
    fn get_prompt_schema() -> &'static str {
        "Boolean value, either `true` or `false`"
    }
    fn root_name() -> &'static str {
        "bool"
    }
}
