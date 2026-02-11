# GitHub Copilot Instructions for llm_xml_caster ✅

## Purpose / Big picture 💡
- This repository provides a tiny framework to ask LLMs (via the `genai` client) to return **strict XML** that can be deserialized into strongly typed Rust structs/enums using `serde` + `quick-xml`.
- Two crates: `llm_xml_caster` (library) and `helper` (a proc-macro crate that implements the `#[llm_prompt]` attribute and `#[derive(LlmPrompt)]`).

## Key files & modules 🔧
- `src/lib.rs` — exposes `LlmPrompt` trait and re-exports macro helpers.
- `src/bind.rs` — the runtime integration with the `genai` client. Entry points: `generate_as` and `generate_as_with_retries`.
- `src/error.rs` — centralized error types (Request, XML extraction, deserialization, retry limit).
- `src/type/*` — per-type parsing and schema helpers (e.g., `vector.rs`, `option.rs`, `bool.rs`, `string.rs`, numeric via `macro.rs`).
- `helper/src/lib.rs` — proc-macro logic that generates `LlmPrompt` implementations and (important) auto-injects `#[serde(deserialize_with = "...")]` for fields with custom parsers.
- `tests/derive_test.rs` — canonical examples and tests demonstrating usage patterns (recommended reference for new changes).

## How it works (short) 🔁
- Types implement `LlmPrompt` which must return:
  - `get_prompt_schema() -> &'static str` describing the expected XML format
  - `root_name() -> &'static str` the expected root tag
- `bind::generate_as` asks the LLM to return XML following `T::get_prompt_schema()` and extracts the substring between `<root>` and `</root>` before deserializing via `quick-xml` into `T`.
- The `helper` proc-macro produces helpful nested XML schema text for complex structs/enums and automatically wires in appropriate `serde` deserializers for `Vec`, `Option`, primitives, etc.

## Project-specific conventions & patterns ⚠️
- Field doc in prompts: use `#[prompt("description")]` on struct fields and enum variants to include human descriptions in generated schema (see `tests/derive_test.rs`).
- Two ways to generate `LlmPrompt`:
  - `#[derive(LlmPrompt)]` — supports *structs* only (backward compatible)
  - `#[llm_prompt]` attribute — supports *structs and enums* (preferred for enums with variant-level `#[prompt(...)]` descriptions)
- String values must be returned inside CDATA: <![CDATA[...]]> to avoid XML escaping issues (see `src/type/string.rs`). Tests assume this format.
- Vec format: elements must be wrapped in `<item>...</item>` tags (see `src/type/vector.rs`).
- Option: if the optional value is absent, omit the tag entirely (see `src/type/option.rs`).
- Numeric parsing: numeric fields accept either numeric literal or quoted strings; numeric serde parsers are generated via macro (`custom_i32_parser`, etc.).
- Boolean parsing accepts many common variants (e.g., "true/false", "1/0", "yes/no", Chinese words "真/假", and other synonyms).

## Adding a new type or custom parser 🧩
1. Add a module in `src/type/` (e.g., `mytype.rs`).
2. Implement `LlmPrompt` for the type and export it in `src/type/mod.rs`.
3. If needed for field-level deserialization, provide a `custom_*_parser` function and ensure the `helper` macro will use the same path pattern
   (the macro expects functions at paths like `::llm_xml_caster::custom_i32_parser` or `::llm_xml_caster::VecParser::<Inner>::custom_vector_parser`).
4. Add unit tests similar to `tests/derive_test.rs` demonstrating both the schema generation and deserialization behavior.

## Build / test / debug workflows 🛠️
- Build: `cargo build` (workspace root).
- Test: `cargo test` (or `cargo test -p llm_xml_caster` for the library)
- To print generated schemas in tests, run: `cargo test -- --nocapture`.
- The LLM runtime uses the `genai::Client` — integration tests that call `generate_as` require a valid `genai` setup and network access.
- When debugging model-driven failures, `generate_as_with_retries` appends assistant messages containing the previous XML and error; inspect those appended messages to diagnose format mismatches.

## Integration & Dependencies 🔗
- `genai` (git dependency) — the chat model client used to send prompts. Look at `src/bind.rs` to follow how prompts are assembled.
- `quick-xml` — XML deserialization
- `serde` — deserialization hooks and `deserialize_with` custom parsers
- `proc-macro` helpers (`helper` crate) use `syn`/`quote`/`proc-macro2` and auto-generate code expected by the main crate.

## Important gotchas / notes ⚠️
- The proc-macro `llm_prompt` will mutate struct fields to insert `#[serde(deserialize_with = "...")]` for supported types. Be careful when editing proc-macro code — changes affect generated serde behavior across the workspace.
- `get_prompt_schema()` values are cached using a global `Cache<T>` (see `src/type/mod.rs`) built on `dashmap::DashMap` + `OnceLock`. This is to avoid repeatedly building big schemas and to work around potential ICF issues.
- Enum `#[llm_prompt]` uses a special root string `XML_ENUM_ROOT` and produces a descriptive list of possible XML alternatives — the runtime expects the LLM to return one of them.

## Examples to copy / paste ✂️
- Struct with prompts + derive:
  ```rust
  #[derive(LlmPrompt, Deserialize)]
  struct MyStruct {
      #[prompt("user id")]
      id: i32,
      #[prompt("display name")]
      name: String,
  }
  ```
- Enum with attribute macro (preferred for enums):
  ```rust
  #[llm_prompt]
  #[derive(Deserialize)]
  enum MyEnum {
      #[prompt("simple variant")]
      Simple,
      #[prompt("variant with data")]
      Data { #[prompt("value")] v: i32 },
  }
  ```

---

If any of these items are unclear or missing (e.g., you want more examples, special CI/test commands, or readme text for contributors), say which sections to expand and I will iterate. 🔁