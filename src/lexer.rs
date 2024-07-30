/// Tokens are simply as list that contains one or more `Token`.
pub type Tokens = Vec<Token>;

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
    Number(f64),
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
    STRING,
    NAME,
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

/// This represents the state of our Lexer sa it's tokenizing the tape.
pub struct Lexer {
    tape: String,
    cursor: isize,
}

impl Lexer {
    pub fn new(text: &str) -> Self {
        // starting at negative index is a little bit of a hack to make the code be slightly nicer.
        Self {
            tape: text.to_string(),
            cursor: -1,
        }
    }

    /// This will return true if the cursor is past the last character of the tape.
    fn is_end_of_file(&self) -> bool {
        self.cursor as usize >= self.tape.len()
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

    /// This checks the next character in the tape but doesn't consume it.
    fn peek(&self) -> Option<char> {
        self.tape.chars().nth(self.cursor as usize + 1)
    }

    /// Returns the char the cursor is currently pointing over
    // fn current_char(&self) -> char {
    //     // we know this can never fail
    //     self.tape.chars().nth(self.cursor as usize).unwrap()
    // }

    /// This transforms a string into a list of parsable tokens.
    pub fn tokenize(&mut self) -> Tokens {
        // store a list of tokens that we've found while lexing.
        let mut tokens: Tokens = Vec::new();

        // while we can still read characters from the tape.
        while let Some(c) = self.advance() {
            // ignore characters that don't care about.
            if c.is_whitespace() {
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
                    panic!("Error: invalid number at column {}.", self.cursor - 1);
                }
                // turn the string into a number.
                let number = buffer.parse::<f64>().unwrap();
                // add the number to the tokens list.
                tokens.push(Token::Number(number));
                continue;
            }

            let token = match c {
                '+' => Token::ADD,
                '-' => Token::SUBTRACT,
                '*' => Token::MULTIPLY,
                '/' => Token::DIVIDE,
                '(' => Token::LEFT_PAREN,
                ')' => Token::RIGHT_PAREN,
                _ => Token::UNDEFINED,
            };

            if token == Token::UNDEFINED {
                // show an error message to the user if we don't know what they input.
                panic!("Error: undefined token '{c}' at column {}.", self.cursor);
            }

            tokens.push(token)
        }

        tokens
    }
}
