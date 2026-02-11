use crate::LlmPrompt;

impl LlmPrompt for String {
    fn get_prompt_schema() -> &'static str {
        "return a string value. please use the format <![CDATA[{actual string content without any escaping}]]> to return the string content. Note that the CDATA tags must be exactly in this format, otherwise the parsing will fail. If you need to return an empty string, please return <![CDATA[]]>"
    }

    fn root_name() -> &'static str {
        "string"
    }
}
