use llm_xml_caster::{LlmPrompt, llm_prompt};
use ordered_float::OrderedFloat;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

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
    #[prompt("A variant with data string")]
    WithStringData {
        #[prompt("The value of the variant")]
        value: String,
    },
    #[prompt("A variant with data float")]
    WithFloatData {
        #[prompt("The float value of the variant")]
        value: f64,
    },
    #[prompt("A variant with data int")]
    WithIntData {
        #[prompt("The int value of the variant")]
        value: i64,
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
    assert!(schema.contains("<WithStringData>"));
    assert!(schema.contains("<value>"));
    assert!(schema.contains("A variant with data string"));
    assert!(schema.contains("The value of the variant"));
    assert!(schema.contains("<WithFloatData>"));
    assert!(schema.contains("A variant with data float"));
    assert!(schema.contains("<WithIntData>"));
    assert!(schema.contains("A variant with data int"));
    assert_eq!(TestEnum::root_name(), "");
}

#[test]
fn test_enum_deserialization() {
    let xml_simple = r#"<Simple/>"#;
    let decoded_simple: TestEnum = from_str(xml_simple).unwrap();
    assert_eq!(decoded_simple, TestEnum::Simple);

    let xml_data_string = r#"<WithStringData>
    <value>aaa</value>
    </WithStringData>"#;
    let decoded_data: TestEnum = from_str(xml_data_string).unwrap();
    assert_eq!(
        decoded_data,
        TestEnum::WithStringData {
            value: "aaa".to_string()
        }
    );
    let xml_data_float = r#"<WithFloatData>
    <value>2.33</value>
    </WithFloatData>"#;
    let decoded_data_float: TestEnum = from_str(xml_data_float).unwrap();
    assert_eq!(decoded_data_float, TestEnum::WithFloatData { value: 2.33 });
    let xml_data_int = r#"<WithIntData>
    <value>114514</value>
    </WithIntData>"#;
    let decoded_data_int: TestEnum = from_str(xml_data_int).unwrap();
    assert_eq!(decoded_data_int, TestEnum::WithIntData { value: 114514 });
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
            <item><WithIntData><value>456</value></WithIntData></item>
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
            enum_list: vec![TestEnum::Simple, TestEnum::WithIntData { value: 456 }],
            optional_float: Some(OrderedFloat(19.19)),
        }
    );
}

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
struct ThirdStruct {
    #[prompt("An optional list of strings")]
    optional_list: Option<Vec<String>>,
}

#[test]
fn test_third_struct_deserialization() {
    let xml_with_list = r#"
    <ThirdStruct>
        <optional_list>
            <item><![CDATA[item1]]></item>
            <item><![CDATA[item2]]></item>
        </optional_list>
    </ThirdStruct>
    "#;
    let decoded_with_list: ThirdStruct = from_str(xml_with_list).unwrap();
    assert_eq!(
        decoded_with_list,
        ThirdStruct {
            optional_list: Some(vec!["item1".to_string(), "item2".to_string()]),
        }
    );

    let xml_without_list = r#"
    <ThirdStruct>
    </ThirdStruct>
    "#;
    let decoded_without_list: ThirdStruct = from_str(xml_without_list).unwrap();
    assert_eq!(
        decoded_without_list,
        ThirdStruct {
            optional_list: None,
        }
    );
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(transparent)]
pub struct PythonValueWeak(PythonValue);

impl LlmPrompt for PythonValueWeak {
    fn get_prompt_schema() -> &'static str {
        "<PythonValue>the type of the value is showed</PythonValue>"
    }
    fn root_name() -> &'static str {
        "PythonValue"
    }
}

#[llm_prompt]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum PythonValue {
    #[prompt("python's None value")]
    None,
    #[prompt("python's str value")]
    String {
        #[prompt("the value of the string")]
        val: String,
    },
    #[prompt("python's int value")]
    Int {
        #[prompt("the value of the int")]
        val: i64,
    },
    #[prompt("python's float value")]
    Float {
        #[prompt("the value of the float")]
        val: OrderedFloat<f64>,
    },
    #[prompt("python's bool value")]
    Bool {
        #[prompt("the value of the bool")]
        val: bool,
    },
    #[prompt("python's list value")]
    List {
        #[prompt("the value of the list")]
        val: Vec<PythonValueWeak>,
    },
    #[prompt("python's dict value")]
    Dict {
        #[prompt("the value of the dict")]
        val: BTreeMap<PythonValueWeak, PythonValueWeak>,
    },
}

#[test]
fn test_python_value_schema() {
    let schema = PythonValue::get_prompt_schema();
    println!("Schema :\n{}", schema);
    assert!(schema.contains("<PythonValue>"));
    assert!(schema.contains("the type of the value is showed"));
    assert_eq!(PythonValue::root_name(), "");
}

