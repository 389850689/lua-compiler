use crate::lexer::Token;
use crate::{log_error, term_color::*};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    errored: bool,
}

enum ASTNode {
    Chunk(Vec<ASTNode>, Vec<ASTNode>),
    Block(Box<ASTNode>),
    Statement(Box<ASTNode>),
    FunctionCall(Box<ASTNode>),
    Do(Box<ASTNode>),
    While {
        // this would be the condition.
        expression: Box<ASTNode>,
        do_block: Box<ASTNode>,
    },
    Repeat {
        block: Box<ASTNode>,
        // this would be the end condition.
        expression: Box<ASTNode>,
    },
    If {
        // this would be the if condition.
        expression: Box<ASTNode>,
        block: Box<ASTNode>,
        elseif: Vec<ASTNode>,
        then_else: Option<Box<ASTNode>>,
    },
    ForNumeric {
        name: Box<ASTNode>,
        from_expression: Box<ASTNode>,
        to_expression: Box<ASTNode>,
        step_expression: Option<Box<ASTNode>>,
        do_block: Box<ASTNode>,
    },
    ForGeneric {
        name_list: Box<ASTNode>,
        expression_list_1: Box<ASTNode>,
        do_block: Box<ASTNode>,
    },
    Function {
        function_name: Box<ASTNode>,
        function_body: Box<ASTNode>,
    },
    LocalFunction {
        // NOTE: this is not a funcname, rather a NAME.
        function_name: Box<ASTNode>,
        function_body: Box<ASTNode>,
    },
    LocalVariable {
        name_list: Box<ASTNode>,
        // this would be the assignment of a variable.
        expression_list: Option<Box<ASTNode>>,
    },
    Name(String),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            errored: false,
        }
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    /// Checks then next token.
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.cursor + 1).cloned()
    }

    /// Returns the current token.
    fn current(&self) -> Token {
        self.tokens.get(self.cursor).cloned().unwrap_or_default()
    }

    fn is_match(&self, token: Token) -> bool {
        // !self.is_eof() && self.peek().unwrap_or_default() == token
        !self.is_eof() && self.current() == token
    }

    fn advance(&mut self) {
        self.cursor += 1;
    }

    fn accept(&mut self, token: Token) -> bool {
        if self.is_match(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: Token) {
        if !self.accept(token) {
            log_error!(
                "[{}] unexpected symbol: {:?}",
                colored("parser", Color::Grey),
                self.current(),
            );
            self.errored = true;
        }
    }

    // parse an expression.
    fn exp(&mut self) -> bool {
        let found_terminal = match self.current() {
            Token::NIL | Token::FALSE | Token::TRUE | Token::DOTS => true,
            Token::NUMBER(_) => true,
            Token::STRING(_) => true,
            _ => false,
        };

        if found_terminal {
            self.advance();
            return found_terminal;
        }

        false
    }

    fn stat(&mut self) -> bool {
        if self.accept(Token::DO) {
            self.block();
            self.expect(Token::END);
            return true;
        }

        false
    }

    fn block(&mut self) {
        self.chunk();
    }

    fn chunk(&mut self) {
        self.stat();
    }

    pub fn parse(&mut self) -> Option<()> {
        let symbol = self.current();
        self.chunk();
        println!("{:?}", symbol);

        if self.errored {
            None
        } else {
            Some(())
        }
    }
}
