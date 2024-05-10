use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

//struct Token {
//    line_num: usize,
//    row_num: usize,
//    token_kind: TokenKind,
//}
//
//enum TokenKind {
//    Literal,
//    Type,
//    Name,
//    Colon,
//    Semi,
//    Number,
//    SquareLeft,
//    SquareRight,
//}

#[derive(Debug)]
struct Char {
    line_num: usize,
    row_num: usize,
    c: char,
}

fn chars_iter<T: BufRead>(source: T) -> Result<impl Iterator<Item = Char>> {
    let lines: Result<Vec<_>, _> = source.lines().collect();

    Ok(lines?.into_iter().enumerate().flat_map(|(i, string)| {
        let mut line_vec = string.chars().collect::<Vec<_>>();
        line_vec.push('\n');

        println!("flag");

        line_vec.into_iter().enumerate().map(move |(j, c)| Char {
            line_num: i + 1,
            row_num: j + 1,
            c,
        })
    }))
}

fn main() -> Result<()> {
    let file = File::open("computer.kror")?;
    let buf_reader = BufReader::new(file);

    let chars = chars_iter(buf_reader)?.peekable();

    for char in chars {
        println!("{:?}", (char.line_num, char.row_num, char.c));
    }

    Ok(())
}
