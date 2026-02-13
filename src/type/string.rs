use crate::LlmPrompt;
use serde::{Deserialize, Deserializer};

pub fn custom_string_parser<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Ok(s.trim().to_string())
}

impl LlmPrompt for String {
    fn get_prompt_schema() -> &'static str {
        "return a string value. please use the format <![CDATA[{actual string content without any escaping}]]> to return the string content. Note that the CDATA tags must be exactly in this format, otherwise the parsing will fail. If you need to return an empty string, please return <![CDATA[]]>"
    }

    fn root_name() -> &'static str {
        "string"
    }
}
