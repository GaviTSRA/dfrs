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

pub fn to_dfrs_name(s: &str) -> String {
    let mut replaced: String = s.trim().to_string();
    replaced = replaced.replace("+=", "addDirect").replace("-=", "subDirect").replace("<=", "lessEqual").replace(">=", "greaterEqual")
        .replace(">", "greater").replace("<", "less").replace("!=", "notEqual")
        .replace('+', "add").replace('-', "sub").replace('%', "mod").replace('/', "div").replace('=', "equal").replace(" ", "");

    if replaced == *"x" {
        replaced = "mul".into();
    }

    let v = replaced.trim().to_owned();
    let mut vv: Vec<char> = v.chars().collect();
    vv[0] = vv[0].to_lowercase().next().unwrap();
    let name: String = vv.into_iter().collect();
    name
}