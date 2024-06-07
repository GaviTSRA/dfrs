pub fn to_camel_case(s: &str) -> String {
    let mut words = s.split_whitespace();
    let mut camel_case_string = String::new();

    if let Some(first_word) = words.next() {
        camel_case_string.push_str(&first_word.to_lowercase());
    }

    for word in words {
        let mut chars = word.chars();
        if let Some(first_char) = chars.next() {
            camel_case_string.push(first_char.to_uppercase().next().unwrap());
            camel_case_string.push_str(&chars.as_str().to_lowercase());
        }
    }

    camel_case_string
}