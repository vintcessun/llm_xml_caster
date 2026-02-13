# llm_xml_caster: Robust XML Caster for LLM Structured Output in Rust

[![Crates.io Version](https://img.shields.io/crates/v/llm_xml_caster)](https://crates.io/crates/llm_xml_caster)
[![License](https://img.shields.io/crates/l/llm_xml_caster)](https://github.com/vintcessun/llm_xml_caster/blob/main/LICENSE)
[![Docs.rs](https://docs.rs/llm_xml_caster/badge.svg)](https://docs.rs/llm_xml_caster)

`llm_xml_caster` is a powerful and reliable Rust library designed to simplify the process of extracting structured data from Large Language Models (LLMs) via XML. By leveraging Rust's type system and procedural macros, it enables developers to define data structures once and automatically generate precise XML schemas and prompts for LLMs, ensuring high-fidelity data casting.

## Features

- **Automatic Schema Generation**: Automatically generates detailed XML schemas (including descriptions) from Rust structs and enums for precise LLM prompting.
- **Type Safety**: Integrates seamlessly with `serde` for safe and efficient deserialization.
- **Robust Parsing**: Built-in support for robust parsing of booleans (handles "yes/no", "true/false", "1/0") and numeric types from LLM output.
- **Rich Type Support**: Supports basic types, nested structs, enums, `Vec<T>`, `Option<T>`, `HashMap`, `BTreeMap`, and `OrderedFloat`.
- **Shadow Types (Weak Types)**: Support for generating shadow types (e.g., `TypeWeak`) to handle circular references or to reference complex structures in prompts without redundancy, saving tokens.
- **LLM Integration**: Direct integration with `genai` for structured generation with automatic retries and error correction.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
llm_xml_caster = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
# Optional for OrderedFloat support
# ordered_float = "4.0"
```

## Usage Example

### 1. Define Your Structure

Use `#[llm_prompt]` on your struct/enum and `#[prompt("...")]` on fields to provide guidance for the LLM.

```rust
use llm_xml_caster::{llm_prompt, LlmPrompt};
use serde::Deserialize;

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
pub struct SimpleStruct {
    #[prompt("The name of the person")]
    name: String,
    #[prompt("The age of the person")]
    age: i32,
    #[prompt("Whether the person is a student (true/false, yes/no)")]
    is_student: bool,
}
```

### 2. Generate XML Schema for Prompting

The `LlmPrompt` trait is automatically implemented:

```rust
let schema = SimpleStruct::get_prompt_schema();
// Generates:
// <SimpleStruct>
//   <name> <!-- The name of the person --> </name>
//   <age> <!-- The age of the person --> </age>
//   <is_student> <!-- Whether the person is a student (true/false, yes/no) --> </is_student>
// </SimpleStruct>
```

### 3. Deserialize LLM Output

```rust
use quick_xml::de::from_str;

let xml_output = r#"
<SimpleStruct>
    <name><![CDATA[John Doe]]></name>
    <age>30</age>
    <is_student>yes</is_student>
</SimpleStruct>
"#;

let decoded: SimpleStruct = from_str(xml_output).unwrap();
assert_eq!(decoded.is_student, true); // Correctly casts 'yes' to true
```

## Advanced Usage

### Nested Structs and Collections

```rust
#[llm_prompt]
#[derive(Deserialize, Debug)]
struct ComplexData {
    #[prompt("List of tags")]
    tags: Vec<String>,
    #[prompt("Optional metadata")]
    metadata: Option<SimpleStruct>,
}
```

### Enums (Sum Types)

```rust
#[llm_prompt]
#[derive(Deserialize, Debug)]
enum Action {
    #[prompt("Stop the process")]
    Stop,
    #[prompt("Move to coordinates")]
    Move { x: i32, y: i32 },
}
```

### Shadow Types (Weak Types)

Handle recursive structures or simplify prompts using `weak = true`:

```rust
#[llm_prompt(weak = true)]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum PythonValue {
    #[prompt("List of values")]
    List {
        // Uses automatically generated PythonValueWeak
        val: Vec<PythonValueWeak>,
    },
    // ...
}
```

The `PythonValueWeak` schema will be represented as `<PythonValue>Referencing the types above.</PythonValue>`, preventing infinite loops in prompt generation and saving tokens.

### Automated Generation with Retries

```rust
// Requires genai setup
let result: SimpleStruct = generate_as(
    &client,
    "gemini-3-flash",
    vec![ChatMessage::user("Give me person info")],
    "<SimpleStruct>...</SimpleStruct>"
).await?;
```

## Contributing

Contributions are welcome! Please check the issues and pull requests on [GitHub](https://github.com/vintcessun/llm_xml_caster).

## License

This project is licensed under the MIT License. See the [LICENSE](https://github.com/vintcessun/llm_xml_caster/blob/main/LICENSE) file for details.
