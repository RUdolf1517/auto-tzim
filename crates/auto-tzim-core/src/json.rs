//! A bounded JSON reader for the untrusted capability document.
//! Duplicate object keys are rejected, rather than choosing an ambiguous value.

use std::collections::BTreeMap;

pub const MAX_BYTES: usize = 1024 * 1024;
const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl Value {
    pub fn member(&self, key: &str) -> Result<&Value, Error> {
        match self {
            Value::Object(fields) => fields
                .get(key)
                .ok_or_else(|| Error(format!("missing key: {key}"))),
            _ => Err(Error("expected object".into())),
        }
    }

    pub fn integer(&self, key: &str) -> Result<u64, Error> {
        match self.member(key)? {
            Value::Number(n) if !n.contains(['.', 'e', 'E', '-']) => n
                .parse()
                .map_err(|_| Error(format!("key {key}: integer out of range"))),
            _ => Err(Error(format!("key {key}: expected integer"))),
        }
    }
}

pub fn parse(bytes: &[u8]) -> Result<Value, Error> {
    if bytes.len() > MAX_BYTES {
        return Err(Error("JSON exceeds 1 MiB".into()));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Error("invalid UTF-8".into()))?;
    let mut parser = Parser { text, pos: 0 };
    let value = parser.value(0)?;
    parser.whitespace();
    if parser.pos != text.len() {
        return Err(parser.error("trailing input"));
    }
    Ok(value)
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, message: &str) -> Error {
        Error(format!("JSON at byte {}: {message}", self.pos))
    }
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }
    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }
    fn take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn value(&mut self, depth: usize) -> Result<Value, Error> {
        self.whitespace();
        match self.peek() {
            Some(b'n') => {
                self.literal("null")?;
                Ok(Value::Null)
            }
            Some(b't') => {
                self.literal("true")?;
                Ok(Value::Bool(true))
            }
            Some(b'f') => {
                self.literal("false")?;
                Ok(Value::Bool(false))
            }
            Some(b'"') => self.string().map(Value::String),
            Some(b'[' | b'{') if depth + 1 >= MAX_DEPTH => {
                Err(self.error("nesting limit exceeded"))
            }
            Some(b'[') => self.array(depth + 1),
            Some(b'{') => self.object(depth + 1),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.error("expected value")),
        }
    }
    fn literal(&mut self, literal: &str) -> Result<(), Error> {
        if self.text[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(self.error("invalid literal"))
        }
    }
    fn array(&mut self, depth: usize) -> Result<Value, Error> {
        self.pos += 1;
        let mut values = Vec::new();
        self.whitespace();
        if self.take(b']') {
            return Ok(Value::Array(values));
        }
        loop {
            values.push(self.value(depth)?);
            self.whitespace();
            if self.take(b']') {
                return Ok(Value::Array(values));
            }
            if !self.take(b',') {
                return Err(self.error("expected comma or ]"));
            }
        }
    }
    fn object(&mut self, depth: usize) -> Result<Value, Error> {
        self.pos += 1;
        let mut fields = BTreeMap::new();
        self.whitespace();
        if self.take(b'}') {
            return Ok(Value::Object(fields));
        }
        loop {
            self.whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.error("expected object key"));
            }
            let key = self.string()?;
            self.whitespace();
            if !self.take(b':') {
                return Err(self.error("expected colon"));
            }
            let value = self.value(depth)?;
            if fields.insert(key, value).is_some() {
                return Err(self.error("duplicate object key"));
            }
            self.whitespace();
            if self.take(b'}') {
                return Ok(Value::Object(fields));
            }
            if !self.take(b',') {
                return Err(self.error("expected comma or }"));
            }
        }
    }
    fn string(&mut self) -> Result<String, Error> {
        self.pos += 1;
        let mut result = String::new();
        loop {
            match self.peek() {
                None => return Err(self.error("unterminated string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(result);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    let escaped = self.peek().ok_or_else(|| self.error("incomplete escape"))?;
                    self.pos += 1;
                    match escaped {
                        b'"' => result.push('"'),
                        b'\\' => result.push('\\'),
                        b'/' => result.push('/'),
                        b'b' => result.push('\u{08}'),
                        b'f' => result.push('\u{0c}'),
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        b'u' => {
                            let first = self.hex4()?;
                            let code = if (0xd800..=0xdbff).contains(&first) {
                                if !self.take(b'\\') || !self.take(b'u') {
                                    return Err(self.error("missing low surrogate"));
                                }
                                let second = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&second) {
                                    return Err(self.error("invalid low surrogate"));
                                }
                                0x10000 + ((first - 0xd800) << 10) + second - 0xdc00
                            } else if (0xdc00..=0xdfff).contains(&first) {
                                return Err(self.error("unpaired low surrogate"));
                            } else {
                                first
                            };
                            result.push(
                                char::from_u32(code)
                                    .ok_or_else(|| self.error("invalid Unicode scalar"))?,
                            );
                        }
                        _ => return Err(self.error("invalid escape")),
                    }
                }
                Some(0..=31) => return Err(self.error("unescaped control character")),
                Some(_) => {
                    let ch = self.text[self.pos..]
                        .chars()
                        .next()
                        .ok_or_else(|| self.error("invalid string"))?;
                    result.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }
    fn hex4(&mut self) -> Result<u32, Error> {
        let mut value = 0;
        for _ in 0..4 {
            let digit = self
                .peek()
                .and_then(|b| (b as char).to_digit(16))
                .ok_or_else(|| self.error("invalid Unicode escape"))?;
            self.pos += 1;
            value = value * 16 + digit;
        }
        Ok(value)
    }
    fn number(&mut self) -> Result<Value, Error> {
        let start = self.pos;
        self.take(b'-');
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
            }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                self.digits();
            }
            _ => return Err(self.error("invalid number")),
        }
        if self.take(b'.') && !self.digits() {
            return Err(self.error("missing fraction digits"));
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !self.digits() {
                return Err(self.error("missing exponent digits"));
            }
        }
        Ok(Value::Number(self.text[start..self.pos].into()))
    }
    fn digits(&mut self) -> bool {
        let start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        self.pos != start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_and_unicode() {
        assert_eq!(
            parse(br#""\n\t\"\\\/\b\f\u0041\uD83D\uDE00""#).unwrap(),
            Value::String("\n\t\"\\/\u{8}\u{c}A😀".into())
        );
        for input in [r#""\uD800""#, r#""\uDC00""#, r#""\uD800\u0041""#] {
            assert!(parse(input.as_bytes()).is_err());
        }
    }
    #[test]
    fn bounds_types_and_duplicates() {
        assert!(parse(&[0xff]).is_err());
        assert!(parse(b"").is_err());
        assert!(parse(b"true false").is_err());
        assert!(parse(br#"{"x":1,"x":2}"#).is_err());
        assert!(parse(format!("{}0{}", "[".repeat(64), "]".repeat(64)).as_bytes()).is_err());
        assert!(parse(&vec![b' '; MAX_BYTES + 1]).is_err());
        assert!(parse(format!("\"{}\"", "a".repeat(MAX_BYTES - 2)).as_bytes()).is_ok());
        let v = parse(br#"{"int":1,"float":1.0}"#).unwrap();
        assert_eq!(v.integer("int").unwrap(), 1);
        assert!(v
            .integer("float")
            .unwrap_err()
            .0
            .contains("expected integer"));
        assert!(v.integer("absent").unwrap_err().0.contains("missing key"));
    }
    #[test]
    fn truncated_inputs_never_panic() {
        for text in [
            r#"{"x":["\uD83D\uDE00",true,-12.3e+4]}"#,
            "[null,false,123]",
        ] {
            for i in 0..=text.len() {
                let _ = parse(&text.as_bytes()[..i]);
            }
        }
    }
}
