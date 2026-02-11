use super::r#macro::impl_llm_numeric_parser;

macro_rules! impl_llm_float_parser {
    (
        $ty:ty
    ) => {
        impl_llm_numeric_parser!(
            $ty,
            "float value, a number that can have a fractional part, e.g., 3.14, -0.001, or 2.0"
        );
    };
}

impl_llm_float_parser!(f32);
impl_llm_float_parser!(f64);
