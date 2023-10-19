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
