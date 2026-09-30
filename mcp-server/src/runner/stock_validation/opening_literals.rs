//! Opening's bounded literal decoding, retaining the stock document-link fence.
use super::*;
use std::borrow::Cow;
use std::iter::Peekable;
use std::str::CharIndices;

/// Consume a single/double/triple quoted literal. Borrow ordinary text; only
/// interpreted escapes require a buffer. Unknown escape/raw forms are unavailable.
pub(super) fn quoted<'a>(
    source: &'a str,
    start: usize,
    quote: char,
    chars: &mut Peekable<CharIndices<'a>>,
    interpret: bool,
) -> Result<Cow<'a, str>, &'static str> {
    if start > 0 && source.as_bytes()[start - 1] == b'r' {
        // Raw-prefixed strings need a different escape interpretation. Never
        // apply ordinary decoding and probe the wrong path instead.
        return Err("ambiguous_literal");
    }
    let mut triple = false;
    if chars.peek().is_some_and(|(_, c)| *c == quote) {
        chars.next();
        if chars.peek().is_some_and(|(_, c)| *c == quote) {
            chars.next();
            triple = true;
        } else {
            return Ok(Cow::Borrowed(""));
        }
    }
    let content_start = chars.peek().map_or(source.len(), |(offset, _)| *offset);
    let mut copied_from = content_start;
    let mut decoded: Option<String> = None;
    while let Some((offset, character)) = chars.next() {
        if character == '\\' {
            let (_, escape) = chars.next().ok_or("ambiguous_literal")?;
            let interpreted = match escape {
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                'a' => '\u{7}',
                'b' => '\u{8}',
                'f' => '\u{c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\u{b}',
                'u' => {
                    let mut code = 0u32;
                    for _ in 0..4 {
                        let (_, digit) = chars.next().ok_or("ambiguous_literal")?;
                        code = code * 16 + digit.to_digit(16).ok_or("ambiguous_literal")?;
                    }
                    char::from_u32(code).ok_or("ambiguous_literal")?
                }
                _ => return Err("ambiguous_literal"),
            };
            if interpret {
                let buffer = decoded.get_or_insert_with(String::new);
                buffer.push_str(&source[copied_from..offset]);
                buffer.push(interpreted);
                copied_from = chars.peek().map_or(source.len(), |(offset, _)| *offset);
            }
        } else if character == quote {
            if triple {
                let mut lookahead = chars.clone();
                if !lookahead.next().is_some_and(|(_, c)| c == quote)
                    || !lookahead.next().is_some_and(|(_, c)| c == quote)
                {
                    continue;
                }
                *chars = lookahead;
            }
            return Ok(match decoded {
                Some(mut buffer) => {
                    buffer.push_str(&source[copied_from..offset]);
                    Cow::Owned(buffer)
                }
                None => Cow::Borrowed(&source[content_start..offset]),
            });
        } else if !triple && character == '\n' {
            return Err("ambiguous_literal");
        }
    }
    Err("ambiguous_literal")
}

pub(super) fn fence(
    request: &WireRequest,
    root: &ProjectRoot,
    source: Option<&str>,
    deadline_at: Instant,
) -> Result<(), &'static str> {
    if request.purpose != Purpose::OpenContext {
        return Ok(());
    }
    let source = source.ok_or("opening_context_unavailable")?;
    let mut chars = source.char_indices().peekable();
    while let Some((start, character)) = chars.next() {
        if character == '#' {
            for (_, character) in chars.by_ref() {
                if character == '\n' {
                    break;
                }
            }
        } else if character == '\'' || character == '"' {
            deadline(deadline_at)?;
            let literal = quoted(source, start, character, &mut chars, true)?;
            if !literal.is_empty() {
                let path = admission::literal_path(&request.root_path, &literal)?;
                admission::probe_literal(root, &path)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(source: &str) -> Result<Cow<'_, str>, &'static str> {
        let mut chars = source.char_indices().peekable();
        let (start, quote) = chars.next().unwrap();
        quoted(source, start, quote, &mut chars, true)
    }

    #[test]
    fn interpreted_escapes_and_triples_cannot_hide_absolute_or_parent_paths() {
        for (literal, interpreted) in [
            ("\"\\u002fprivate/outside.gd\"", "/private/outside.gd"),
            ("'''../../outside.gd'''", "../../outside.gd"),
            (
                "\"\\u002e\\u002e/\\u002e\\u002e/outside.gd\"",
                "../../outside.gd",
            ),
            ("\"quoted\\\"name\"", "quoted\"name"),
        ] {
            assert_eq!(decode(literal).unwrap(), interpreted);
        }
        assert!(decode("\"unknown\\q/path\"").is_err());
        assert!(decode("\"\\ud800\"").is_err());
        assert!(decode("\"unterminated").is_err());
    }
}
