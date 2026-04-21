fn is_valid_escape(c: char) -> bool {
    matches!(c, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u')
}

pub fn sanitize_json(raw_input: &str) -> String {
    let mut result = String::with_capacity(raw_input.len());
    let mut in_string = false;
    let mut escape_next = false;

    for c in raw_input.chars() {
        if in_string {
            if escape_next {
                escape_next = false;
                if is_valid_escape(c) {
                    result.push(c);
                } else {
                    result.push_str("\\");
                    if c < '\u{0020}' {
                        escape_control(&mut result, c);
                    } else {
                        result.push(c);
                    }
                }
                continue;
            }

            if c == '\\' {
                escape_next = true;
                result.push(c);
                continue;
            }

            if c == '"' {
                in_string = false;
                result.push(c);
                continue;
            }

            if c < '\u{0020}' {
                escape_control(&mut result, c);
                continue;
            }

            result.push(c);
        } else {
            if c == '"' {
                in_string = true;
            }
            result.push(c);
        }
    }
    result
}

#[inline]
fn escape_control(result: &mut String, c: char) {
    match c {
        '\n' => result.push_str("\\n"),
        '\r' => result.push_str("\\r"),
        '\t' => result.push_str("\\t"),
        '\x08' => result.push_str("\\b"),
        '\x0C' => result.push_str("\\f"),
        _ => result.push_str(&format!("\\u{:04x}", c as u32)),
    }
}
