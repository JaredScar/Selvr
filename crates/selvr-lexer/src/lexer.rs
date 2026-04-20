use smol_str::SmolStr;
use crate::{
    error::LexError,
    span::{Span, Spanned},
    token::Token,
};

/// Hand-written lexer for Selvr.
///
/// Produces a flat `Vec<Spanned<Token>>` including trivia (comments).
/// Errors are collected rather than immediately aborting — the parser
/// receives `Token::Error` for each bad character and can continue.
///
/// Template literals (backtick strings) are broken into segments:
///   `Hello ${name}, you have ${count} messages!`
/// becomes:
///   TemplateLitStart("Hello ")  ·  Ident(name)  ·  TemplateLitMid(", you have ")
///   · Ident(count) · TemplateLitEnd(" messages!")
pub struct Lexer<'src> {
    src: &'src str,
    pos: usize,
    file_id: u32,
    pub errors: Vec<LexError>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str, file_id: u32) -> Self {
        Self { src, pos: 0, file_id, errors: Vec::new() }
    }

    pub fn tokenize(mut self) -> (Vec<Spanned<Token>>, Vec<LexError>) {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.node == Token::Eof;
            tokens.push(tok);
            if is_eof { break; }
        }
        let errors = self.errors;
        (tokens, errors)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn span(&self, start: usize) -> Span {
        Span::new(start, self.pos, self.file_id)
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek2(&self) -> Option<char> {
        let mut chars = self.src[self.pos..].chars();
        chars.next();
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn eat_while(&mut self, mut pred: impl FnMut(char) -> bool) {
        while self.peek().map_or(false, |c| pred(c)) {
            self.advance();
        }
    }

    fn current_slice(&self, start: usize) -> &str {
        &self.src[start..self.pos]
    }

    // ── Token production ──────────────────────────────────────────────────────

    fn next_token(&mut self) -> Spanned<Token> {
        self.skip_whitespace();
        let start = self.pos;

        let ch = match self.advance() {
            None => return Spanned::new(Token::Eof, self.span(start)),
            Some(c) => c,
        };

        let tok = match ch {
            // ── Single-char punctuation ──────────────────────────────────────
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ';' => Token::Semi,
            ',' => Token::Comma,
            '~' => Token::Tilde,
            '^' => Token::Caret,
            '@' => Token::At,
            '#' => Token::Hash,
            '`' => return self.template_literal(start),

            // ── Potentially multi-char operators ─────────────────────────────
            '+' => self.eat_eq(Token::PlusEq, Token::Plus),
            '%' => self.eat_eq(Token::PercentEq, Token::Percent),

            // !  !=  !==
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Token::BangEq }
                    else { Token::BangEqLoose }
                } else { Token::Bang }
            }

            // =  ==  ===  =>
            '=' => match self.peek() {
                Some('=') => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Token::EqEq }
                    else { Token::EqEqLoose }
                }
                Some('>') => { self.advance(); Token::FatArrow }
                _ => Token::Eq,
            },

            '<' => match self.peek() {
                Some('=') => { self.advance(); Token::LtEq }
                Some('<') => { self.advance(); Token::LtLt }
                _ => Token::Lt,
            },
            '>' => match self.peek() {
                Some('=') => { self.advance(); Token::GtEq }
                Some('>') => { self.advance(); Token::GtGt }
                _ => Token::Gt,
            },
            '-' => match self.peek() {
                Some('>') => { self.advance(); Token::Arrow }
                Some('=') => { self.advance(); Token::MinusEq }
                _ => Token::Minus,
            },
            '*' => self.eat_eq(Token::StarEq, Token::Star),
            '&' => self.eat_ch('&', Token::AmpAmp, Token::Amp),
            '|' => self.eat_ch('|', Token::PipePipe, Token::Pipe),
            ':' => self.eat_ch(':', Token::ColonColon, Token::Colon),
            '.' => match self.peek() {
                Some('.') => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Token::DotDotEq }
                    else { Token::DotDot }
                }
                _ => Token::Dot,
            },
            '?' => Token::Question,

            // ${ — template literal interpolation (only valid inside a template literal,
            // but we tokenise it here so the parser/template literal handler can use it)
            '$' if self.peek() == Some('{') => {
                self.advance();
                Token::DollarLBrace
            }

            '/' => match self.peek() {
                Some('/') => self.line_comment(),
                Some('*') => self.block_comment(start),
                Some('=') => { self.advance(); Token::SlashEq }
                _ => Token::Slash,
            },

            '"' => self.string_literal(start),
            '\'' => self.char_literal(start),

            c if c.is_ascii_digit() => self.number(c, start),
            c if c.is_alphabetic() || c == '_' => self.ident_or_keyword(start),

            c => {
                self.errors.push(LexError::UnexpectedChar { ch: c, span: self.span(start) });
                Token::Error(c)
            }
        };

        Spanned::new(tok, self.span(start))
    }

    fn skip_whitespace(&mut self) {
        self.eat_while(|c| c.is_whitespace());
    }

    fn eat_eq(&mut self, with_eq: Token, without: Token) -> Token {
        if self.peek() == Some('=') { self.advance(); with_eq } else { without }
    }

    fn eat_ch(&mut self, expected: char, with: Token, without: Token) -> Token {
        if self.peek() == Some(expected) { self.advance(); with } else { without }
    }

    fn line_comment(&mut self) -> Token {
        let start = self.pos - 1;
        self.eat_while(|c| c != '\n');
        Token::LineComment(SmolStr::new(self.current_slice(start)))
    }

    fn block_comment(&mut self, start: usize) -> Token {
        self.advance(); // consume opening '*'
        let mut depth = 1usize;
        loop {
            match self.advance() {
                None => {
                    self.errors.push(LexError::UnterminatedBlockComment { span: self.span(start) });
                    break;
                }
                Some('*') if self.peek() == Some('/') => {
                    self.advance();
                    depth -= 1;
                    if depth == 0 { break; }
                }
                Some('/') if self.peek() == Some('*') => {
                    self.advance();
                    depth += 1;
                }
                _ => {}
            }
        }
        Token::BlockComment(SmolStr::new(self.current_slice(start)))
    }

    fn string_literal(&mut self, start: usize) -> Token {
        let mut value = String::new();
        loop {
            match self.advance() {
                None | Some('\n') => {
                    self.errors.push(LexError::UnterminatedString { span: self.span(start) });
                    break;
                }
                Some('"') => break,
                Some('\\') => {
                    if let Some(ch) = self.parse_escape(start) { value.push(ch); }
                }
                Some(c) => value.push(c),
            }
        }
        Token::StrLit(SmolStr::new(value))
    }

    fn char_literal(&mut self, start: usize) -> Token {
        let ch = match self.advance() {
            Some('\\') => match self.parse_escape(start) {
                Some(c) => c,
                None => {
                    return Spanned::new(Token::Error('\''), self.span(start)).node;
                }
            },
            Some(c) => c,
            None => {
                self.errors.push(LexError::UnterminatedString { span: self.span(start) });
                return Token::Error('\'');
            }
        };
        if self.peek() == Some('\'') { self.advance(); }
        else { self.errors.push(LexError::UnterminatedString { span: self.span(start) }); }
        Token::CharLit(ch)
    }

    fn parse_escape(&mut self, lit_start: usize) -> Option<char> {
        let esc_start = self.pos - 1;
        match self.advance() {
            Some('n')  => Some('\n'),
            Some('t')  => Some('\t'),
            Some('r')  => Some('\r'),
            Some('\\') => Some('\\'),
            Some('"')  => Some('"'),
            Some('\'') => Some('\''),
            Some('`')  => Some('`'),
            Some('0')  => Some('\0'),
            Some(c) => {
                self.errors.push(LexError::InvalidEscape { ch: c, span: self.span(esc_start) });
                None
            }
            None => {
                self.errors.push(LexError::UnterminatedString { span: self.span(lit_start) });
                None
            }
        }
    }

    /// Lex a full template literal starting right after the opening backtick.
    ///
    /// Segments between `${...}` blocks are returned as:
    ///   TemplateLitStart(text before first ${)
    ///   ...tokens for the expression...
    ///   TemplateLitMid(text before next ${)   [repeated]
    ///   ...tokens for expression...
    ///   TemplateLitEnd(text after last })
    ///
    /// The caller receives all of these as separate `Spanned<Token>` entries
    /// interleaved in the token stream.
    fn template_literal(&mut self, _start: usize) -> Spanned<Token> {
        let seg_start = self.pos;
        let seg = self.read_template_segment();
        let span = self.span(seg_start.saturating_sub(1)); // include opening `

        // Determine whether there's an interpolation following
        if self.peek() == Some('$') && self.peek2() == Some('{') {
            self.advance(); // $
            self.advance(); // {
            Spanned::new(Token::TemplateLitStart(seg), span)
        } else {
            // No interpolation — the whole literal is the end segment
            Spanned::new(Token::TemplateLitEnd(seg), span)
        }
    }

    /// Read characters until we hit `${` or a closing backtick.
    /// Handles `\${` as a literal `${` (escaped).
    fn read_template_segment(&mut self) -> SmolStr {
        let mut buf = String::new();
        loop {
            match self.peek() {
                None => break,
                Some('`') => { self.advance(); break; }
                Some('$') if self.peek2() == Some('{') => break, // interpolation ahead
                Some('\\') => {
                    self.advance();
                    // \` → `, \${ → ${ (don't start interpolation)
                    match self.peek() {
                        Some('`')  => { self.advance(); buf.push('`'); }
                        Some('$')  => { self.advance(); buf.push('$'); }
                        Some('n')  => { self.advance(); buf.push('\n'); }
                        Some('t')  => { self.advance(); buf.push('\t'); }
                        Some('\\') => { self.advance(); buf.push('\\'); }
                        Some(c)    => { let c = c; self.advance(); buf.push('\\'); buf.push(c); }
                        None       => break,
                    }
                }
                Some(c) => { let c = c; self.advance(); buf.push(c); }
            }
        }
        SmolStr::new(buf)
    }

    fn number(&mut self, first: char, start: usize) -> Token {
        if first == '0' {
            match self.peek() {
                Some('x') | Some('X') => {
                    self.advance();
                    self.eat_while(|c| c.is_ascii_hexdigit() || c == '_');
                    let s = self.current_slice(start).replace('_', "");
                    let val = i64::from_str_radix(&s[2..], 16).unwrap_or_else(|_| {
                        self.errors.push(LexError::IntegerOverflow { span: self.span(start) });
                        0
                    });
                    return Token::IntLit(val);
                }
                Some('b') | Some('B') => {
                    self.advance();
                    self.eat_while(|c| c == '0' || c == '1' || c == '_');
                    let s = self.current_slice(start).replace('_', "");
                    let val = i64::from_str_radix(&s[2..], 2).unwrap_or_else(|_| {
                        self.errors.push(LexError::IntegerOverflow { span: self.span(start) });
                        0
                    });
                    return Token::IntLit(val);
                }
                _ => {}
            }
        }

        self.eat_while(|c| c.is_ascii_digit() || c == '_');

        let is_float = self.peek() == Some('.')
            && self.peek2().map_or(false, |c| c.is_ascii_digit());

        if is_float {
            self.advance();
            self.eat_while(|c| c.is_ascii_digit() || c == '_');
            if matches!(self.peek(), Some('e') | Some('E')) {
                self.advance();
                if matches!(self.peek(), Some('+') | Some('-')) { self.advance(); }
                self.eat_while(|c| c.is_ascii_digit());
            }
            let s = self.current_slice(start).replace('_', "");
            Token::FloatLit(s.parse().unwrap_or(0.0))
        } else {
            let s = self.current_slice(start).replace('_', "");
            s.parse::<i64>().map(Token::IntLit).unwrap_or_else(|_| {
                self.errors.push(LexError::IntegerOverflow { span: self.span(start) });
                Token::IntLit(0)
            })
        }
    }

    fn ident_or_keyword(&mut self, start: usize) -> Token {
        self.eat_while(|c| c.is_alphanumeric() || c == '_');
        let word = self.current_slice(start);
        match word {
            "fn"       => Token::Fn,
            "let"      => Token::Let,
            "const"    => Token::Const,
            "if"       => Token::If,
            "else"     => Token::Else,
            "match"    => Token::Match,
            "return"   => Token::Return,
            "struct"   => Token::Struct,
            "enum"     => Token::Enum,
            "impl"     => Token::Impl,
            "trait"    => Token::Trait,
            "for"      => Token::For,
            "in"       => Token::In,
            "while"    => Token::While,
            "loop"     => Token::Loop,
            "break"    => Token::Break,
            "continue" => Token::Continue,
            "import"   => Token::Import,
            "export"   => Token::Export,
            "mod"      => Token::Mod,
            "type"     => Token::Type,
            "as"       => Token::As,
            "async"    => Token::Async,
            "await"    => Token::Await,
            "macro"    => Token::Macro,
            "where"    => Token::Where,
            "this"     => Token::This,
            "true"     => Token::BoolLit(true),
            "false"    => Token::BoolLit(false),
            // Primitive type keywords
            "i32"      => Token::KwI32,
            "i64"      => Token::KwI64,
            "f32"      => Token::KwF32,
            "f64"      => Token::KwF64,
            "boolean"  => Token::KwBoolean,
            "string"   => Token::KwString,
            "char"     => Token::KwChar,
            "void"     => Token::KwVoid,
            "number"   => Token::KwNumber,
            other      => Token::Ident(SmolStr::new(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Vec<Token> {
        let (toks, _errs) = Lexer::new(src, 0).tokenize();
        toks.into_iter().map(|s| s.node).filter(|t| !t.is_trivia()).collect()
    }

    #[test]
    fn hello_world_tokens() {
        let src = r#"fn main(): void { return; }"#;
        let toks = lex(src);
        assert!(matches!(toks[0], Token::Fn));
        assert!(matches!(toks[1], Token::Ident(_)));
        assert!(matches!(toks[2], Token::LParen));
    }

    #[test]
    fn integer_literals() {
        let src = "42 0xFF 0b1010 1_000";
        let toks = lex(src);
        assert_eq!(toks[0], Token::IntLit(42));
        assert_eq!(toks[1], Token::IntLit(255));
        assert_eq!(toks[2], Token::IntLit(10));
        assert_eq!(toks[3], Token::IntLit(1000));
    }

    #[test]
    fn string_escape() {
        let (toks, errs) = Lexer::new(r#""hello\nworld""#, 0).tokenize();
        assert!(errs.is_empty());
        assert_eq!(toks[0].node, Token::StrLit(SmolStr::new("hello\nworld")));
    }

    #[test]
    fn strict_equality_operators() {
        let src = "a === b !== c == d != e";
        let toks = lex(src);
        assert!(matches!(toks[1], Token::EqEq));
        assert!(matches!(toks[3], Token::BangEq));
        assert!(matches!(toks[5], Token::EqEqLoose));
        assert!(matches!(toks[7], Token::BangEqLoose));
    }

    #[test]
    fn new_keywords() {
        let src = "import export this boolean string where";
        let toks = lex(src);
        assert!(matches!(toks[0], Token::Import));
        assert!(matches!(toks[1], Token::Export));
        assert!(matches!(toks[2], Token::This));
        assert!(matches!(toks[3], Token::KwBoolean));
        assert!(matches!(toks[4], Token::KwString));
        assert!(matches!(toks[5], Token::Where));
    }

    #[test]
    fn template_literal_simple() {
        let src = "`hello world`";
        let toks = lex(src);
        // A template literal with no interpolation → TemplateLitEnd
        assert!(matches!(toks[0], Token::TemplateLitEnd(_)));
    }

    #[test]
    fn fat_arrow_closure() {
        // (  x  )  =>  x  *  2
        //  0  1  2   3  4  5  6
        let src = "(x) => x * 2";
        let toks = lex(src);
        assert!(matches!(toks[0], Token::LParen));
        assert!(matches!(toks[1], Token::Ident(_)));
        assert!(matches!(toks[2], Token::RParen));
        assert!(matches!(toks[3], Token::FatArrow));
    }
}
