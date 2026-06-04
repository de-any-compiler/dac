//! Constrained TOML reader for hint files.
//!
//! Supports the narrow subset the [`crate::Hints`] schema needs:
//! `[[name]]` headers, scalar / array / inline-table values, hash
//! comments, and string escapes `\"`, `\\`, `\n`, `\t`. The
//! resulting [`Document`] is then folded into [`crate::Hints`] by
//! [`parse_toml`]; that step also runs the semantic checks
//! (matcher present, type parses, …).

use super::{FunctionHint, HintError, HintMatcher, HintType, Hints, StructFieldHint, StructHint};

/// Top-level entry point: parse a hint file's contents into a
/// [`Hints`] catalogue.
pub fn parse_toml(input: &str) -> Result<Hints, HintError> {
    let doc = Parser::new(input).parse_document()?;
    fold(doc)
}

/// Standalone type-string parser used by [`crate::HintType::parse`].
pub(crate) fn parse_type(input: &str) -> Result<HintType, HintError> {
    let trimmed = input.trim();
    let mut cursor = TypeCursor::new(trimmed);
    let ty = cursor.parse()?;
    cursor.skip_whitespace();
    if !cursor.is_eof() {
        return Err(HintError::Semantic {
            line: 0,
            message: format!("unexpected trailing input in type `{input}`"),
        });
    }
    Ok(ty)
}

// ---------- TOML AST -------------------------------------------------

#[derive(Debug, Default)]
struct Document {
    tables: Vec<TableInstance>,
}

#[derive(Debug)]
struct TableInstance {
    name: String,
    line: u32,
    entries: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Integer(i128),
    Array(Vec<Value>),
    InlineTable(Vec<(String, Value)>),
}

// ---------- folding TOML AST → Hints --------------------------------

fn fold(doc: Document) -> Result<Hints, HintError> {
    let mut hints = Hints::default();
    let mut next_id: u64 = 1;
    for table in doc.tables {
        match table.name.as_str() {
            "function" => {
                let hint = fold_function(next_id, table)?;
                next_id += 1;
                hints.functions.push(hint);
            }
            "struct" => {
                let hint = fold_struct(next_id, table)?;
                next_id += 1;
                hints.structs.push(hint);
            }
            other => {
                return Err(HintError::Semantic {
                    line: table.line,
                    message: format!("unknown hint table `[[{other}]]`"),
                });
            }
        }
    }
    Ok(hints)
}

fn fold_function(id: u64, table: TableInstance) -> Result<FunctionHint, HintError> {
    let line = table.line;
    let mut address: Option<u64> = None;
    let mut name: Option<String> = None;
    let mut rename: Option<String> = None;
    let mut return_ty: Option<HintType> = None;
    let mut args: Option<Vec<HintType>> = None;

    for (key, value) in table.entries {
        match key.as_str() {
            "address" => {
                let s = expect_string(&key, &value, line)?;
                address = Some(parse_address(&s, line)?);
            }
            "name" => name = Some(expect_string(&key, &value, line)?),
            "rename" => rename = Some(expect_string(&key, &value, line)?),
            "return" => {
                let s = expect_string(&key, &value, line)?;
                return_ty = Some(parse_type_at(&s, line)?);
            }
            "args" => {
                let arr = expect_array(&key, &value, line)?;
                let mut list = Vec::with_capacity(arr.len());
                for item in arr {
                    let s = expect_string("args[]", item, line)?;
                    let ty = parse_type_at(&s, line)?;
                    if ty.is_void() {
                        return Err(HintError::Semantic {
                            line,
                            message: "`void` is not a valid argument type".into(),
                        });
                    }
                    list.push(ty);
                }
                args = Some(list);
            }
            other => {
                return Err(HintError::Semantic {
                    line,
                    message: format!("unknown key `{other}` in [[function]]"),
                });
            }
        }
    }

    let matcher = match (address, name) {
        (Some(a), Some(n)) => HintMatcher::Both {
            address: a,
            name: n,
        },
        (Some(a), None) => HintMatcher::Address(a),
        (None, Some(n)) => HintMatcher::Name(n),
        (None, None) => {
            return Err(HintError::Semantic {
                line,
                message: "[[function]] requires `address` or `name` (or both)".into(),
            });
        }
    };
    if rename.is_none() && return_ty.is_none() && args.is_none() {
        return Err(HintError::Semantic {
            line,
            message: "[[function]] has no effect (no rename / return / args)".into(),
        });
    }
    Ok(FunctionHint {
        id,
        line,
        matcher,
        rename,
        return_ty,
        args,
        evidence: None,
    })
}

