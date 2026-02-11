use super::r#macro::impl_llm_numeric_parser;

macro_rules! impl_llm_integer_parser {
    (
        $ty:ty
    ) => {
        impl_llm_numeric_parser!(
            $ty,
            "integer value, a whole number without a fractional part, e.g., 42, -7, or 0"
        );
    };
}

impl_llm_integer_parser!(i8);
impl_llm_integer_parser!(i16);
impl_llm_integer_parser!(i32);
impl_llm_integer_parser!(i64);
impl_llm_integer_parser!(i128);

impl_llm_integer_parser!(u8);
impl_llm_integer_parser!(u16);
impl_llm_integer_parser!(u32);
impl_llm_integer_parser!(u64);
impl_llm_integer_parser!(u128);
