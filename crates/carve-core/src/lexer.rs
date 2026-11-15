//! Tokenizer for a single script line (after the block structure is known).

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// A bare word, original case kept. `arg-1` style words are one token.
    Word(String),
    Num(f64),
    /// Text between double quotes, with `""` already turned into `"`.
    Str(String),
    /// Raw text between `{` and `}`.
    Var(String),
    Sym(&'static str),
}

impl Tok {
    /// Does this token equal a pattern literal (lowercase word or symbol)?
    pub fn is_lit(&self, lit: &str) -> bool {
        match self {
            Tok::Word(w) => w.eq_ignore_ascii_case(lit),
            Tok::Sym(s) => *s == lit,
            _ => false,
        }
    }
}

const SYMS2: [&str; 3] = [">=", "<=", "!="];
const SYMS1: [&str; 15] = [
    ",", "[", "]", "(", ")", "+", "-", "*", "/", "=", "<", ">", ":", "?", "%",
];

pub fn lex(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let n = c.len();
    let mut i = 0;
    let mut out = Vec::new();
    while i < n {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            while i < n && c[i].is_ascii_digit() {
                i += 1;
            }
            if i + 1 < n && c[i] == '.' && c[i + 1].is_ascii_digit() {
                i += 1;
                while i < n && c[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let text: String = c[start..i].iter().collect();
            let v = text.parse::<f64>().map_err(|_| format!("bad number '{text}'"))?;
            out.push(Tok::Num(v));
            continue;
        }
        if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < n && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            // Contractions like `doesn't`, `isn't`.
            if i + 1 < n && c[i] == '\'' && c[i + 1] == 't' && (i + 2 >= n || !c[i + 2].is_alphanumeric()) {
                i += 2;
            }
            let mut word: String = c[start..i].iter().collect();
            // `bottle o' enchanting`: the apostrophe belongs to the word.
            if word == "o"
                && i < n
                && c[i] == '\''
                && matches!(out.last(), Some(Tok::Word(w)) if w.eq_ignore_ascii_case("bottle"))
            {
                i += 1;
                word = "o'".to_string();
            }
            // `arg-1`, `arg-2`: one word.
            if word.eq_ignore_ascii_case("arg") && i + 1 < n && c[i] == '-' && c[i + 1].is_ascii_digit() {
                i += 1;
                let ds = i;
                while i < n && c[i].is_ascii_digit() {
                    i += 1;
                }
                let digits: String = c[ds..i].iter().collect();
                word = format!("arg-{digits}");
            }
            // `event-block`, `event-max-players`, `event-line-1`: one word.
            if word.eq_ignore_ascii_case("event") && i + 1 < n && c[i] == '-' && c[i + 1].is_alphabetic() {
                let mut w = String::from("event");
                while i + 1 < n && c[i] == '-' && c[i + 1].is_alphanumeric() {
                    i += 1;
                    w.push('-');
                    while i < n && c[i].is_alphanumeric() {
                        w.push(c[i].to_ascii_lowercase());
                        i += 1;
                    }
                }
                word = w;
            }
            // `loop-value`, `loop-player`, `loop-value-2`: one word.
            if word.eq_ignore_ascii_case("loop") && i + 1 < n && c[i] == '-' && c[i + 1].is_alphabetic() {
                i += 1;
                let ws = i;
                while i < n && c[i].is_alphabetic() {
                    i += 1;
                }
                let part: String = c[ws..i].iter().collect();
                word = format!("loop-{}", part.to_ascii_lowercase());
                // `loop-offline player`: two words, one value.
                if word == "loop-offline"
                    && c.get(i) == Some(&' ')
                    && c.get(i + 1..i + 7)
                        .is_some_and(|w| w.iter().collect::<String>().eq_ignore_ascii_case("player"))
                    && !c.get(i + 7).is_some_and(|ch| ch.is_alphanumeric())
                {
                    i += 7;
                    word = "loop-offlineplayer".to_string();
                }
                if i + 1 < n && c[i] == '-' && c[i + 1].is_ascii_digit() {
                    i += 1;
                    let ds = i;
                    while i < n && c[i].is_ascii_digit() {
                        i += 1;
                    }
                    let digits: String = c[ds..i].iter().collect();
                    word = format!("{word}-{digits}");
                }
            }
            out.push(Tok::Word(word));
            continue;
        }
        // Possessive `player's`.
        if ch == '\''
            && i + 1 < n
            && (c[i + 1] == 's' || c[i + 1] == 'S')
            && (i + 2 >= n || !c[i + 2].is_alphanumeric())
        {
            out.push(Tok::Sym("'s"));
            i += 2;
            continue;
        }
        if ch == '"' {
            i += 1;
            let mut text = String::new();
            // Inside `%...%` quotes start a nested text, like `"%f("x")%"`.
            let mut in_expr = false;
            loop {
                if i >= n {
                    return Err("unclosed text: missing closing \"".into());
                }
                if c[i] == '%' {
                    if !in_expr && i + 1 < n && c[i + 1] == '%' {
                        text.push_str("%%");
                        i += 2;
                        continue;
                    }
                    in_expr = !in_expr;
                } else if c[i] == '"' && in_expr && c.get(i + 1) == Some(&'"') {
                    // `""` stays the escaped quote, also inside `%...%`.
                    text.push('"');
                    i += 2;
                    continue;
                } else if c[i] == '"' && in_expr && c.get(i + 1..).is_some_and(|r| r.contains(&'%')) {
                    let close = c.get(i + 1..).and_then(|r| r.iter().position(|ch| *ch == '"'));
                    if let Some(len) = close {
                        text.extend(c.get(i..i + len + 2).unwrap_or_default());
                        i += len + 2;
                        continue;
                    }
                }
                if c[i] == '"' {
                    if i + 1 < n && c[i + 1] == '"' {
                        text.push('"');
                        i += 2;
                        continue;
                    }
                    i += 1;
                    break;
                }
                text.push(c[i]);
                i += 1;
            }
            out.push(Tok::Str(text));
            continue;
        }
        if ch == '{' {
            i += 1;
            let mut depth = 1;
            let mut text = String::new();
            loop {
                if i >= n {
                    return Err("unclosed variable: missing closing }".into());
                }
                match c[i] {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                text.push(c[i]);
                i += 1;
            }
            if text.trim().is_empty() {
                return Err("empty variable name {}".into());
            }
            out.push(Tok::Var(text));
            continue;
        }
        if let Some(&next) = c.get(i + 1)
            && let Some(s) = SYMS2.iter().find(|s| s.chars().eq([ch, next]))
        {
            out.push(Tok::Sym(s));
            i += 2;
            continue;
        }
        if let Some(s) = SYMS1.iter().find(|s| s.chars().eq([ch])) {
            out.push(Tok::Sym(s));
            i += 1;
            continue;
        }
        return Err(format!("unexpected character '{ch}'"));
    }
    Ok(out)
}

/// Remove a `#` comment, ignoring `#` inside quotes or braces. Quotes nest
/// inside `%...%` in a text (`"%f("#,##0")%"`); `""` and `%%` are escapes.
pub fn strip_comment(line: &str) -> &str {
    #[derive(Clone, Copy)]
    enum Ctx {
        Str,
        Interp,
        Brace,
    }
    let mut stack: Vec<Ctx> = Vec::new();
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        let next = chars.peek().map(|&(_, c)| c);
        match (stack.last().copied(), ch) {
            (Some(Ctx::Str), '"' | '%') if next == Some(ch) => {
                chars.next();
            }
            (Some(Ctx::Str), '"') | (Some(Ctx::Interp), '%') | (Some(Ctx::Brace), '}') => {
                stack.pop();
            }
            (Some(Ctx::Str | Ctx::Brace), '%') => stack.push(Ctx::Interp),
            (Some(Ctx::Str), _) => {}
            (_, '"') => stack.push(Ctx::Str),
            (_, '{') => stack.push(Ctx::Brace),
            (None, '#') => return &line[..idx],
            _ => {}
        }
    }
    line
}