fn fold_struct(id: u64, table: TableInstance) -> Result<StructHint, HintError> {
    let line = table.line;
    let mut name: Option<String> = None;
    let mut fields: Vec<StructFieldHint> = Vec::new();

    for (key, value) in table.entries {
        match key.as_str() {
            "name" => name = Some(expect_string(&key, &value, line)?),
            "fields" => {
                let arr = expect_array(&key, &value, line)?;
                for item in arr {
                    let entries = expect_inline_table("fields[]", item, line)?;
                    fields.push(fold_struct_field(entries, line)?);
                }
            }
            other => {
                return Err(HintError::Semantic {
                    line,
                    message: format!("unknown key `{other}` in [[struct]]"),
                });
            }
        }
    }
    let name = name.ok_or(HintError::Semantic {
        line,
        message: "[[struct]] requires `name`".into(),
    })?;
    if fields.is_empty() {
        return Err(HintError::Semantic {
            line,
            message: "[[struct]] requires at least one field".into(),
        });
    }
    Ok(StructHint {
        id,
        line,
        name,
        fields,
        evidence: None,
    })
}

fn fold_struct_field(
    entries: Vec<(String, Value)>,
    line: u32,
) -> Result<StructFieldHint, HintError> {
    let mut name: Option<String> = None;
    let mut offset: Option<u64> = None;
    let mut ty: Option<HintType> = None;
    for (k, v) in entries {
        match k.as_str() {
            "name" => name = Some(expect_string(&k, &v, line)?),
            "offset" => {
                offset = Some(match v {
                    Value::Integer(n) if n >= 0 => n as u64,
                    Value::String(ref s) => parse_address(s, line)?,
                    other => {
                        return Err(HintError::Semantic {
                            line,
                            message: format!(
                                "field `offset` must be an integer or hex string, got {other:?}"
                            ),
                        });
                    }
                });
            }
            "ty" => {
                let s = expect_string(&k, &v, line)?;
                ty = Some(parse_type_at(&s, line)?);
            }
            other => {
                return Err(HintError::Semantic {
                    line,
                    message: format!("unknown key `{other}` in struct field"),
                });
            }
        }
    }
    let name = name.ok_or(HintError::Semantic {
        line,
        message: "struct field requires `name`".into(),
    })?;
    let offset = offset.ok_or(HintError::Semantic {
        line,
        message: "struct field requires `offset`".into(),
    })?;
    let ty = ty.ok_or(HintError::Semantic {
        line,
        message: "struct field requires `ty`".into(),
    })?;
    if ty.is_void() {
        return Err(HintError::Semantic {
            line,
            message: "`void` is not a valid field type".into(),
        });
    }
    Ok(StructFieldHint { name, offset, ty })
}

fn expect_string(key: &str, value: &Value, line: u32) -> Result<String, HintError> {
    match value {
        Value::String(s) => Ok(s.clone()),
        other => Err(HintError::Semantic {
            line,
            message: format!("expected string for `{key}`, got {other:?}"),
        }),
    }
}

fn expect_array<'a>(key: &str, value: &'a Value, line: u32) -> Result<&'a [Value], HintError> {
    match value {
        Value::Array(items) => Ok(items.as_slice()),
        other => Err(HintError::Semantic {
            line,
            message: format!("expected array for `{key}`, got {other:?}"),
        }),
    }
}

