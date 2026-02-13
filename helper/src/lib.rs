extern crate proc_macro;
use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
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
    let item_name = match &input {
        Item::Struct(s) => s.ident.to_string(),
        Item::Enum(e) => e.ident.to_string(),
        _ => {
            return quote! { compile_error!("llm_prompt only supports Struct and Enum"); }.into();
        }
    };
    let mut extra_functions = Vec::new();

    let mut extra_impls = Vec::new();

    match &mut input {
        Item::Struct(s) => {
            let name = &s.ident;
            let root_tag = name.to_string();
            let mut field_generators = Vec::new();

            if let Fields::Named(fields) = &mut s.fields {
                for field in &mut fields.named {
                    let field_quote = process_field(&item_name, None, field, &mut field_generators);
                    extra_functions.push(field_quote);
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
                        let field_quote =
                            process_field(&item_name, Some(&v_name), field, &mut f_parts);
                        extra_functions.push(field_quote);
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
                    fn root_name() -> &'static str { "" }
                }
            });
        }
        _ => return quote! { compile_error!("llm_prompt only supports Struct and Enum"); }.into(),
    }

    let result = quote! {
        #input
        #(#extra_impls)*
        #(#extra_functions)*
    };
    result.into()
}

fn process_field(
    item_name: &str,
    variant_name: Option<&str>,
    field: &mut Field,
    generators: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().expect("Only support named fields");
    let field_name = field_ident.to_string();
    let field_type = &field.ty;
    let mut extra_functions = Vec::new();

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
    let inner_field_name = if let Some(v) = variant_name {
        format!("{}_{}_{}", item_name, v, field_name)
    } else {
        format!("{}_{}", item_name, field_name)
    };
    if let (code, Some(parser_path)) = get_custom_parser(&inner_field_name, field_type) {
        let attr: syn::Attribute = if is_option(field_type) {
            parse_quote! { #[serde(deserialize_with = #parser_path, default)] }
        } else {
            parse_quote! { #[serde(deserialize_with = #parser_path)] }
        };
        field.attrs.push(attr);
        extra_functions.push(code);
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

    quote! {
        #(#extra_functions)*
    }
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

fn get_custom_parser(name: &str, ty: &Type) -> (proc_macro2::TokenStream, Option<String>) {
    let tp = if let Type::Path(p) = ty {
        p
    } else {
        return (quote! {}, None);
    };
    let segment = if let Some(segment) = tp.path.segments.last() {
        segment
    } else {
        return (quote! {}, None);
    };

    let mut extra_functions = Vec::new();
    let mut ret_function_name = None;

    let type_str = quote! { #ty }.to_string();
    let mut hasher = DefaultHasher::new();
    type_str.hash(&mut hasher);
    let type_hash = hasher.finish();

    match &segment.arguments {
        PathArguments::None => {
            ret_function_name = match segment.ident.to_string().as_str() {
                "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
                | "f32" | "f64" | "bool" => {
                    Some(format!("::llm_xml_caster::custom_{}_parser", segment.ident))
                }
                "String" => Some(format!("::llm_xml_caster::custom_string_parser")),
                _ => None,
            };
        }
        PathArguments::AngleBracketed(path) => match path.args.len() {
            1 => match segment.ident.to_string().as_str() {
                #[cfg(feature = "ordered_float")]
                "OrderedFloat" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        let inner_name = format!("_{}_{}_inner", type_hash, name);
                        let (inner_tokens, _) = get_custom_parser(&inner_name, inner_ty);

                        let parser_call = quote! { ::llm_xml_caster::OrderedFloatParser::<#inner_ty>::custom_ordered_float_parser };

                        let func_ident = format_ident!("{}", name);

                        let wrapper_function = quote! {
                            pub fn #func_ident<'de, D>(deserializer: D) -> Result<#ty, D::Error>
                            where
                                D: serde::Deserializer<'de>,
                            {
                                #parser_call(deserializer)
                            }
                        };
                        extra_functions.push(quote! {
                            #inner_tokens
                            #wrapper_function
                        });
                        ret_function_name = Some(func_ident.to_string());
                    }
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        let inner_name = format!("_{}_{}_inner", type_hash, name);
                        let (inner_tokens, _) = get_custom_parser(&inner_name, inner_ty);

                        let parser_call = quote! { ::llm_xml_caster::VecParser::<#inner_ty>::custom_vector_parser };

                        let func_ident = format_ident!("{}", name);

                        let wrapper_function = quote! {
                            pub fn #func_ident<'de, D>(deserializer: D) -> Result<#ty, D::Error>
                            where
                                D: serde::Deserializer<'de>,
                            {
                                #parser_call(deserializer)
                            }
                        };
                        extra_functions.push(quote! {
                            #inner_tokens
                            #wrapper_function
                        });
                        ret_function_name = Some(func_ident.to_string());
                    }
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        let inner_name = format!("_{}_{}_inner", type_hash, name);
                        let (inner_tokens, inner_parser) = get_custom_parser(&inner_name, inner_ty);

                        let func_ident = format_ident!("{}", name);

                        let wrapper_function = if let Some(inner_parser_path) = inner_parser {
                            quote! {
                                pub fn #func_ident<'de, D>(deserializer: D) -> Result<#ty, D::Error>
                                where
                                    D: serde::Deserializer<'de>,
                                {
                                    #[derive(serde::Deserialize)]
                                    struct OptionWrapper(#[serde(deserialize_with = #inner_parser_path)] #inner_ty);

                                    match OptionWrapper::deserialize(deserializer) {
                                        Ok(wrapper) => Ok(Some(wrapper.0)),
                                        Err(_) => Ok(None),
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub fn #func_ident<'de, D>(deserializer: D) -> Result<#ty, D::Error>
                                where
                                    D: serde::Deserializer<'de>,
                                {
                                    ::llm_xml_caster::OptionParser::<#inner_ty>::custom_option_parser(deserializer)
                                }
                            }
                        };

                        extra_functions.push(quote! {
                            #inner_tokens
                            #wrapper_function
                        });
                        ret_function_name = Some(func_ident.to_string());
                    }
                }
                _ => {}
            },
            2 => match segment.ident.to_string().as_str() {
                "BTreeMap" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        let mut args_iter = args.args.iter();
                        if let (
                            Some(GenericArgument::Type(key_ty)),
                            Some(GenericArgument::Type(val_ty)),
                        ) = (args_iter.next(), args_iter.next())
                        {
                            let key_name = format!("_{}_{}_key", type_hash, name);
                            let val_name = format!("_{}_{}_val", type_hash, name);

                            let (key_tokens, _) = get_custom_parser(&key_name, key_ty);
                            let (val_tokens, _) = get_custom_parser(&val_name, val_ty);

                            let parser_call = quote! { ::llm_xml_caster::BTreeMapParser::<#key_ty, #val_ty>::custom_btreemap_parser };

                            let func_ident = format_ident!("{}", name);

                            let wrapper_function = quote! {
                                pub fn #func_ident<'de, D>(deserializer: D) -> Result<#ty, D::Error>
                                where
                                    D: serde::Deserializer<'de>,
                                {
                                    #parser_call(deserializer)
                                }
                            };
                            extra_functions.push(quote! {
                                #key_tokens
                                #val_tokens
                                #wrapper_function
                            });
                            ret_function_name = Some(func_ident.to_string());
                        }
                    }
                }
                "HashMap" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        let mut args_iter = args.args.iter();
                        if let (
                            Some(GenericArgument::Type(key_ty)),
                            Some(GenericArgument::Type(val_ty)),
                        ) = (args_iter.next(), args_iter.next())
                        {
                            let key_name = format!("_{}_{}_key", type_hash, name);
                            let val_name = format!("_{}_{}_val", type_hash, name);

                            let (key_tokens, _) = get_custom_parser(&key_name, key_ty);
                            let (val_tokens, _) = get_custom_parser(&val_name, val_ty);

                            let parser_call = quote! { ::llm_xml_caster::HashMapParser::<#key_ty, #val_ty>::custom_hashmap_parser };

                            let func_ident = format_ident!("{}", name);

                            let wrapper_function = quote! {
                                pub fn #func_ident<'de, D>(deserializer: D) -> Result<#ty, D::Error>
                                where
                                    D: serde::Deserializer<'de>,
                                {
                                    #parser_call(deserializer)
                                }
                            };
                            extra_functions.push(quote! {
                                #key_tokens
                                #val_tokens
                                #wrapper_function
                            });
                            ret_function_name = Some(func_ident.to_string());
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }

    (
        quote! {
            #(#extra_functions)*
        },
        ret_function_name,
    )
}
