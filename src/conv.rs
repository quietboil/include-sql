//! Identifires case converions

pub(super) fn to_camel_case(in_name: &str) -> String {
    let mut out_name = String::with_capacity(in_name.len());
    for name_fragment in in_name.split('_') {
        let mut chars = name_fragment.chars();
        if let Some( first ) = chars.next() {
            out_name.push(first.to_ascii_uppercase());
            for c in chars {
                out_name.push(c);
            }
        }
    }
    out_name
}

pub(super) fn to_snake_case(in_name: &str) -> String {
    if in_name.is_empty() {
        return in_name.to_string();
    }
    let mut out_name = String::with_capacity(in_name.len() + in_name.len() / 4);
    let mut chars = in_name.chars();
    let mut prev = '_';
    let mut curr = '_';
    while let Some( next ) = chars.next() {
        if next != '_' {
            curr = next;
            break;
        }
        // skipping leading underscores otheriwse
    }
    while let Some( next ) = chars.next() {
        if curr == '_' && prev == '_' {
            // skipping consecutive underscores
            continue;
        }
        if curr.is_ascii_uppercase() {
            if prev != '_' {
                if !prev.is_ascii_uppercase() || next != '_' && !next.is_ascii_uppercase() {
                    out_name.push('_');
                }
            }
            out_name.push(curr.to_ascii_lowercase());
        } else {
            out_name.push(curr);
        }
        prev = curr;
        curr = next;
    }
    if curr.is_ascii_uppercase() {
        if prev != '_' && !prev.is_ascii_uppercase() {
            out_name.push('_');
        }
        out_name.push(curr.to_ascii_lowercase());
    } else if curr != '_' {
        out_name.push(curr);
    } else if prev == '_' {
        out_name.truncate(out_name.len() - 1);
    }
    out_name
}

#[cfg(test)]
mod tests {

    #[test]
    fn to_camel_case() {
        use super::to_camel_case;

        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_camel_case("snake_case_name"), "SnakeCaseName");
        assert_eq!(to_camel_case("AlreadyCamelCase"), "AlreadyCamelCase");
        assert_eq!(to_camel_case("partialCamelCase"), "PartialCamelCase");
        assert_eq!(to_camel_case("mixedCase001_of_a_name_WithATail"), "MixedCase001OfANameWithATail");
    }

    #[test]
    fn to_snake_case() {
        use super::to_snake_case;

        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_snake_case("already_snake_case_name"), "already_snake_case_name");
        assert_eq!(to_snake_case("__snake_with_prefix"), "snake_with_prefix");
        assert_eq!(to_snake_case("TypicalCamelCaseName"), "typical_camel_case_name");
        assert_eq!(to_snake_case("mixed52Case_002_WithATail"), "mixed52_case_002_with_a_tail");
        assert_eq!(to_snake_case("UPPER_CASE_SNAKE"), "upper_case_snake");
        assert_eq!(to_snake_case("snake2_snake2_case"), "snake2_snake2_case");
        assert_eq!(to_snake_case("getHTTPResponseCode"), "get_http_response_code");
        assert_eq!(to_snake_case("HTTPResponseCodeXYZ"), "http_response_code_xyz");
        assert_eq!(to_snake_case("___NameWithLeadingAndTrailingUnderscores___"), "name_with_leading_and_trailing_underscores");
    }
}

