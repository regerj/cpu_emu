use std::{iter::Peekable, str::Chars};

use anyhow::{Context, Result, bail};

trait PeekingCollectWhile: Iterator {
    fn collect_while(&mut self, p: impl FnMut(&Self::Item) -> bool) -> String;
}

impl PeekingCollectWhile for Peekable<Chars<'_>> {
    fn collect_while(&mut self, mut p: impl FnMut(&Self::Item) -> bool) -> String {
        let mut ret = String::new();
        while let Some(ch) = self.peek() {
            if p(ch) {
                // Safe unwrap
                ret.push(self.next().unwrap());
            } else {
                break;
            }
        }

        ret
    }
}

#[derive(Debug)]
pub enum Token {
    LabelDecl(String),
    LabelInvoc(String),
    Register(String),
    Immediate(String),
    Mneumonic(String),
    Comma,
    Comment(String),
    Deref,
}

enum State {
    BEGIN,
    ARGS,
}

pub fn tokenize(str: String) -> Result<Vec<Token>> {
    let mut chars = str.chars().peekable();
    let mut tokens = Vec::new();
    let mut state = State::BEGIN;

    // Basic state machine
    while let Some(ch) = chars.peek() {
        match state {
            State::BEGIN => {
                // Newline no op
                if *ch == '\n' {
                    chars.next().context("Unexpected end of input")?;
                // Maybe comment
                } else if *ch == '/' {
                    chars.next().context("Unexpected end of input")?;
                    if chars.next().context("Unexpected end of input")? == '/' {
                        tokens.push(Token::Comment(chars.collect_while(|ch| *ch != '\n')));
                    } else {
                        bail!("Invalid syntax: single /");
                    }
                // Mneumonic or Label
                } else if ch.is_alphabetic() {
                    let s: String = chars.collect_while(|ch| !ch.is_whitespace());
                    if s.ends_with(':') {
                        tokens.push(Token::LabelDecl(s[0..s.len() - 1].to_string()));
                    } else {
                        tokens.push(Token::Mneumonic(s));
                        state = State::ARGS;
                    }
                }
            }
            State::ARGS => {
                if *ch == '\n' {
                    chars.next().context("Unexpected end of input")?;
                    state = State::BEGIN;
                // Skip all spaces
                } else if *ch == ' ' || *ch == '\t' {
                    chars.next().context("Unexpected end of input")?;
                    continue;
                } else if *ch == '*' {
                    chars.next().context("Unexpected end of input")?;
                    tokens.push(Token::Deref);
                // A alphabetic character means it must be a label
                } else if ch.is_alphabetic() {
                    tokens.push(Token::LabelInvoc(chars.collect_while(is_valid_label_ch)));
                // A $ means a register
                } else if *ch == '$' {
                    chars.next().context("Unexpected end of input")?;
                    tokens.push(Token::Register(chars.collect_while(|ch| ch.is_alphanumeric())));
                // A numeric means it must be an immediate
                } else if ch.is_numeric() {
                    tokens.push(Token::Immediate(chars.collect_while(|ch| ch.is_numeric())));
                } else if *ch == ',' {
                    chars.next().context("Unexpected end of input")?;
                    tokens.push(Token::Comma);
                }
                else {
                    bail!("Unexpected value while parsing for an argument: '{ch}'");
                }
            }
        }
    }

    Ok(tokens)
}

fn is_valid_label_ch(ch: &char) -> bool {
    ch.is_alphanumeric() || *ch == '_'
}
