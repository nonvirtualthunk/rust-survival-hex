use prelude::*;


pub fn prefix_with_indefinite_article<S : Into<String>>(string : S) -> String {
    let string = string.into();
    if let Some(first_char) = string.chars().next() {
        match first_char.to_ascii_lowercase() {
            'a' | 'e' | 'i' | 'o' | 'u' | 'y' => format!("an {}", string),
            _ => format!("a {}", string),
        }
    } else { string }
}