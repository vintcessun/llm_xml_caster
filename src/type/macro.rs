macro_rules! impl_llm_numeric_parser {
    (
        $ty:ty,
        $prompt:expr
    ) => {
        paste::paste! {
            pub fn [<custom_ $ty _parser>]<'de, D>(deserializer: D) -> Result<$ty, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::{self, Visitor};
                use std::fmt;

                struct MyVisitor;

                impl<'de> Visitor<'de> for MyVisitor {
                    type Value = $ty;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(const_format::formatcp!("a {} value or a string representing a {}", stringify!($ty), stringify!($ty)))
                    }

                    fn visit_i8<E>(self, v: i8) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_i16<E>(self, v: i16) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_i32<E>(self, v: i32) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_i64<E>(self, v: i64) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_i128<E>(self, v: i128) -> Result<$ty, E> { Ok(v as $ty) }

                    fn visit_u8<E>(self, v: u8) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_u16<E>(self, v: u16) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_u32<E>(self, v: u32) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_u64<E>(self, v: u64) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_u128<E>(self, v: u128) -> Result<$ty, E> { Ok(v as $ty) }

                    fn visit_f32<E>(self, v: f32) -> Result<$ty, E> { Ok(v as $ty) }
                    fn visit_f64<E>(self, v: f64) -> Result<$ty, E> { Ok(v as $ty) }

                    fn visit_str<E>(self, v: &str) -> Result<$ty, E>
                    where
                        E: de::Error,
                    {
                        let val_str = v.trim();
                        lexical_core::parse::<$ty>(val_str.as_bytes())
                            .map_err(|_| de::Error::custom(format!("can not parse '{}' as a {} value", v, stringify!($ty))))
                    }
                }

                deserializer.[<deserialize_ $ty>](MyVisitor)
            }

            impl crate::LlmPrompt for $ty {
                fn get_prompt_schema() -> &'static str {
                    $prompt
                }

                fn root_name() -> &'static str {
                    stringify!($ty)
                }

                const IS_ENUM: bool = false;
            }
        }
    };
}

pub(crate) use impl_llm_numeric_parser;
