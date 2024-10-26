#![allow(dead_code)]

mod classes;
mod schedule;
mod transcript;

pub mod client;
pub use transcript::Transcript;

#[macro_export]
macro_rules! selector {
    ( $text:literal ) => {{
        static SEL: ::std::sync::LazyLock<::scraper::Selector> =
            ::std::sync::LazyLock::new(|| ::scraper::Selector::parse($text).unwrap());
        &*SEL
    }};
}