fn expect_inline_table(
    key: &str,
    value: &Value,
    line: u32,
) -> Result<Vec<(String, Value)>, HintError> {
    match value {
        Value::InlineTable(entries) => Ok(entries.clone()),
        other => Err(HintError::Semantic {
            line,
            message: format!("expected inline table for `{key}`, got {other:?}"),
        }),
    }
}

fn parse_address(s: &str, line: u32) -> Result<u64, HintError> {
    let stripped = s.trim();
    let (radix, body) = if let Some(rest) = stripped
        .strip_prefix("0x")
        .or_else(|| stripped.strip_prefix("0X"))
    {
        (16, rest)
    } else {
        (10, stripped)
    };
    u64::from_str_radix(body, radix).map_err(|_| HintError::Semantic {
        line,
        message: format!("invalid address `{s}`"),
    })
}

fn parse_type_at(s: &str, line: u32) -> Result<HintType, HintError> {
    HintType::parse(s).map_err(|e| match e {
        HintError::Semantic { message, .. } => HintError::Semantic { line, message },
        other => other,
    })
}

// ---------- type-string parser --------------------------------------

struct TypeCursor<'a> {
    src: &'a str,
    idx: usize,
}

impl<'a> TypeCursor<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, idx: 0 }
    }

    fn parse(&mut self) -> Result<HintType, HintError> {
        self.skip_whitespace();
        let atom = self.parse_atom()?;
        let mut ty = atom;
        loop {
            self.skip_whitespace();
            if self.peek() == Some('*') {
                self.idx += 1;
                ty = HintType::Ptr(Box::new(ty));
            } else {
                break;
            }
        }
        Ok(ty)
    }

    fn parse_atom(&mut self) -> Result<HintType, HintError> {
        let start = self.idx;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.idx += c.len_utf8();
            } else {
                break;
            }
        }
        if self.idx == start {
            return Err(HintError::Semantic {
                line: 0,
                message: format!("expected type atom near `{}`", &self.src[start..]),
            });
        }
        let atom = &self.src[start..self.idx];
        let ty = match atom {
            "void" => HintType::Void,
            "char" => HintType::Int {
                width_bits: 8,
                signed: Some(true),
            },
            "uchar" => HintType::Int {
                width_bits: 8,
                signed: Some(false),
            },
            "short" => HintType::Int {
                width_bits: 16,
                signed: Some(true),
            },
            "ushort" => HintType::Int {
                width_bits: 16,
                signed: Some(false),
            },
            "int" => HintType::Int {
                width_bits: 32,
                signed: Some(true),
            },
            "uint" => HintType::Int {
                width_bits: 32,
                signed: Some(false),
            },
            "long" => HintType::Int {
                width_bits: 64,
                signed: Some(true),
            },
            "ulong" => HintType::Int {
                width_bits: 64,
                signed: Some(false),
            },
            "int8" | "i8" => HintType::Int {
                width_bits: 8,
                signed: Some(true),
            },
            "int16" | "i16" => HintType::Int {
                width_bits: 16,
                signed: Some(true),
            },
            "int32" | "i32" => HintType::Int {
                width_bits: 32,
                signed: Some(true),
            },
            "int64" | "i64" => HintType::Int {
                width_bits: 64,
                signed: Some(true),
            },
            "uint8" | "u8" => HintType::Int {
                width_bits: 8,
                signed: Some(false),
            },
            "uint16" | "u16" => HintType::Int {
                width_bits: 16,
                signed: Some(false),
            },
            "uint32" | "u32" => HintType::Int {
                width_bits: 32,
                signed: Some(false),
            },
            "uint64" | "u64" => HintType::Int {
                width_bits: 64,
                signed: Some(false),
            },
            other => {
                return Err(HintError::Semantic {
                    line: 0,
                    message: format!("unknown type atom `{other}`"),
                });
            }
        };
        Ok(ty)
    }

    fn peek(&self) -> Option<char> {
        self.src[self.idx..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.idx += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.src.len()
    }
}

