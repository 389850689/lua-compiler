use std::{collections::HashMap, hash::Hash};

use crate::{log_error, term_color::*};

trait StrExt {
    fn remove_last(&self) -> &str;
}

impl StrExt for str {
    fn remove_last(&self) -> &str {
        match self.char_indices().next_back() {
            Some((i, _)) => &self[..i],
            None => self,
        }
    }
}

type Tokens = Vec<Token>;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Token {
    AND,
    END,
    BREAK,
    DO,
    ELSE,
    ELSEIF,
    FALSE,
    FOR,
    FUNCTION,
    IF,
    IN,
    LOCAL,
    NIL,
    NOT,
    OR,
    REPEAT,
    RETURN,
    THEN,
    TRUE,
    UNTIL,
    WHILE,
    NUMBER(f64),
    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACKET,
    RIGHT_BRACKET,
    LEFT_BRACE,
    RIGHT_BRACE,
    GREATER_THAN,
    LESS_THAN,
    GREATER_EQUAL,
    LESS_EQUAL,
    CONCAT,
    DOTS,
    STRING(String),
    NAME(String),
    XOR,
    MODULO,
    HASHTAG,
    ASSIGN,
    EQ,
    NEQ,
    SEMICOLON,
    COLON,
    COMMA,
    DOT,
    UNDEFINED,
}

fn is_end_of_line(c: char) -> bool {
    match c {
        '\n' => true,
        _ => false,
    }
}

/// This represents the state of our Lexer sa it's tokenizing the tape.
pub struct Lexer {
    tape: String,
    cursor: isize,
    line: usize,
    errored: bool,
}

impl Lexer {
    pub fn new(text: &str) -> Self {
        // starting at negative index is a little bit of a hack to make the code be slightly nicer.
        Self {
            line: 1,
            errored: false,
            tape: text.to_string(),
            cursor: -1,
        }
    }

    /// This will return true if the cursor is past the last character of the tape.
    fn is_end_of_file(&self) -> bool {
        self.cursor as usize >= self.tape.len()
    }

    /// This will return true if n is past the last character of the tape.
    fn is_end_of_file_nth(&self, n: isize) -> bool {
        n as usize >= self.tape.len()
    }

    /// Advances the cursor by one then returns the consumed character.
    fn advance(&mut self) -> Option<char> {
        // increase our internal cursor by one.
        self.cursor = self.cursor + 1;

        if self.is_end_of_file() {
            // if we're at the end of the file we can't advance.
            return None;
        }

        // we know this will work since we do the bounds checking ourselves.
        Some(self.tape.chars().nth(self.cursor as _).unwrap())
    }

    fn advance_nth(&mut self, n: isize) -> Option<char> {
        for _ in 0..n {
            self.advance();
        }
        None
    }

    /// This checks the next character in the tape but doesn't consume it.
    fn peek(&self) -> Option<char> {
        self.tape.chars().nth(self.cursor as usize + 1)
    }

    /// This checks an arbitrary character in the tape but doesn't consume it.
    fn peek_nth(&self, n: isize) -> Option<char> {
        self.tape.chars().nth(self.cursor as usize + n as usize)
    }

    /// Creates a substring given start on the tape, and the size.
    fn sub_tape(&self, start: usize, size: usize) -> String {
        self.tape[start..start + size].to_string()
    }

    /// This will continue peaking until it can no longer peak.
    fn while_peek<F: Fn(char) -> bool, P: Fn(char, usize) -> bool>(
        &self,
        p: P,
        f: F,
    ) -> (isize, String) {
        let mut stack: String = String::new();
        let mut current_peek = 1;
        loop {
            // if there isn't a next character just return from the peek.
            let current_char = match self.peek_nth(current_peek) {
                Some(c) => c,
                None => break,
            };

            // push the newest character to thet stack.
            stack.push(current_char);

            if p(current_char, current_peek as _) {
                break;
            }

            if !f(current_char) {
                break;
            };

            current_peek += 1;
        }
        (current_peek, stack)
    }

    /// Returns the char the cursor is currently pointing over
    // fn current_char(&self) -> char {
    //     // we know this can never fail
    //     self.tape.chars().nth(self.cursor as usize).unwrap()
    // }

