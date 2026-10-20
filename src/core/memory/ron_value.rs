#[derive(Debug)]
pub(super) enum RonValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Unit(String),
    Option(Option<Box<RonValue>>),
    Tuple(Vec<RonValue>),
    List(Vec<RonValue>),
    Map(Vec<(String, RonValue)>),
    TupleVariant(String, Vec<RonValue>),
    StructVariant(String, Vec<(String, RonValue)>),
}

#[derive(Clone, Copy)]
struct RonCursor<'a> {
    source: &'a str,
    byte: usize,
}

impl<'a> RonCursor<'a> {
    fn rest(self) -> &'a str {
        &self.source[self.byte..]
    }

    fn peek(self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.byte += ch.len_utf8();
        Some(ch)
    }

    fn consume_str(&mut self, token: &str) -> bool {
        if self.rest().starts_with(token) {
            self.byte += token.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: &str) -> Result<(), String> {
        if self.consume_str(token) {
            Ok(())
        } else {
            Err(self.fail(&format!("expected {token}")))
        }
    }

    fn fail(self, message: &str) -> String {
        format!("{message} at byte {}", self.byte)
    }

    fn skip_ws(&mut self) -> Result<(), String> {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.bump();
            }
            if self.consume_str("//") {
                while self.peek().is_some_and(|ch| ch != '\n') {
                    self.bump();
                }
                continue;
            }
            if self.consume_str("/*") {
                self.skip_block_comment()?;
                continue;
            }
            return Ok(());
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        loop {
            if self.consume_str("*/") {
                return Ok(());
            }
            if self.bump().is_none() {
                return Err(self.fail("unterminated block comment"));
            }
        }
    }

    fn consume_ident(&mut self, token: &str) -> bool {
        if !self.rest().starts_with(token) {
            return false;
        }
        let after = self.byte + token.len();
        let continues = self.source[after..]
            .chars()
            .next()
            .is_some_and(is_ident_continue);
        if continues {
            return false;
        }
        self.byte = after;
        true
    }

    fn consume_any_ident(&mut self) -> Option<String> {
        let start = self.byte;
        if !self.peek().is_some_and(is_ident_start) {
            return None;
        }
        self.bump();
        while self.peek().is_some_and(is_ident_continue) {
            self.bump();
        }
        Some(self.source[start..self.byte].to_string())
    }

    fn looks_like_field(self) -> bool {
        let mut probe = self;
        if probe.consume_any_ident().is_none() {
            return false;
        }
        let _ = probe.skip_ws();
        probe.rest().starts_with(':')
    }

    fn parse_value(&mut self) -> Result<RonValue, String> {
        self.skip_ws()?;
        if self.consume_ident("true") {
            return Ok(RonValue::Bool(true));
        }
        if self.consume_ident("false") {
            return Ok(RonValue::Bool(false));
        }
        if self.consume_ident("None") {
            return Ok(RonValue::Option(None));
        }
        if self.consume_ident("Some") {
            return self.parse_some();
        }
        if self.peek() == Some('"') {
            return self.parse_string().map(RonValue::String);
        }
        if self.peek().is_some_and(starts_number) {
            return self.parse_number();
        }
        if self.rest().starts_with('(') {
            return self.parse_paren();
        }
        if self.rest().starts_with('[') {
            return self.parse_list();
        }
        if let Some(name) = self.consume_any_ident() {
            return self.parse_after_ident(name);
        }
        Err(self.fail("expected a RON value"))
    }

    fn parse_some(&mut self) -> Result<RonValue, String> {
        self.skip_ws()?;
        self.expect("(")?;
        let inner = self.parse_value()?;
        self.skip_ws()?;
        self.expect(")")?;
        Ok(RonValue::Option(Some(Box::new(inner))))
    }

    fn parse_after_ident(&mut self, name: String) -> Result<RonValue, String> {
        self.skip_ws()?;
        if !self.consume_str("(") {
            return Ok(RonValue::Unit(name));
        }
        self.skip_ws()?;
        if self.consume_str(")") {
            return Ok(RonValue::TupleVariant(name, Vec::new()));
        }
        if self.looks_like_field() {
            let fields = self.parse_fields_until(')')?;
            return Ok(RonValue::StructVariant(name, fields));
        }
        let items = self.parse_values_until(')')?;
        Ok(RonValue::TupleVariant(name, items))
    }

    fn parse_paren(&mut self) -> Result<RonValue, String> {
        self.expect("(")?;
        self.skip_ws()?;
        if self.consume_str(")") {
            return Ok(RonValue::Tuple(Vec::new()));
        }
        if self.looks_like_field() {
            return Ok(RonValue::Map(self.parse_fields_until(')')?));
        }
        Ok(RonValue::Tuple(self.parse_values_until(')')?))
    }

    fn parse_list(&mut self) -> Result<RonValue, String> {
        self.expect("[")?;
        Ok(RonValue::List(self.parse_values_until(']')?))
    }

    fn parse_values_until(&mut self, closer: char) -> Result<Vec<RonValue>, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws()?;
            if self.consume_closer(closer) {
                return Ok(items);
            }
            items.push(self.parse_value()?);
            self.skip_ws()?;
            if self.consume_str(",") {
                continue;
            }
            if self.consume_closer(closer) {
                return Ok(items);
            }
            return Err(self.fail("expected a comma or the end of a sequence"));
        }
    }

    fn parse_fields_until(&mut self, closer: char) -> Result<Vec<(String, RonValue)>, String> {
        let mut fields = Vec::new();
        loop {
            self.skip_ws()?;
            if self.consume_closer(closer) {
                return Ok(fields);
            }
            let Some(key) = self.consume_any_ident() else {
                return Err(self.fail("expected a field name"));
            };
            self.skip_ws()?;
            self.expect(":")?;
            let value = self.parse_value()?;
            fields.push((key, value));
            self.skip_ws()?;
            if self.consume_str(",") {
                continue;
            }
            if self.consume_closer(closer) {
                return Ok(fields);
            }
            return Err(self.fail("expected a comma or the end of a struct"));
        }
    }

    fn consume_closer(&mut self, closer: char) -> bool {
        if self.peek() == Some(closer) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn parse_number(&mut self) -> Result<RonValue, String> {
        let start = self.byte;
        if self.peek() == Some('+') || self.peek() == Some('-') {
            self.bump();
        }
        let mut saw_digit = self.consume_digits();
        let mut is_float = false;
        if self.peek() == Some('.') {
            is_float = true;
            self.bump();
            saw_digit = self.consume_digits() || saw_digit;
        }
        if self.peek() == Some('e') || self.peek() == Some('E') {
            is_float = true;
            self.bump();
            if self.peek() == Some('+') || self.peek() == Some('-') {
                self.bump();
            }
            if !self.consume_digits() {
                return Err(self.fail("expected an exponent"));
            }
        }
        if !saw_digit {
            return Err(self.fail("expected a number"));
        }
        let lexeme = &self.source[start..self.byte];
        if is_float {
            let value = lexeme
                .parse::<f64>()
                .map_err(|_| self.fail("invalid float"))?;
            return Ok(RonValue::Float(value));
        }
        let value = lexeme
            .parse::<i64>()
            .map_err(|_| self.fail("invalid integer"))?;
        Ok(RonValue::Integer(value))
    }

    fn consume_digits(&mut self) -> bool {
        let mut saw_digit = false;
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            saw_digit = true;
            self.bump();
        }
        saw_digit
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect("\"")?;
        let mut text = String::new();
        loop {
            let Some(ch) = self.bump() else {
                return Err(self.fail("unterminated string"));
            };
            if ch == '"' {
                return Ok(text);
            }
            if ch != '\\' {
                text.push(ch);
                continue;
            }
            text.push(self.parse_escape()?);
        }
    }

    fn parse_escape(&mut self) -> Result<char, String> {
        let Some(escaped) = self.bump() else {
            return Err(self.fail("unterminated string escape"));
        };
        match escaped {
            '"' | '\\' | '\'' => Ok(escaped),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '0' => Ok('\0'),
            'u' => self.parse_unicode_escape(),
            _ => Err(self.fail("unknown string escape")),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        self.expect("{")?;
        let start = self.byte;
        while self.peek().is_some_and(|ch| ch.is_ascii_hexdigit()) {
            self.bump();
        }
        let hex = &self.source[start..self.byte];
        if hex.is_empty() {
            return Err(self.fail("empty unicode escape"));
        }
        self.expect("}")?;
        let code = u32::from_str_radix(hex, 16).map_err(|_| self.fail("invalid unicode escape"))?;
        char::from_u32(code).ok_or_else(|| self.fail("invalid unicode scalar"))
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn starts_number(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '+' || ch == '-'
}

pub(super) fn parse_ron_value(source: &str) -> Result<RonValue, String> {
    let mut cursor = RonCursor { source, byte: 0 };
    let value = cursor.parse_value()?;
    cursor.skip_ws()?;
    if cursor.byte != cursor.source.len() {
        return Err(cursor.fail("unexpected trailing input"));
    }
    Ok(value)
}

pub(super) fn print_ron_value(value: &RonValue) -> String {
    let mut out = String::new();
    write_value(&mut out, value);
    out
}

fn write_value(out: &mut String, value: &RonValue) {
    match value {
        RonValue::Bool(true) => out.push_str("true"),
        RonValue::Bool(false) => out.push_str("false"),
        RonValue::Integer(number) => out.push_str(&number.to_string()),
        RonValue::Float(number) => write_float(out, *number),
        RonValue::String(text) => write_string(out, text),
        RonValue::Unit(name) => out.push_str(name),
        RonValue::Option(None) => out.push_str("None"),
        RonValue::Option(Some(inner)) => {
            out.push_str("Some(");
            write_value(out, inner);
            out.push(')');
        }
        RonValue::Tuple(items) => write_seq(out, '(', ')', items),
        RonValue::List(items) => write_seq(out, '[', ']', items),
        RonValue::Map(fields) => write_fields(out, None, fields),
        RonValue::TupleVariant(name, items) => {
            out.push_str(name);
            write_seq(out, '(', ')', items);
        }
        RonValue::StructVariant(name, fields) => write_fields(out, Some(name), fields),
    }
}

fn write_seq(out: &mut String, open: char, close: char, items: &[RonValue]) {
    out.push(open);
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write_value(out, item);
    }
    out.push(close);
}

fn write_fields(out: &mut String, name: Option<&str>, fields: &[(String, RonValue)]) {
    if let Some(name) = name {
        out.push_str(name);
    }
    out.push('(');
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(key);
        out.push(':');
        write_value(out, value);
    }
    out.push(')');
}

fn write_float(out: &mut String, value: f64) {
    let text = value.to_string();
    out.push_str(&text);
    if !text.contains('.') && !text.contains('e') && !text.contains('E') {
        out.push_str(".0");
    }
}

fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        for escaped in ch.escape_debug() {
            out.push(escaped);
        }
    }
    out.push('"');
}
