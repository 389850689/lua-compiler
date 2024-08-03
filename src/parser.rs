use crate::lexer::Token;
use crate::{log_error, term_color::*};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    errored: bool,
}

type MaybeASTNode = Option<ASTNode>;

#[derive(Clone, Debug)]
pub enum ASTNode {
    Chunk(Vec<ASTNode>, Option<Box<ASTNode>>),
    Block(Box<ASTNode>),
    Statement(Box<ASTNode>),
    Expression(Box<ASTNode>),
    FunctionCall(Box<ASTNode>),
    LValueAssign {
        var_list: Box<ASTNode>,
        expression_list: Box<ASTNode>,
    },
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
    Return(Option<Box<ASTNode>>),
    FunctionName {
        name: Box<ASTNode>,
        members: Vec<ASTNode>,
        colon: Option<Box<ASTNode>>,
    },
    VariableList {
        variable: Box<ASTNode>,
        tail_list: Vec<ASTNode>,
    },
    Variable(Box<ASTNode>),
    PrefixExpression(Box<ASTNode>),
    PrefixExpressionBrackets(Box<ASTNode>),
    PrefixExpressionDot(Box<ASTNode>),
    PrefixExpressionArgs {
        prefix_expression: Box<ASTNode>,
        arguments: Box<ASTNode>,
    },
    PrefixExpressionNameArgs {
        prefix_expression: Box<ASTNode>,
        name: Box<ASTNode>,
        arguments: Box<ASTNode>,
    },
    NameList {
        name: Box<ASTNode>,
        tail_list: Vec<ASTNode>,
    },
    BinaryOp {
        left: Box<ASTNode>,
        binary_operator: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    UnaryOp {
        unary_operator: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    ExpressionList {
        head_list: Vec<ASTNode>,
        expression: Box<ASTNode>,
    },
    ArgsParamList(Option<Box<ASTNode>>),
    FunctionBody {
        parameter_list: Option<Box<ASTNode>>,
        block: Box<ASTNode>,
    },
    ParameterList(Box<ASTNode>),
    // This is just one of the possible productions.
    ParameterListA {
        name_list: Box<ASTNode>,
        variadic: Box<ASTNode>,
    },
    TableConstructor(Box<ASTNode>),
    FieldList {
        field: Box<ASTNode>,
        fieldsep: Vec<(ASTNode, ASTNode)>,
        fieldsep_b: Option<Box<ASTNode>>,
    },
    Field(Box<ASTNode>),
    FieldA {
        expression_a: Box<ASTNode>,
        expression_b: Box<ASTNode>,
    },
    FieldB {
        name: Box<ASTNode>,
        expression: Box<ASTNode>,
    },
    Fieldsep(Box<ASTNode>),
    LastStatement(Box<ASTNode>),
    BinaryOperator(Box<ASTNode>),
    UnaryOperator(Box<ASTNode>),
    Name(String),
    Token(Token),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            errored: false,
        }
    }

