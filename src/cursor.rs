use crate::Char;
use anyhow::Result;
use std::{io::BufRead, iter::Peekable};

pub struct Cursor<T: Iterator<Item = Char>> {
    iter: Peekable<T>,
}

pub fn cursor(source: impl BufRead) -> Result<Cursor<impl Iterator<Item = Char>>> {
    let lines = source.lines().collect::<Result<Vec<_>, _>>()?;

    let iter = lines
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
        .peekable();

    Ok(Cursor { iter })
}

impl<T: Iterator<Item = Char>> Cursor<T> {
    pub fn next(&mut self) -> Option<Char> {
        self.iter.next()
    }

    pub fn consume(&mut self, c: char) -> bool {
        self.iter.next_if(|char| char.c == c).is_some()
    }

    pub fn skip(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while {
            if let Some(char) = self.iter.peek() {
                predicate(char.c)
            } else {
                false
            }
        } {
            self.iter.next();
        }
    }
}
