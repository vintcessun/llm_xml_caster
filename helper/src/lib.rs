extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Field, Fields, GenericArgument, Item, PathArguments, Type, parse_macro_input, parse_quote,
};

/// The main procedural macro for `llm_xml_caster`.
///
/// This attribute macro derives the `LlmPrompt` trait implementation for the decorated
/// struct or enum. It enables automatic XML schema generation and inserts necessary
/// `serde` attributes for custom deserialization logic, especially for handling boolean
/// and numerical types parsed from XML text nodes.
///
/// Use `#[prompt("Description")]` on struct fields or enum variants to provide guidance
/// for the Large Language Model.
#[proc_macro_attribute]
pub fn llm_prompt(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as Item);

    let mut extra_impls = Vec::new();

    match &mut input {
        Item::Struct(s) => {
            let name = &s.ident;
            let root_tag = name.to_string();
            let mut field_generators = Vec::new();

            if let Fields::Named(fields) = &mut s.fields {
                for field in &mut fields.named {
                    process_field(field, &mut field_generators);
                }
            }

            extra_impls.push(quote! {
                impl ::llm_xml_caster::LlmPrompt for #name {
                    fn get_prompt_schema() -> &'static str {
                        use std::sync::OnceLock;
                        static SCHEMA_CACHE: OnceLock<String> = OnceLock::new();
                        SCHEMA_CACHE.get_or_init(|| {
                            let mut parts = Vec::new();
                            #( parts.push(#field_generators); )*
                            format!("<{root}>\n  {inner}\n</{root}>",
                                root = #root_tag, inner = parts.join("\n  "))
                        })
                    }
                    fn root_name() -> &'static str { #root_tag }
                }
            });
        }
        Item::Enum(e) => {
            let name = &e.ident;
            let mut variants_schemas = Vec::new();

            for variant in &mut e.variants {
                let v_ident = &variant.ident;
                let v_name = v_ident.to_string();

                // Extract variant description
                let mut v_desc = String::new();
                for attr in &variant.attrs {
                    if attr.path().is_ident("prompt")
                        && let Ok(lit) = attr.parse_args::<syn::LitStr>()
                    {
                        v_desc = lit.value();
                    }
                }

                // Remove #[prompt] from variant attributes
                variant.attrs.retain(|attr| !attr.path().is_ident("prompt"));

                let mut f_parts = Vec::new();
                if let Fields::Named(fields) = &mut variant.fields {
                    for field in &mut fields.named {
                        process_field(field, &mut f_parts);
                    }
                }

                let fields_prompt_quote = if f_parts.is_empty() {
                    quote! { String::new() }
                } else {
                    quote! { vec![#(#f_parts),*].join("\n") }
                };

                variants_schemas.push(quote! {
                    {
                        let inner_xml = #fields_prompt_quote;
                        let desc = #v_desc;
                        if inner_xml.is_empty() {
                            format!("<{name}/> <!-- {desc} -->", name = #v_name, desc = desc)
                        } else {
                            let indented_inner = inner_xml.lines()
                                .map(|line| format!("  {}", line))
                                .collect::<Vec<_>>()
                                .join("\n");
                            format!("<{name}>\n{inner}\n</{name}> <!-- {desc} -->",
                                name = #v_name, inner = indented_inner, desc = desc)
                        }
                    }
                });
            }

            extra_impls.push(quote! {
                impl ::llm_xml_caster::LlmPrompt for #name {
                    fn get_prompt_schema() -> &'static str {
                        use std::sync::OnceLock;
                        static SCHEMA_CACHE: OnceLock<String> = OnceLock::new();
                        SCHEMA_CACHE.get_or_init(|| {
                            let mut parts = vec!["The following are possible XML structures for the current enum type:".to_string()];
                            #( parts.push(#variants_schemas); )*
                            parts.join("\n")
                        })
                    }
                    fn root_name() -> &'static str { "XML_ENUM_ROOT" }
                }
            });
        }
        _ => return quote! { compile_error!("llm_prompt only supports Struct and Enum"); }.into(),
    }

    let result = quote! {
        #input
        #(#extra_impls)*
    };
    result.into()
}

fn process_field(field: &mut Field, generators: &mut Vec<proc_macro2::TokenStream>) {
    let field_ident = field.ident.as_ref().expect("Only support named fields");
    let field_name = field_ident.to_string();
    let field_type = &field.ty;

    // Extract #[prompt("...")]
    let mut user_description = None;
    for attr in &field.attrs {
        if attr.path().is_ident("prompt")
            && let Ok(lit) = attr.parse_args::<syn::LitStr>()
        {
            user_description = Some(lit);
        }
    }

    let user_description_quote = match user_description {
        Some(desc) => quote! { #desc },
        None => quote! { "" }, // Should probably be a compile error if we want strictness
    };

    // Auto-generate #[serde(deserialize_with = "...")]
    if let Some(parser_path) = get_custom_parser(field_type) {
        let attr: syn::Attribute = if is_option(field_type) {
            parse_quote! { #[serde(deserialize_with = #parser_path, default)] }
        } else {
            parse_quote! { #[serde(deserialize_with = #parser_path)] }
        };
        field.attrs.push(attr);
    }

    generators.push(quote! {
        {
            let sub_schema = <#field_type as ::llm_xml_caster::LlmPrompt>::get_prompt_schema();
            let description = #user_description_quote;
            let indented_schema = sub_schema.lines()
                .map(|line| format!("  {}", line))
                .collect::<Vec<_>>()
                .join("\n");
            format!("<{name}>\n{schema}\n</{name}> <!-- {desc} -->",
                name = #field_name, schema = indented_schema, desc = description)
        }
    });

    // Remove #[prompt] from the field attributes so it doesn't cause a compile error
    field.attrs.retain(|attr| !attr.path().is_ident("prompt"));
}

fn is_option(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(segment) = p.path.segments.last()
    {
        let ident = segment.ident.to_string();
        return ident == "Option";
    }

    false
}

fn get_custom_parser(ty: &Type) -> Option<String> {
    let tp = if let Type::Path(p) = ty {
        p
    } else {
        return None;
    };
    let segment = tp.path.segments.last()?;
    let ident = segment.ident.to_string();

    match ident.as_str() {
        "bool" => Some("::llm_xml_caster::custom_bool_parser".to_string()),
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32"
        | "f64" => Some(format!("::llm_xml_caster::custom_{}_parser", ident)),
        "Vec" => {
            if let PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(GenericArgument::Type(inner)) = args.args.first()
            {
                // Re-parsing inner to get clean string
                let inner_str = quote!(#inner).to_string().replace(" ", "");
                return Some(format!(
                    "::llm_xml_caster::VecParser::<{}>::custom_vector_parser",
                    inner_str
                ));
            }
            None
        }
        "Option" => {
            if let PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(GenericArgument::Type(inner)) = args.args.first()
            {
                let inner_str = quote!(#inner).to_string().replace(" ", "");
                return Some(format!(
                    "::llm_xml_caster::OptionParser::<{}>::custom_option_parser",
                    inner_str
                ));
            }
            None
        }
        _ => None,
    }
}