    fn report_expected_error(&mut self, expected: &str) {
        log_error!(
            "[{}] expected {:?}, found {:?}.",
            colored("parser", Color::Grey),
            expected,
            self.current(),
        );
        self.errored = true;
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

    fn explist1(&mut self) -> MaybeASTNode {
        None
    }

    fn name(&mut self) -> MaybeASTNode {
        let string = match self.current() {
            Token::STRING(s) => s,
            _ => String::new(),
        };
        if self.accept(Token::STRING(string.clone())) {
            return Some(ASTNode::Name(string));
        }
        None
    }

    fn table_constructor(&mut self) -> MaybeASTNode {
        None
    }

    fn args(&mut self) -> Option<ASTNode> {
        if self.accept(Token::LEFT_PAREN) {
            // doesn't really matter if this returns anything.
            let exp_list = self.explist1();
            self.expect(Token::RIGHT_PAREN);
            return Some(ASTNode::ArgsParamList(match exp_list {
                Some(t) => Some(Box::new(t)),
                None => None,
            }));
        }

        None
    }

    fn functioncall(&mut self) -> Option<ASTNode> {
        if let Some(prefix_exp) = self.prefixexp() {
            let args = match self.args() {
                Some(args) => args,
                None => {
                    self.report_expected_error("<args>");
                    return None;
                }
            };
            return Some(ASTNode::FunctionCall(Box::new(
                ASTNode::PrefixExpressionArgs {
                    prefix_expression: Box::new(prefix_exp),
                    arguments: Box::new(args),
                },
            )));
        }

        if let Some(prefix_exp) = self.prefixexp() {
            self.expect(Token::COLON);

            let name = match self.name() {
                Some(args) => args,
                None => {
                    self.report_expected_error("<name>");
                    return None;
                }
            };

            let args = match self.args() {
                Some(args) => args,
                None => {
                    self.report_expected_error("<args>");
                    return None;
                }
            };

            return Some(ASTNode::FunctionCall(Box::new(
                ASTNode::PrefixExpressionNameArgs {
                    prefix_expression: Box::new(prefix_exp),
                    name: Box::new(name),
                    arguments: Box::new(args),
                },
            )));
        }

        None
    }

    fn var(&mut self) -> Option<ASTNode> {
        if let Some(tree) = self.name() {
            return Some(ASTNode::Variable(Box::new(tree)));
        }
        None
    }

    fn prefixexp(&mut self) -> Option<ASTNode> {
        if let Some(tree) = self.var() {
            return Some(ASTNode::PrefixExpression(Box::new(tree)));
        }

        if let Some(tree) = self.var() {
            return Some(ASTNode::PrefixExpression(Box::new(tree)));
        }

        None
    }

    fn function(&mut self) -> Option<ASTNode> {
        None
    }

    // parse an expression.
    fn exp(&mut self) -> Option<ASTNode> {
        let found_terminal = match self.current() {
            Token::NIL | Token::FALSE | Token::TRUE | Token::DOTS => true,
            Token::NUMBER(_) => true,
            Token::STRING(_) => true,
            _ => false,
        };

        if found_terminal {
            self.advance();
            return Some(ASTNode::Token(self.current()));
        }

        if let Some(tree) = self.function() {
            return Some(tree);
        }

        None
    }

    fn stat(&mut self) -> MaybeASTNode {
        if self.accept(Token::DO) {
            let block = match self.block() {
                Some(block) => block,
                None => {
                    self.report_expected_error("<block>");
                    return None;
                }
            };

            self.expect(Token::END);

            return Some(ASTNode::Statement(Box::new(ASTNode::Do(Box::new(block)))));
        }

        None
    }

    fn laststat(&mut self) -> MaybeASTNode {
        if self.accept(Token::RETURN) {
            let expression_list = self.explist1();
            return Some(ASTNode::LastStatement(match expression_list {
                Some(t) => Box::new(t),
                None => Box::new(ASTNode::Token(Token::RETURN)),
            }));
        }

        if self.accept(Token::BREAK) {
            return Some(ASTNode::LastStatement(Box::new(ASTNode::Token(
                Token::BREAK,
            ))));
        }

        None
    }

    fn block(&mut self) -> MaybeASTNode {
        self.chunk()
    }

    fn chunk(&mut self) -> MaybeASTNode {
        let mut statements = Vec::new();

        while let Some(tree) = self.stat() {
            // optional, no need to do anything.
            self.accept(Token::SEMICOLON);
            statements.push(tree);
        }

        let last_statement = self.laststat();

        let chunk = ASTNode::Chunk(
            statements.clone(),
            match last_statement.clone() {
                Some(t) => Some(Box::new(t)),
                None => None,
            },
        );

        // if statements.is_empty() && last_statement.is_none() {
        //     None
        // } else {
        Some(chunk)
        // }
    }

    pub fn parse(&mut self) -> MaybeASTNode {
        let chunk = self.chunk();

        if self.errored {
            None
        } else {
            chunk
        }
    }
}
