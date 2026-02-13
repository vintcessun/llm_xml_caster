//! A robust Rust library for converting structured XML output from Large Language Models (LLMs)
//! back into Rust data structures.
//!
//! This library provides a procedural macro (`#[llm_prompt]`) to automatically generate XML schemas
//! and prompts from your Rust structs, ensuring LLMs generate predictable and correct XML output
//! for reliable deserialization.
//!
//! For more details and usage examples, see the [README](https://github.com/vintcessun/llm_xml_caster).

mod bind;
mod error;
pub mod r#type;

pub type Error = error::RequestError;
pub type Result<T> = std::result::Result<T, Error>;
pub use r#type::*;

/// Trait implemented by structures annotated with `#[llm_prompt]`.
///
/// This trait provides the necessary methods to obtain the XML schema and root element name,
/// which are crucial for instructing the Large Language Model (LLM) on the desired output format.
pub trait LlmPrompt {
    /// Returns the XML schema string containing field descriptions for LLM prompting.
    fn get_prompt_schema() -> &'static str;
    /// Returns the root XML element name expected by the deserializer.
    fn root_name() -> &'static str;
    /// Indicates whether the type is an enum.
    const IS_ENUM: bool;
}

pub use bind::{generate_as, generate_as_with_retries};
/// Procedural macro used to derive `LlmPrompt` implementation and integrate custom deserialization
/// logic for LLM-generated XML.
///
/// Place this attribute on any struct or enum that you want to be able to cast from XML output.
/// Use `#[prompt("description")]` on fields to provide context to the LLM.
pub use llm_xml_caster_helper::llm_prompt;
