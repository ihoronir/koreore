use crate::cursor::{cursor, Cursor};
use anyhow::Result;
use std::{fs::File, io::BufReader};

mod cursor;

#[derive(Clone, Copy, Debug)]
struct Char {
    line_num: usize,
    row_num: usize,
    c: char,
}

struct Token {
    line_num: usize,
    row_num: usize,
    token_kind: TokenKind,
}

#[derive(Debug)]
enum ReservedKind {
    Type,
    Enum,
    Logic,
}

fn detect_reserved(word: &str) -> Option<ReservedKind> {
    match word {
        "type" => Some(ReservedKind::Type),
        "enum" => Some(ReservedKind::Enum),
        "logic" => Some(ReservedKind::Logic),
        _ => None,
    }
}

#[derive(Debug)]
enum TokenKind {
    /// example) "// comment"
    Comment,

    /// '\t', '\n', '\r', ' '
    Whitespace,

    /// example) "Bus"
    Ident(String),

    /// "type", "enum", "logic", ...
    Reserved(ReservedKind),

    /// example) "8"
    Number(u32),

    /// example) "\"_@\""
    Literal { bitwidth: u32, value: u32 },

    // One-char tokens:
    /// ";"
    Semi,
    /// ","
    Comma,
    /// "."
    Dot,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,
    /// "@"
    At,
    /// "#"
    Pound,
    /// "~"
    Tilde,
    /// "?"
    Question,
    /// ":"
    Colon,
    /// "$"
    Dollar,
    /// "="
    Eq,
    /// "!"
    Bang,
    /// "<"
    Lt,
    /// ">"
    Gt,
    /// "-"
    Minus,
    /// "&"
    And,
    /// "|"
    Or,
    /// "+"
    Plus,
    /// "*"
    Star,
    /// "/"
    Slash,
    /// "^"
    Caret,
    /// "%"
    Percent,
}

fn scan(cur: &mut Cursor<impl Iterator<Item = Char>>) -> Option<Token> {
    let char = cur.next()?;

    let token_kind = match char.c {
        '\t' | '\n' | '\r' | ' ' => {
            cur.skip(|c| matches!(c, '\t' | '\n' | '\r' | ' '));
            TokenKind::Whitespace
        }
        ';' => TokenKind::Semi,
        ',' => TokenKind::Comma,
        '.' => TokenKind::Dot,
        '(' => TokenKind::OpenParen,
        ')' => TokenKind::CloseParen,
        '{' => TokenKind::OpenBrace,
        '}' => TokenKind::CloseBrace,
        '[' => TokenKind::OpenBracket,
        ']' => TokenKind::CloseBracket,
        '@' => TokenKind::At,
        '#' => TokenKind::Pound,
        '~' => TokenKind::Tilde,
        '?' => TokenKind::Question,
        ':' => TokenKind::Colon,
        '$' => TokenKind::Dollar,
        '=' => TokenKind::Eq,
        '!' => TokenKind::Bang,
        '<' => TokenKind::Lt,
        '>' => TokenKind::Gt,
        '-' => TokenKind::Minus,
        '&' => TokenKind::And,
        '|' => TokenKind::Or,
        '+' => TokenKind::Plus,
        '*' => TokenKind::Star,
        '/' => {
            if cur.consume('/') {
                cur.skip(|c| c != '\n');
                TokenKind::Comment
            } else {
                TokenKind::Slash
            }
        }
        '^' => TokenKind::Caret,
        '%' => TokenKind::Percent,

        _ => {
            if char.c.is_ascii_alphabetic() {
                scan_ident_or_reserved(cur, char)
            } else if char.c.is_ascii_digit() {
                scan_number(cur, char)
            } else if char.c == '"' {
                scan_literal(cur)
            } else {
                unimplemented!()
            }
        }
    };

    Some(Token {
        token_kind,
        line_num: char.line_num,
        row_num: char.row_num,
    })
}

fn scan_ident_or_reserved(cur: &mut Cursor<impl Iterator<Item = Char>>, first: Char) -> TokenKind {
    let mut word = first.c.to_string();

    cur.skip(|c| {
        if c.is_ascii_alphanumeric() || c == '_' {
            word.push(c);
            true
        } else {
            false
        }
    });

    if let Some(reserved_kind) = detect_reserved(&word) {
        TokenKind::Reserved(reserved_kind)
    } else {
        TokenKind::Ident(word)
    }
}

fn scan_number(cur: &mut Cursor<impl Iterator<Item = Char>>, first: Char) -> TokenKind {
    let mut digits = first.c.to_string();

    cur.skip(|c| {
        if c.is_ascii_digit() {
            digits.push(c);
            true
        } else {
            false
        }
    });

    TokenKind::Number(digits.parse().unwrap())
}

fn scan_literal(cur: &mut Cursor<impl Iterator<Item = Char>>) -> TokenKind {
    let mut bits = String::new();
    let mut bitwidth = 0;

    cur.skip(|c| match c {
        '_' => {
            bits.push('0');
            bitwidth += 1;
            true
        }
        '@' => {
            bits.push('1');
            bitwidth += 1;
            true
        }
        '?' => unimplemented!(),
        '"' => true,
        _ => false,
    });

    TokenKind::Literal {
        bitwidth,
        value: u32::from_str_radix(&bits, 2).unwrap(),
    }
}

fn tokenize() -> Result<()> {
    let file = File::open("computer.kror")?;
    let buf_reader = BufReader::new(file);

    let mut cur = cursor(buf_reader)?;

    while let Some(token) = scan(&mut cur) {
        println!("{:?}", (token.line_num, token.row_num, token.token_kind));
    }

    Ok(())
}

fn main() -> Result<()> {
    tokenize()?;

    Ok(())
}
