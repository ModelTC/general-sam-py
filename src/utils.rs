use std::fmt::Display;

pub use either::{
    for_both, Either as CharOrByte,
    {Either::Left as CharSide, Either::Right as ByteSide},
};

#[macro_export]
macro_rules! char_or_byte_type {
    ($( $name:ident )::+ $(; $( $life:lifetime ),* )?) => {
        $crate::utils::CharOrByte<
            $($name)::+ < $( $($life),* ,)? char>,
            $($name)::+ < $( $($life),* ,)? u8>,
        >
    };
}

pub use char_or_byte_type;

#[derive(Debug, Clone)]
pub struct InconsistentCharOrByte;

impl Display for InconsistentCharOrByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inconsistent char or byte as the backends of the types")
    }
}

#[macro_export]
macro_rules! for_both_and_wrap {
    ($($value:expr),+; ($($pattern:pat),+) => $result:expr) => {
        match ($($value),+) {
            ($($crate::utils::CharSide($pattern)),+) => Ok($crate::utils::CharSide($result)),
            ($($crate::utils::ByteSide($pattern)),+) => Ok($crate::utils::ByteSide($result)),
            _ => Err($crate::utils::InconsistentCharOrByte),
        }
    }
}

pub use for_both_and_wrap;

#[macro_export]
macro_rules! for_both_with_side {
    ($value:expr, $side:ident, $pattern:pat => $result:expr) => {
        match $value {
            $crate::utils::CharSide($pattern) => {
                let $side = |x| $crate::utils::CharSide(x);
                $result
            }
            $crate::utils::ByteSide($pattern) => {
                let $side = |x| $crate::utils::ByteSide(x);
                $result
            }
        }
    };
}

pub use for_both_with_side;

pub fn get_char_or_byte_variant_name<L, R>(e: &CharOrByte<L, R>) -> &str {
    match e {
        CharSide(_) => "char",
        ByteSide(_) => "byte",
    }
}
