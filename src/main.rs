use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
struct Char {
    line_num: usize,
    row_num: usize,
    c: char,
}

fn chars_iter<T: BufRead>(source: T) -> Result<Peekable<impl Iterator<Item = Char>>> {
    let lines = source.lines().collect::<Result<Vec<_>, _>>()?;

    Ok(lines
        .into_iter()
        .enumerate()
        .flat_map(|(i, string)| {
            let mut line_vec: Vec<_> = string.chars().collect();
            line_vec.push('\n');

            line_vec.into_iter().enumerate().map(move |(j, c)| Char {
                line_num: i + 1,
                row_num: j + 1,
                c,
            })
        })
        .peekable())
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

use std::iter::Peekable;

fn consume(cur: &mut Peekable<impl Iterator<Item = Char>>, c: char) -> Option<bool> {
    Some(cur.next_if(|char| char.c == c)?.c == c)
}

fn skip(cur: &mut Peekable<impl Iterator<Item = Char>>, mut predicate: impl FnMut(char) -> bool) {
    while {
        if let Some(char) = cur.peek() {
            predicate(char.c)
        } else {
            false
        }
    } {
        cur.next();
    }
}

fn scan(cur: &mut Peekable<impl Iterator<Item = Char>>) -> Option<Token> {
    let char = cur.next()?;

    let token_kind = match char.c {
        '\t' | '\n' | '\r' | ' ' => {
            skip(cur, |c| matches!(c, '\t' | '\n' | '\r' | ' '));
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
            if consume(cur, '/')? {
                skip(cur, |c| c != '\n');
                TokenKind::Comment
            } else {
                TokenKind::Slash
            }
        }
        '^' => TokenKind::Caret,
        '%' => TokenKind::Percent,

        _ => {
            // Ident or Reserved
            if char.c.is_ascii_alphabetic() {
                let mut word = char.c.to_string();

                skip(cur, |c| {
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
            } else
            // Number
            if char.c.is_ascii_digit() {
                let mut digits = char.c.to_string();

                skip(cur, |c| {
                    if c.is_ascii_digit() {
                        digits.push(c);
                        true
                    } else {
                        false
                    }
                });

                TokenKind::Number(digits.parse().unwrap())
            } else
            // Literal
            if char.c == '"' {
                let mut bits = String::new();
                let mut bitwidth = 0;

                skip(cur, |c| match c {
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

fn tokenize() -> Result<()> {
    let file = File::open("computer.kror")?;
    let buf_reader = BufReader::new(file);

    let mut cur = chars_iter(buf_reader)?;

    while let Some(token) = scan(&mut cur) {
        println!("{:?}", (token.line_num, token.row_num, token.token_kind));
    }

    Ok(())
}

fn main() -> Result<()> {
    tokenize()?;

    Ok(())
}