#[test]
fn test_python_value_deserialization() {
    let xml_string = r#"<String><val><![CDATA[test]]></val></String>"#;
    let decoded_string: PythonValue = from_str(xml_string).unwrap();
    assert_eq!(
        decoded_string,
        PythonValue::String {
            val: "test".to_string()
        }
    );
    let xml_int = r#"<Int><val>42</val></Int>"#;
    let decoded_int: PythonValue = from_str(xml_int).unwrap();
    assert_eq!(decoded_int, PythonValue::Int { val: 42 });

    let xml_float = r#"<Float><val>1.14</val></Float>"#;
    let decoded_float: PythonValue = from_str(xml_float).unwrap();
    assert_eq!(
        decoded_float,
        PythonValue::Float {
            val: OrderedFloat(1.14)
        }
    );
    let xml_bool = r#"<Bool><val>true</val></Bool>"#;
    let decoded_bool: PythonValue = from_str(xml_bool).unwrap();
    assert_eq!(decoded_bool, PythonValue::Bool { val: true });

    let xml_list = r#"<List>
    <val>
        <item><Float><val>1.14</val></Float></item>
        <item><Bool><val>true</val></Bool></item>
    </val>
</List>"#;
    let decoded_list: PythonValue = from_str(xml_list).unwrap();
    assert_eq!(
        decoded_list,
        PythonValue::List {
            val: vec![
                PythonValueWeak(PythonValue::Float {
                    val: OrderedFloat(1.14)
                }),
                PythonValueWeak(PythonValue::Bool { val: true }),
            ]
        }
    );

    let xml_map = r#"
    <Dict>
        <val>
            <entry>
                <key><String><val><![CDATA[key1]]></val></String></key>
                <value><Int><val>100</val></Int></value>
            </entry>
        </val>
    </Dict>
    "#;
    let decoded_map: PythonValue = from_str(xml_map).unwrap();
    assert_eq!(
        decoded_map,
        PythonValue::Dict {
            val: {
                let mut map = BTreeMap::new();
                map.insert(
                    PythonValueWeak(PythonValue::String {
                        val: "key1".to_string(),
                    }),
                    PythonValueWeak(PythonValue::Int { val: 100 }),
                );
                map
            },
        }
    );

    let xml_full = r#"
    <Dict>
        <val>
            <entry>
                <key><String><val><![CDATA[key1]]></val></String></key>
                <value><Int><val>100</val></Int></value>
            </entry>
            <entry>
                <key><String><val><![CDATA[key2]]></val></String></key>
                <value><List>
                    <val>
                        <item>
                            <Float><val>1.14</val></Float>
                        </item>
                        <item>
                            <Bool><val>true</val></Bool>
                        </item>
                    </val>
                </List></value>
            </entry>
        </val>
    </Dict>
    "#;

    let decoded: PythonValue = from_str(xml_full).unwrap();
    assert_eq!(
        decoded,
        PythonValue::Dict {
            val: {
                let mut map = BTreeMap::new();
                map.insert(
                    PythonValueWeak(PythonValue::String {
                        val: "key1".to_string(),
                    }),
                    PythonValueWeak(PythonValue::Int { val: 100 }),
                );
                map.insert(
                    PythonValueWeak(PythonValue::String {
                        val: "key2".to_string(),
                    }),
                    PythonValueWeak(PythonValue::List {
                        val: vec![
                            PythonValueWeak(PythonValue::Float {
                                val: OrderedFloat(1.14),
                            }),
                            PythonValueWeak(PythonValue::Bool { val: true }),
                        ],
                    }),
                );
                map
            },
        }
    );
}

#[llm_prompt]
#[derive(Deserialize, Debug, PartialEq)]
enum HashMapTest {
    #[prompt("A hashmap variant")]
    HashMapVariant {
        #[prompt("The hashmap value")]
        val: HashMap<String, i32>,
    },
}

#[test]
fn test_hashmap_deserialization() {
    let xml = r#"
    <HashMapVariant>
        <val>
            <entry>
                <key><![CDATA[key1]]></key>
                <value>100</value>
            </entry>
            <entry>
                <key><![CDATA[key2]]></key>
                <value>200</value>
            </entry>
        </val>
    </HashMapVariant>
    "#;
    let decoded: HashMapTest = from_str(xml).unwrap();
    let mut expected_map = HashMap::new();
    expected_map.insert("key1".to_string(), 100);
    expected_map.insert("key2".to_string(), 200);
    assert_eq!(decoded, HashMapTest::HashMapVariant { val: expected_map });
}