    /// This transforms a string into a list of parsable tokens.
    pub fn tokenize(&mut self) -> Option<Tokens> {
        // store a list of tokens that we've found while lexing.
        let mut tokens: Tokens = Vec::new();

        let keywords = HashMap::from([
            ("and", Token::AND),
            ("or", Token::OR),
            ("while", Token::WHILE),
            ("for", Token::FOR),
            ("repeat", Token::REPEAT),
            ("return", Token::RETURN),
            ("then", Token::THEN),
            ("true", Token::TRUE),
            ("until", Token::UNTIL),
            ("function", Token::FUNCTION),
            ("if", Token::IF),
            ("in", Token::IN),
            ("local", Token::LOCAL),
            ("nil", Token::NIL),
            ("end", Token::END),
            ("break", Token::BREAK),
            ("do", Token::DO),
            ("else", Token::ELSE),
            ("elseif", Token::ELSEIF),
            ("false", Token::FALSE),
            ("not", Token::NOT),
        ]);

        // while we can still read characters from the tape.
        while let Some(c) = self.advance() {
            // ignore characters that don't care about.
            if c.is_whitespace() && !is_end_of_line(c) {
                continue;
            }

            if is_end_of_line(c) {
                self.line += 1;
                continue;
            }

            if c == '"' || c == '\'' {
                // collect the stack of chars into a string.
                let (mut n, string) = self.while_peek(
                    |c, n| {
                        self.sub_tape((self.cursor as usize + n) - 2, 3) != "\\\r\n"
                            && is_end_of_line(c)
                    },
                    |c| !(c == '"'),
                );

                // so this is a bool set if the peek is at the end of the line.
                let end_of_line = is_end_of_line(string.chars().last().unwrap());

                if self.is_end_of_file_nth(self.cursor + n) || end_of_line {
                    log_error!(
                        "[{}] unclosed string, starting at column {}, line {}.",
                        colored("token", Color::Grey),
                        self.cursor,
                        self.line
                    );
                    self.errored = true;
                    // we subtract two to account for the CRLF.
                    if end_of_line {
                        n -= 2;
                    }
                } else {
                    let string = &string[..].remove_last();
                    tokens.push(Token::STRING(string.to_string()));
                }

                self.advance_nth(n);
                continue;
            }

            // since numbers can be more then 1 character long we will handle it separately.
            if c.is_numeric() || c == '.' {
                // this is a local buffer where we'll put our incomplete number.
                let mut buffer = format!("{c}");
                loop {
                    // if there is another character in the tape.
                    if let Some(c) = self.peek() {
                        if c.is_numeric() || c == '.' {
                            // we know that there is another character so the unwrap will never fail.
                            buffer.push(self.advance().unwrap())
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                // count the number of points in a number.
                if buffer.chars().filter(|&p| p == '.').count() > 1 {
                    log_error!("invalid number at column {}.", self.cursor - 1);
                    std::process::exit(-1);
                }
                // turn the string into a number.
                let number = buffer.parse::<f64>().unwrap();
                // add the number to the tokens list.
                tokens.push(Token::NUMBER(number));
                continue;
            }

            let token = match c {
                '+' => Token::ADD,
                '-' => Token::SUBTRACT,
                '*' => Token::MULTIPLY,
                '/' => Token::DIVIDE,
                '(' => Token::LEFT_PAREN,
                ')' => Token::RIGHT_PAREN,
                '^' => Token::XOR,
                '.' => Token::DOT,
                ',' => Token::COMMA,
                '#' => Token::HASHTAG,
                ';' => Token::SEMICOLON,
                ':' => Token::COLON,
                ']' => Token::LEFT_BRACKET,
                '[' => Token::RIGHT_BRACKET,
                '{' => Token::LEFT_BRACE,
                '}' => Token::RIGHT_BRACE,
                // '<' => Token::LESS_THAN,
                // '>' => Token::GREATER_THAN,
                '%' => Token::MODULO,
                _ => Token::UNDEFINED,
            };

            if token == Token::UNDEFINED {
                // show an error message to the user if we don't know what they input.
                log_error!(
                    "[{}] undefined token '{c}' at column {}, line {}.",
                    colored("token", Color::Grey),
                    self.cursor,
                    self.line
                );
                self.errored = true;
            }

            tokens.push(token)
        }

        // if there was an error during lexing we still want to show all the error messages at
        // once.
        if self.errored {
            return None;
        }

        Some(tokens)
    }
}
