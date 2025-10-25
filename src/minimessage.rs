use regex::Regex;

#[derive(Debug)]
enum Node {
  Text(String),
  Tag { name: String, children: Vec<Node> },
}

pub fn parse_minimessage(input: &str) -> String {
  let tag_re = Regex::new(r"</?([^<>/]+)>").unwrap();
  let mut nodes = Vec::new();
  let mut stack: Vec<(String, Vec<Node>)> = Vec::new();
  let mut last_index = 0;

  for cap in tag_re.captures_iter(input) {
    let full_match = cap.get(0).unwrap();
    let tag_name = cap.get(1).unwrap().as_str();
    let is_closing = full_match.as_str().starts_with("</");

    if full_match.start() > last_index {
      let text = &input[last_index..full_match.start()];
      let node = Node::Text(text.to_string());
      if let Some((_, children)) = stack.last_mut() {
        children.push(node);
      } else {
        nodes.push(node);
      }
    }

    if is_closing {
      if let Some((open_tag, children)) = stack.pop() {
        let node = Node::Tag {
          name: open_tag,
          children,
        };
        if let Some((_, parent_children)) = stack.last_mut() {
          parent_children.push(node);
        } else {
          nodes.push(node);
        }
      }
    } else {
      stack.push((tag_name.to_string(), Vec::new()));
    }

    last_index = full_match.end();
  }

  if last_index < input.len() {
    let text = &input[last_index..];
    let node = Node::Text(text.to_string());
    if let Some((_, children)) = stack.last_mut() {
      children.push(node);
    } else {
      nodes.push(node);
    }
  }

  while let Some((tag, children)) = stack.pop() {
    let node = Node::Tag {
      name: tag,
      children,
    };
    if let Some((_, parent_children)) = stack.last_mut() {
      parent_children.push(node);
    } else {
      nodes.push(node);
    }
  }

  let mut snbt_nodes: Vec<String> = nodes.iter().map(node_to_snbt).collect();
  if snbt_nodes.len() == 0 {
    snbt_nodes.push("{text:\"\"}".to_string());
  }
  let final_extra = snbt_nodes.join(",");
  format!("{{extra:[{final_extra}],italic:0b,text:\"\"}}")
}

fn collect_properties(node: &Node) -> Vec<String> {
  let mut props = Vec::new();

  fn apply_tag(name: &str, props: &mut Vec<String>) {
    match name {
      "bold" => props.push("bold:1b".into()),
      "italic" => props.push("italic:1b".into()),
      "underlined" => props.push("underlined:1b".into()),
      "strikethrough" => props.push("strikethrough:1b".into()),
      "obfuscated" => props.push("obfuscated:1b".into()),
      // TODO: non-minimessage colors should not be parsed as a tag
      c => props.push(format!("color:\"{}\"", c)),
    }
  }

  let mut current = node;
  loop {
    match current {
      Node::Tag { name, children } => {
        apply_tag(name, &mut props);
        if children.len() == 1 {
          if let Node::Tag { .. } = &children[0] {
            current = &children[0];
            continue;
          }
        }
      }
      _ => {}
    }
    break;
  }

  props
}

fn node_to_snbt(node: &Node) -> String {
  match node {
    Node::Text(t) => format!("\"{}\"", t),
    Node::Tag { children, .. } => {
      let mut props = collect_properties(node);

      if let Some(text) = extract_single_text(children) {
        props.push(format!("text:\"{}\"", text));
        return format!("{{{}}}", props.join(","));
      }

      if !children.is_empty() {
        let extras: Vec<String> = children.iter().map(node_to_snbt).collect();
        props.push(format!("extra:[{}]", extras.join(",")));
      }

      props.push(String::from("text:\"\""));
      format!("{{{}}}", props.join(","))
    }
  }
}

fn extract_single_text(children: &[Node]) -> Option<String> {
  if children.len() == 1 {
    match &children[0] {
      Node::Text(t) => Some(t.clone()),
      Node::Tag { children, .. } => extract_single_text(children),
    }
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn empty_text() {
    assert_eq!(
      parse_minimessage(""),
      "{extra:[{text:\"\"}],italic:0b,text:\"\"}",
    )
  }

  #[test]
  pub fn unformatted_text() {
    let test_values = vec![
      "A",
      "Abc",
      "Test value",
      "This is a test-value",
      "Newline\nchar",
    ];

    for value in test_values {
      assert_eq!(
        parse_minimessage(value),
        format!("{{extra:[\"{value}\"],italic:0b,text:\"\"}}")
      );
    }
  }

  #[test]
  pub fn single_color() {
    assert_eq!(
      parse_minimessage("<red>a"),
      "{extra:[{color:\"red\",text:\"a\"}],italic:0b,text:\"\"}"
    )
  }

  #[test]
  pub fn closed_color() {
    assert_eq!(
      parse_minimessage("<red>a</red>b"),
      "{extra:[{color:\"red\",text:\"a\"},\"b\"],italic:0b,text:\"\"}"
    )
  }

  #[test]
  pub fn bold_and_color() {
    assert_eq!(
      parse_minimessage("<red><bold>a"),
      "{extra:[{color:\"red\",bold:1b,text:\"a\"}],italic:0b,text:\"\"}"
    )
  }

  #[test]
  pub fn bold_after_color() {
    assert_eq!(
      parse_minimessage("<red>a<bold>b"),
      "{extra:[{color:\"red\",extra:[\"a\",{bold:1b,text:\"b\"}],text:\"\"}],italic:0b,text:\"\"}",
    )
  }

  #[test]
  pub fn underlined() {
    assert_eq!(
      parse_minimessage("<underlined>a"),
      "{extra:[{underlined:1b,text:\"a\"}],italic:0b,text:\"\"}",
    )
  }

  #[test]
  pub fn italic() {
    assert_eq!(
      parse_minimessage("<italic>a"),
      "{extra:[{italic:1b,text:\"a\"}],italic:0b,text:\"\"}",
    )
  }

  #[test]
  pub fn strikethrough() {
    assert_eq!(
      parse_minimessage("<strikethrough>a"),
      "{extra:[{strikethrough:1b,text:\"a\"}],italic:0b,text:\"\"}",
    )
  }

  #[test]
  pub fn obfuscated() {
    assert_eq!(
      parse_minimessage("<obfuscated>a"),
      "{extra:[{obfuscated:1b,text:\"a\"}],italic:0b,text:\"\"}",
    )
  }

  #[test]
  pub fn rgb_color() {
    assert_eq!(
      parse_minimessage("<#FF00FF>a"),
      "{extra:[{color:\"#FF00FF\",text:\"a\"}],italic:0b,text:\"\"}",
    )
  }
}
