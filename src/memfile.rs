use std::num::{IntErrorKind, ParseIntError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemfileError {
    kind: MemfileErrorKind,
    line: usize,
}
impl MemfileError {
    pub fn new(line: usize, kind: MemfileErrorKind) -> Self {
        Self { line, kind }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemfileErrorKind {
    InvalidDigit(String),
    OutOfRangeInteger(String),
    MemoryOverflow,
}
impl std::fmt::Display for MemfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            MemfileErrorKind::InvalidDigit(x) => {
                write!(f, "invalid number in line {}: {x}", self.line)
            }
            MemfileErrorKind::OutOfRangeInteger(x) => {
                write!(f, "out of range integer in line {}: {x}", self.line)
            }
            MemfileErrorKind::MemoryOverflow => {
                write!(f, "Memory cursor overflow")
            }
        }
    }
}

enum ParserState {
    Org,
    Normal,
}

/// Parses a memory file in the following format:
/// A sequence of tokens, being one of:
/// - byte: A number in decimal (positive or negative) or hexadecimal,
///         that will be inserted at the memory cursor position.
/// - ORG byte: Changes the memory cursor to this position.
pub fn parse_memfile(mem: &mut [u8], source: &str) -> Result<(), MemfileError> {
    let filtered = remove_comments(source);
    let source = &filtered;
    let mut mem_cursor = 0;
    let mut stt = ParserState::Normal;
    let words = source.split_whitespace();
    for word in words {
        if mem_cursor == 256 {
            return Err(err(source, word, MemfileErrorKind::MemoryOverflow));
        }
        match stt {
            ParserState::Normal if parse_org(word) => {
                stt = ParserState::Org;
            }
            ParserState::Normal => {
                mem[mem_cursor] = parse_byte(word).map_err(|e| err(source, word, e))?;
                mem_cursor += 1;
            }
            ParserState::Org => {
                mem_cursor = parse_byte(word).map_err(|e| err(source, word, e))? as usize;
                stt = ParserState::Normal;
            }
        }
    }
    Ok(())
}
fn err(source: &str, word: &str, kind: MemfileErrorKind) -> MemfileError {
    let offset = word.as_ptr() as usize - source.as_ptr() as usize;
    let line = source[..offset].chars().filter(|c| *c == '\n').count() + 1;
    MemfileError::new(line, kind)
}

fn parse_org(token: &str) -> bool {
    token == "org" || token == "ORG"
}

fn parse_byte(token: &str) -> Result<u8, MemfileErrorKind> {
    if token.starts_with("0x") {
        u8::from_str_radix(&token[2..], 16).map_err(|e| parse_int_err(e, token))
    } else if token.starts_with('-') {
        i8::from_str_radix(token, 10)
            .map(|x| x as u8)
            .map_err(|e| parse_int_err(e, token))
    } else {
        u8::from_str_radix(token, 10).map_err(|e| parse_int_err(e, token))
    }
}
fn parse_int_err(e: ParseIntError, token: &str) -> MemfileErrorKind {
    match e.kind() {
        IntErrorKind::Empty | IntErrorKind::InvalidDigit => {
            MemfileErrorKind::InvalidDigit(token.to_string())
        }
        IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => {
            MemfileErrorKind::OutOfRangeInteger(token.to_string())
        }
        _ => unreachable!(),
    }
}
fn remove_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut comment = false;
    for c in source.chars() {
        if c == ';' {
            comment = true;
        } else if c == '\n' {
            comment = false;
        }
        if !comment {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memfile_parsing() {
        let mut mem = [0_u8; 256];
        let source = r#"
        1 2 3
        0x4 0x5 0x6
        0xff -10 0
        org 20
        7 8 9
        "#;
        let res = parse_memfile(&mut mem, source);
        assert_eq!(res, Ok(()));
        assert_eq!(&mem[0..9], [1, 2, 3, 4, 5, 6, 255, 246, 0]);
        assert_eq!(&mem[20..23], [7, 8, 9]);
    }
    #[test]
    fn test_commented() {
        let src = "abc; 123; 45\ndef";
        assert_eq!(remove_comments(src), "abc\ndef");
    }
}
