use serde::de;
use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Unexpected, Visitor};
use serde_derive::*;
use std::fmt;


// NOTE: Only intended to handle conversion from ASCII CamelCase to SnakeCase
//   This function is used to convert static Rust identifiers to snakecase
// TODO: pub(crate) fn
pub fn to_snakecase(camel: &str) -> String {
    let mut i = 0;
    let mut snake = String::new();
    let mut chars = camel.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            if i > 0 && !chars.peek().unwrap_or(&'A').is_uppercase() {
                snake.push('_');
            }
            snake.push(ch.to_lowercase().next().unwrap_or(ch));
        } else {
            snake.push(ch);
        }
        i += 1;
    }

    snake
}

//// Provides typings for match expressions
/// match typing<AuthInfo>(auth_info) {
///     Err(e) => Err(e)
///     Ok(auth_info) => {
///         // auth_info is now typed, gives autcompletion
///     }
/// }
pub fn typing<T>(t: T) -> T {
    t
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_snakecase() {
        use super::to_snakecase;

        assert_eq!(to_snakecase("snake_case").as_str(), "snake_case");
        assert_eq!(to_snakecase("CamelCase").as_str(), "camel_case");
        assert_eq!(to_snakecase("XMLHttpRequest").as_str(), "xml_http_request");
        assert_eq!(to_snakecase("UPPER").as_str(), "upper");
        assert_eq!(to_snakecase("lower").as_str(), "lower");
    }

}
