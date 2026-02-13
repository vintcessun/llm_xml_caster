use llm_xml_caster::{LlmPrompt, llm_prompt};
use ordered_float::OrderedFloat;
use quick_xml::de::from_str;
use serde::Deserialize;

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
struct SimpleStruct {
    #[prompt("The name of the person")]
    name: String,
    #[prompt("The age of the person")]
    age: i32,
    #[prompt("Whether the person is a student")]
    is_student: bool,
}

#[test]
fn test_simple_struct_schema() {
    let schema = SimpleStruct::get_prompt_schema();
    println!("Schema :\n{}", schema);
    assert!(schema.contains("<SimpleStruct>"));
    assert!(schema.contains("<name>"));
    assert!(schema.contains("The name of the person"));
    assert!(schema.contains("<age>"));
    assert!(schema.contains("The age of the person"));
    assert!(schema.contains("<is_student>"));
    assert!(schema.contains("Whether the person is a student"));
    assert_eq!(SimpleStruct::root_name(), "SimpleStruct");
}

#[test]
fn test_simple_struct_deserialization() {
    // Testing custom parsers (e.g., bool parsing "yes", i32 parsing from string)
    let xml = r#"
    <SimpleStruct>
        <name><![CDATA[John Doe]]></name>
        <age>30</age>
        <is_student>yes</is_student>
    </SimpleStruct>
    "#;
    let decoded: SimpleStruct = from_str(xml).unwrap();
    assert_eq!(
        decoded,
        SimpleStruct {
            name: "John Doe".to_string(),
            age: 30,
            is_student: true,
        }
    );
}

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
struct NestedStruct {
    #[prompt("The person details")]
    person: SimpleStruct,
    #[prompt("The score of the person")]
    score: f32,
    #[prompt("State")]
    state: bool,
}

#[test]
fn test_nested_struct_schema() {
    let schema = NestedStruct::get_prompt_schema();
    println!("Schema :\n{}", schema);
    assert!(schema.contains("<NestedStruct>"));
    assert!(schema.contains("<person>"));
    assert!(schema.contains("<SimpleStruct>"));
    assert!(schema.contains("<score>"));
    assert!(schema.contains("The score of the person"));
    assert!(schema.contains("<state>"));
    assert!(schema.contains("State"));
    assert_eq!(NestedStruct::root_name(), "NestedStruct");
}

#[test]
fn test_nested_struct_deserialization() {
    let xml = r#"
    <NestedStruct>
        <person>
            <name><![CDATA[Jane Doe]]></name>
            <age>25</age>
            <is_student>false</is_student>
        </person>
        <score>85.5</score>
        <state>true</state>
    </NestedStruct>
    "#;
    let decoded: NestedStruct = from_str(xml).unwrap();
    assert_eq!(
        decoded,
        NestedStruct {
            person: SimpleStruct {
                name: "Jane Doe".to_string(),
                age: 25,
                is_student: false,
            },
            score: 85.5,
            state: true,
        }
    );
}

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
enum TestEnum {
    #[prompt("A simple variant")]
    Simple,
    #[prompt("A variant with data")]
    WithData {
        #[prompt("The value of the variant")]
        value: i32,
    },
}

#[test]
fn test_enum_schema() {
    let schema = TestEnum::get_prompt_schema();
    assert!(
        schema.contains("The following are possible XML structures for the current enum type:")
    );
    assert!(schema.contains("<Simple/>"));
    assert!(schema.contains("A simple variant"));
    assert!(schema.contains("<WithData>"));
    assert!(schema.contains("<value>"));
    assert!(schema.contains("A variant with data"));
    assert!(schema.contains("The value of the variant"));
    assert_eq!(TestEnum::root_name(), "XML_ENUM_ROOT");
}

#[test]
fn test_enum_deserialization() {
    let xml_simple = r#"<Simple/>"#;
    let decoded_simple: TestEnum = from_str(xml_simple).unwrap();
    assert_eq!(decoded_simple, TestEnum::Simple);

    let xml_data = r#"<WithData><value>123</value></WithData>"#;
    let decoded_data: TestEnum = from_str(xml_data).unwrap();
    assert_eq!(decoded_data, TestEnum::WithData { value: 123 });
}

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
struct CollectionsStruct {
    #[prompt("A list of strings")]
    tags: Vec<String>,
    #[prompt("An optional description")]
    description: Option<String>,
}

#[test]
fn test_collections_schema() {
    let schema = CollectionsStruct::get_prompt_schema();
    println!("Schema :\n{}", schema);
    assert!(schema.contains("<tags>"));
    assert!(schema.contains("<item>"));
    assert!(schema.contains("A series(0 or more elements) of items where "));
    assert!(schema.contains("<description>"));
    assert!(schema.contains("Optional. if not provided"));
}

#[test]
fn test_collections_deserialization() {
    let xml = r#"
    <CollectionsStruct>
        <tags>
            <item><![CDATA[tag1]]></item>
            <item><![CDATA[tag2]]></item>
        </tags>
        <description><![CDATA[Hello World]]></description>
    </CollectionsStruct>
    "#;
    let decoded: CollectionsStruct = from_str(xml).unwrap();
    println!("Decoded: {:?}", decoded);
    assert_eq!(
        decoded,
        CollectionsStruct {
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            description: Some("Hello World".to_string()),
        }
    );

    let xml_empty = r#"
    <CollectionsStruct>
        <tags></tags>
    </CollectionsStruct>
    "#;
    let decoded_empty: CollectionsStruct = from_str(xml_empty).unwrap();
    println!("Decoded empty: {:?}", decoded_empty);
    assert_eq!(
        decoded_empty,
        CollectionsStruct {
            tags: vec![],
            description: None,
        }
    );
}

#[test]
fn test_ordered_float_deserialization() {
    #[llm_prompt]
    #[derive(Deserialize, Debug, PartialEq)]
    struct OrderedFloatStruct {
        #[prompt("An ordered float value")]
        value: OrderedFloat<f64>,
    }

    let xml = r#"
    <OrderedFloatStruct>
        <value>114.514</value>
    </OrderedFloatStruct>
    "#;
    let decoded: OrderedFloatStruct = from_str(xml).unwrap();
    assert_eq!(
        decoded,
        OrderedFloatStruct {
            value: OrderedFloat(114.514),
        }
    );
}

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
struct ComplexStruct {
    #[prompt("A nested struct")]
    nested: NestedStruct,
    #[prompt("A list of enums")]
    enum_list: Vec<TestEnum>,
    #[prompt("An optional ordered float value")]
    optional_float: Option<OrderedFloat<f32>>,
}

#[test]
fn test_complex_struct_deserialization() {
    let xml = r#"
    <ComplexStruct>
        <nested>
            <person>
                <name><![CDATA[Alice]]></name>
                <age>28</age>
                <is_student>false</is_student>
            </person>
            <score>92.0</score>
            <state>true</state>
        </nested>
        <enum_list>
            <item><Simple/></item>
            <item><WithData><value>456</value></WithData></item>
        </enum_list>
        <optional_float>19.19</optional_float>
    </ComplexStruct>
    "#;
    let decoded: ComplexStruct = from_str(xml).unwrap();
    assert_eq!(
        decoded,
        ComplexStruct {
            nested: NestedStruct {
                person: SimpleStruct {
                    name: "Alice".to_string(),
                    age: 28,
                    is_student: false,
                },
                score: 92.0,
                state: true,
            },
            enum_list: vec![TestEnum::Simple, TestEnum::WithData { value: 456 }],
            optional_float: Some(OrderedFloat(19.19)),
        }
    );
}