// ---------- strict-TOML parser --------------------------------------

struct Parser<'a> {
    src: &'a [u8],
    idx: usize,
    line: u32,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            idx: 0,
            line: 1,
        }
    }

    fn parse_document(mut self) -> Result<Document, HintError> {
        let mut doc = Document::default();
        let mut current: Option<TableInstance> = None;
        loop {
            self.skip_whitespace_and_comments();
            if self.is_eof() {
                break;
            }
            if self.peek() == Some(b'[') && self.peek_at(1) == Some(b'[') {
                if let Some(t) = current.take() {
                    doc.tables.push(t);
                }
                let (name, line) = self.parse_array_header()?;
                current = Some(TableInstance {
                    name,
                    line,
                    entries: Vec::new(),
                });
            } else if self.peek() == Some(b'[') {
                return Err(self.error_here(
                    "single-table headers `[name]` are not supported; use `[[name]]`",
                ));
            } else {
                let Some(ref mut tbl) = current else {
                    return Err(self
                        .error_here("expected `[[function]]` or `[[struct]]` header before keys"));
                };
                let entry = self.parse_key_value()?;
                tbl.entries.push(entry);
            }
        }
        if let Some(t) = current {
            doc.tables.push(t);
        }
        Ok(doc)
    }

    fn parse_array_header(&mut self) -> Result<(String, u32), HintError> {
        let line = self.line;
        // Consume `[[`.
        self.idx += 2;
        self.skip_inline_whitespace();
        let name = self.parse_bare_key()?;
        self.skip_inline_whitespace();
        if !self.consume_byte(b']') || !self.consume_byte(b']') {
            return Err(self.error_here("expected `]]` to close array-of-tables header"));
        }
        self.expect_line_end()?;
        Ok((name, line))
    }

    fn parse_key_value(&mut self) -> Result<(String, Value), HintError> {
        let key = self.parse_bare_key()?;
        self.skip_inline_whitespace();
        if !self.consume_byte(b'=') {
            return Err(self.error_here(&format!("expected `=` after key `{key}`")));
        }
        self.skip_inline_whitespace();
        let value = self.parse_value()?;
        self.expect_line_end()?;
        Ok((key, value))
    }

    fn parse_value(&mut self) -> Result<Value, HintError> {
        self.skip_inline_whitespace();
        match self.peek() {
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b'[') => self.parse_array().map(Value::Array),
            Some(b'{') => self.parse_inline_table().map(Value::InlineTable),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_integer().map(Value::Integer),
            _ => Err(self.error_here("expected value (string, integer, array, or inline table)")),
        }
    }

    fn parse_string(&mut self) -> Result<String, HintError> {
        // Consume opening quote.
        self.idx += 1;
        let mut out = String::new();
        loop {
            match self.next_byte() {
                None => return Err(self.error_here("unterminated string literal")),
                Some(b'"') => return Ok(out),
                Some(b'\\') => {
                    let esc = self
                        .next_byte()
                        .ok_or_else(|| self.error_here("dangling backslash at end of input"))?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        other => {
                            return Err(self.error_here(&format!(
                                "unsupported string escape `\\{}`",
                                other as char
                            )));
                        }
                    }
                }
                Some(b'\n') => {
                    return Err(self.error_here("unterminated string literal (saw newline)"))
                }
                Some(b) => out.push(b as char),
            }
        }
    }

    fn parse_integer(&mut self) -> Result<i128, HintError> {
        let start = self.idx;
        if self.peek() == Some(b'-') {
            self.idx += 1;
        }
        while let Some(b) = self.peek() {
            if b.is_ascii_digit() {
                self.idx += 1;
            } else {
                break;
            }
        }
        let raw = std::str::from_utf8(&self.src[start..self.idx]).unwrap_or("");
        raw.parse::<i128>()
            .map_err(|_| self.error_here(&format!("invalid integer literal `{raw}`")))
    }

    fn parse_array(&mut self) -> Result<Vec<Value>, HintError> {
        // Consume `[`.
        self.idx += 1;
        let mut items = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(b']') {
                self.idx += 1;
                return Ok(items);
            }
            let value = self.parse_value()?;
            items.push(value);
            self.skip_whitespace_and_comments();
            if self.peek() == Some(b',') {
                self.idx += 1;
                continue;
            }
            self.skip_whitespace_and_comments();
            if self.peek() == Some(b']') {
                self.idx += 1;
                return Ok(items);
            }
            return Err(self.error_here("expected `,` or `]` in array"));
        }
    }

    fn parse_inline_table(&mut self) -> Result<Vec<(String, Value)>, HintError> {
        // Consume `{`.
        self.idx += 1;
        let mut entries = Vec::new();
        loop {
            self.skip_inline_whitespace_and_newlines();
            if self.peek() == Some(b'}') {
                self.idx += 1;
                return Ok(entries);
            }
            let key = self.parse_bare_key()?;
            self.skip_inline_whitespace();
            if !self.consume_byte(b'=') {
                return Err(self.error_here(&format!("expected `=` after `{key}` in inline table")));
            }
            self.skip_inline_whitespace();
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_inline_whitespace_and_newlines();
            if self.peek() == Some(b',') {
                self.idx += 1;
                continue;
            }
            self.skip_inline_whitespace_and_newlines();
            if self.peek() == Some(b'}') {
                self.idx += 1;
                return Ok(entries);
            }
            return Err(self.error_here("expected `,` or `}` in inline table"));
        }
    }

    fn parse_bare_key(&mut self) -> Result<String, HintError> {
        let start = self.idx;
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                self.idx += 1;
            } else {
                break;
            }
        }
        if self.idx == start {
            return Err(self.error_here("expected key"));
        }
        Ok(std::str::from_utf8(&self.src[start..self.idx])
            .unwrap()
            .to_string())
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(b' ') | Some(b'\t') | Some(b'\r') => self.idx += 1,
                Some(b'\n') => {
                    self.idx += 1;
                    self.line += 1;
                }
                Some(b'#') => {
                    while let Some(b) = self.peek() {
                        if b == b'\n' {
                            break;
                        }
                        self.idx += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn skip_inline_whitespace(&mut self) {
        while let Some(b' ') | Some(b'\t') | Some(b'\r') = self.peek() {
            self.idx += 1;
        }
    }

    fn skip_inline_whitespace_and_newlines(&mut self) {
        loop {
            match self.peek() {
                Some(b' ') | Some(b'\t') | Some(b'\r') => self.idx += 1,
                Some(b'\n') => {
                    self.idx += 1;
                    self.line += 1;
                }
                Some(b'#') => {
                    while let Some(b) = self.peek() {
                        if b == b'\n' {
                            break;
                        }
                        self.idx += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn expect_line_end(&mut self) -> Result<(), HintError> {
        self.skip_inline_whitespace();
        if self.peek() == Some(b'#') {
            while let Some(b) = self.peek() {
                if b == b'\n' {
                    break;
                }
                self.idx += 1;
            }
        }
        match self.peek() {
            None => Ok(()),
            Some(b'\n') => {
                self.idx += 1;
                self.line += 1;
                Ok(())
            }
            Some(_) => Err(self.error_here("expected end of line after value")),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.idx).copied()
    }

    fn peek_at(&self, k: usize) -> Option<u8> {
        self.src.get(self.idx + k).copied()
    }

    fn next_byte(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.idx += 1;
        if b == b'\n' {
            self.line += 1;
        }
        Some(b)
    }

    fn consume_byte(&mut self, target: u8) -> bool {
        if self.peek() == Some(target) {
            self.idx += 1;
            true
        } else {
            false
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.src.len()
    }

    fn error_here(&self, message: &str) -> HintError {
        HintError::Syntax {
            line: self.line,
            message: message.to_string(),
        }
    }
}
