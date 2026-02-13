use crate::{Error, LlmPrompt, Result};
use genai::{
    Client,
    chat::{ChatMessage, ChatOptions, ChatRequest},
};
use quick_xml::de::from_str;
use serde::de::DeserializeOwned;

/// Attempts to generate structured data of type `T` from an LLM response.
///
/// This function uses a default retry limit of 3 attempts. It constructs a system message
/// instructing the LLM to return a valid XML document based on the schema derived from `T`.
///
/// # Arguments
///
/// * `client` - The `genai::Client` used for the API request.
/// * `model_name` - The name of the model to use (e.g., "gemini-2.5-flash").
/// * `prompt` - The initial user prompt messages.
/// * `valid_example` - A valid XML example string to guide the LLM.
pub async fn generate_as<T: DeserializeOwned + LlmPrompt>(
    client: &Client,
    model_name: &str,
    prompt: Vec<ChatMessage>,
    valid_example: &str,
) -> Result<T> {
    generate_as_with_retries(client, model_name, prompt, valid_example, 3).await
}

/// Attempts to generate structured data of type `T` from an LLM response with a specified number of retries.
///
/// If the LLM response does not contain valid XML or the XML cannot be deserialized,
/// the function will send a follow-up message to the LLM with the error details,
/// prompting it to correct its output.
///
/// # Arguments
///
/// * `client` - The `genai::Client` used for the API request.
/// * `model_name` - The name of the model to use (e.g., "gemini-2.5-flash").
/// * `prompt` - The initial user prompt messages.
/// * `valid_example` - A valid XML example string to guide the LLM.
/// * `retries` - The maximum number of attempts to correct and regenerate the output.
///
/// # Errors
///
/// Returns `Error::RetryLimitExceeded` if the XML output remains invalid after all retry attempts.
pub async fn generate_as_with_retries<T: DeserializeOwned + LlmPrompt>(
    client: &Client,
    model_name: &str,
    prompt: Vec<ChatMessage>,
    valid_example: &str,
    retries: usize,
) -> Result<T> {
    let chat_req = ChatRequest::new(prompt);
    let mut chat_req = chat_req.append_message(
        ChatMessage::system(format!("You must respond with a valid XML document(root name is {}) that adheres to the following schema: {}", T::root_name(), T::get_prompt_schema()))
    );
    let options = ChatOptions::default().with_temperature(0.1);

    let mut errs = Vec::new();

    for _attempt in 1..=retries {
        let res = client
            .exec_chat(model_name, chat_req.clone(), Some(&options))
            .await?;
        if let Some(text) = res.first_text() {
            let root_name = T::root_name();
            let start_tag = format!("<{}>", root_name);
            let end_tag = format!("</{}>", root_name);

            let xml_content: &str;
            let data: T;

            if let (Some(xml_start), Some(xml_end_tag_start)) =
                (text.find(&start_tag), text.rfind(&end_tag))
            {
                let xml_end = xml_end_tag_start + end_tag.len();
                xml_content = &text[xml_start..xml_end];
                data = match from_str(xml_content) {
                    Ok(v) => v,
                    Err(e) => {
                        chat_req = chat_req.append_message(
                            ChatMessage::assistant(format!("The last time you responded, the XML content was: {}\nThe error was: {}\nPlease ensure your response strictly follows the required XML format.\nThe format body is: {}", xml_content, e,T::get_prompt_schema()))
                        );
                        chat_req = chat_req.append_message(ChatMessage::assistant(format!(
                            "Here is a valid example for your reference:\n{}",
                            valid_example
                        )));
                        errs.push(Error::XmlDeserialization(e));
                        continue;
                    }
                };
            } else {
                errs.push(Error::XmlExtraction(format!(
                    "cannot find the root {} of the structure",
                    T::root_name()
                )));
                chat_req = chat_req.append_message(ChatMessage::assistant(format!("The error was: cannot find the root {} of the structure\nPlease ensure your response strictly follows the required XML format.\n The format body is: {}", T::root_name(), T::get_prompt_schema())));
                chat_req = chat_req.append_message(ChatMessage::assistant(format!(
                    "Here is a valid example for your reference:\n{}",
                    valid_example
                )));
                continue;
            };

            return Ok(data);
        }
    }

    Err(Error::RetryLimitExceeded(errs))
}
