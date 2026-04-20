use smol_str::SmolStr;

/// Every token the Selvr lexer can produce.
///
/// Naming convention:
///   - Keyword variants use the keyword name (e.g. `Fn`, `Let`, `Match`).
///   - Punctuation / operators use descriptive names (e.g. `Arrow`, `FatArrow`).
///   - Literals carry their parsed value inline.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Literals ──────────────────────────────────────────────────────────────
    /// Integer literal, e.g. `42`, `0xFF`, `0b1010`, `1_000_000`
    IntLit(i64),
    /// Floating-point literal, e.g. `3.14`, `1.0e-9`
    FloatLit(f64),
    /// Boolean literal
    BoolLit(bool),
    /// Plain string literal (double-quoted, contents already un-escaped)
    StrLit(SmolStr),
    /// Template literal segment — the raw text between `${` interpolations.
    /// The lexer breaks `` `Hello ${name}!` `` into:
    ///   TemplateLitStart("Hello ") · Expr · TemplateLitMid("!") · TemplateLitEnd
    TemplateLitStart(SmolStr),
    TemplateLitMid(SmolStr),
    TemplateLitEnd(SmolStr),
    /// Character literal, e.g. `'a'`
    CharLit(char),

    // ── Identifiers & keywords ────────────────────────────────────────────────
    Ident(SmolStr),

    // ── Keywords ──────────────────────────────────────────────────────────────
    Fn,
    Let,
    Const,
    If,
    Else,
    Match,
    Return,
    Struct,
    Enum,
    Impl,
    Trait,
    For,
    In,
    While,
    Loop,
    Break,
    Continue,
    Import,     // import { X } from "module"
    Export,     // export fn / export const / export struct
    Mod,
    Type,
    As,
    Async,
    Await,
    Macro,
    Where,
    This,       // `this` in method bodies — replaces `self`

    // ── Built-in type keywords ─────────────────────────────────────────────────
    KwI32,
    KwI64,
    KwF32,
    KwF64,
    KwBoolean,  // `boolean`  (TypeScript naming)
    KwString,   // `string`   (TypeScript naming)
    KwChar,
    KwVoid,
    KwNumber,   // `number` alias (maps to f64 internally)

    // ── Operators ─────────────────────────────────────────────────────────────
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Amp,         // &
    Pipe,        // |
    Caret,       // ^
    Tilde,       // ~
    Bang,        // !
    Eq,          // =
    EqEq,        // ===   (strict equality — TypeScript style)
    EqEqLoose,   // ==    (kept for compatibility but discouraged)
    BangEq,      // !==   (strict inequality)
    BangEqLoose, // !=
    Lt,          // <
    LtEq,        // <=
    Gt,          // >
    GtEq,        // >=
    LtLt,        // <<
    GtGt,        // >>
    PlusEq,      // +=
    MinusEq,     // -=
    StarEq,      // *=
    SlashEq,     // /=
    PercentEq,   // %=
    AmpAmp,      // &&
    PipePipe,    // ||
    Arrow,       // ->  (kept for function type signatures)
    FatArrow,    // =>  (closures and match arms)
    DotDot,      // ..
    DotDotEq,    // ..=
    Question,    // ?
    At,          // @

    // ── Delimiters ────────────────────────────────────────────────────────────
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    DollarLBrace, // ${  (template literal interpolation start)

    // ── Punctuation ───────────────────────────────────────────────────────────
    Semi,      // ;
    Colon,     // :
    ColonColon,// ::
    Comma,     // ,
    Dot,       // .
    Hash,      // #
    Backtick,  // `  (template literal delimiter)

    // ── Trivia (skipped by the parser but kept for IDE tools) ─────────────────
    LineComment(SmolStr),
    BlockComment(SmolStr),

    // ── Special ───────────────────────────────────────────────────────────────
    /// Produced when the lexer encounters an unrecognised character.
    Error(char),
    /// End of file.
    Eof,
}

impl Token {
    pub fn is_trivia(&self) -> bool {
        matches!(self, Token::LineComment(_) | Token::BlockComment(_))
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::IntLit(_)
                | Token::FloatLit(_)
                | Token::BoolLit(_)
                | Token::StrLit(_)
                | Token::CharLit(_)
        )
    }

    /// Human-readable description used in error messages.
    pub fn describe(&self) -> &'static str {
        match self {
            Token::IntLit(_)           => "integer literal",
            Token::FloatLit(_)         => "float literal",
            Token::BoolLit(_)          => "boolean literal",
            Token::StrLit(_)           => "string literal",
            Token::TemplateLitStart(_) => "template literal",
            Token::CharLit(_)          => "character literal",
            Token::Ident(_)            => "identifier",
            Token::Fn                  => "`fn`",
            Token::Let                 => "`let`",
            Token::Const               => "`const`",
            Token::If                  => "`if`",
            Token::Else                => "`else`",
            Token::Match               => "`match`",
            Token::Return              => "`return`",
            Token::Struct              => "`struct`",
            Token::Enum                => "`enum`",
            Token::Impl                => "`impl`",
            Token::Trait               => "`trait`",
            Token::For                 => "`for`",
            Token::In                  => "`in`",
            Token::While               => "`while`",
            Token::Loop                => "`loop`",
            Token::Break               => "`break`",
            Token::Continue            => "`continue`",
            Token::Import              => "`import`",
            Token::Export              => "`export`",
            Token::Mod                 => "`mod`",
            Token::Type                => "`type`",
            Token::As                  => "`as`",
            Token::Async               => "`async`",
            Token::Await               => "`await`",
            Token::Macro               => "`macro`",
            Token::Where               => "`where`",
            Token::This                => "`this`",
            Token::Arrow               => "`->`",
            Token::FatArrow            => "`=>`",
            Token::EqEq                => "`===`",
            Token::BangEq              => "`!==`",
            Token::LtEq                => "`<=`",
            Token::GtEq                => "`>=`",
            Token::AmpAmp              => "`&&`",
            Token::PipePipe            => "`||`",
            Token::Semi                => "`;`",
            Token::Colon               => "`:`",
            Token::ColonColon          => "`::`",
            Token::Comma               => "`,`",
            Token::Dot                 => "`.`",
            Token::LParen              => "`(`",
            Token::RParen              => "`)`",
            Token::LBrace              => "`{`",
            Token::RBrace              => "`}`",
            Token::LBracket            => "`[`",
            Token::RBracket            => "`]`",
            Token::Backtick            => "`` ` ``",
            Token::DollarLBrace        => "`${`",
            Token::Eof                 => "end of file",
            Token::Error(_)            => "unexpected character",
            _                          => "token",
        }
    }
}
