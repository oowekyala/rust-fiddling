use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Write};
use std::io;
use std::io::prelude::*;
use std::result;
use regex::Regex;

/*
    A simple brainfuck interpreter.
 */

#[derive(Debug)]
struct Problem {
    message: String
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::convert::From<io::Error> for Problem {
    fn from(e: io::Error) -> Self {
        return Problem { message: e.to_string() }
    }
}

impl Error for Problem {}

fn main() -> result::Result<(), Problem> {
    let args: Vec<String> = env::args().collect();
    match &args[..] {
        [_, fname, ..] => {
            let file = File::open(fname)?;
            let mut reader = BufReader::new(file);
            let mut text = String::new();
            reader.read_to_string(&mut text)?;
            eval_bf(cleanup_program(&text).as_bytes(), 4096)
        }
        _ => Err(Problem { message: String::from("Usage: $ bf filename.bf") }),
    }
}

fn cleanup_program(text: &str) -> String {
    let r = Regex::new(r"[^.,<>\]\[+-]").unwrap();
    r.replace_all(text, "").to_string()
}

fn eval_bf(text: &[u8], memsize: usize) -> result::Result<(), Problem> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut text_idx: usize = 0;
    let mut loop_stack: Vec<usize> = Vec::new();
    let mut memory: Vec<u8> = vec![0; memsize];
    let mut ptr: usize = 0;
    while text_idx < text.len() {
        match text[text_idx] {
            b',' => { stdin.read_exact(&mut memory[ptr..=ptr])?; },
            b'.' => {
                stdout.write(&memory[ptr..=ptr])?; },
            b'+' => { memory[ptr] = memory[ptr].wrapping_add(1); },
            b'-' => { memory[ptr] = memory[ptr].wrapping_sub(1); },
            b'>' => { ptr = ptr.wrapping_add(1); },
            b'<' => { ptr = ptr.wrapping_sub(1); },
            b'[' => {
                if memory[ptr] == 0 {
                    text_idx = text_idx + skip_loop(&text[text_idx..])?
                } else {
                    loop_stack.push(text_idx)
                };
            },
            b']' => {
                text_idx = match loop_stack.pop() {
                    Some(i) => i,
                    None => {
                        return Err(Problem { message: String::from("Undeclared loop") })
                    }
                };
                continue
            },
            #[cold] _ => {} // pass
        };
        text_idx += 1;
    }
    Ok(())
}

fn skip_loop(text: &[u8]) -> result::Result<usize, Problem> {
    let mut depth = 0;
    for (i, c) in text.iter().enumerate() {
        match c {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i)
                }
            }
            _ => {} // pass
        }
    }
    Err(Problem { message: String::from("Unclosed loop!") })
}
