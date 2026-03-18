use std::collections::HashMap;

pub fn format_command<A: AsRef<str>, B: AsRef<str>, C: AsRef<str>>(text: A, to_replace: &HashMap<B, C>) -> String {
    let mut text = text.as_ref().to_string();

    for (key, value) in to_replace {
        text = text.replace(key.as_ref(), value.as_ref());
    }

    text
}