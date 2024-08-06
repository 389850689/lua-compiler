use std::thread::current;

use crate::lexer::Token;
use crate::{log_error, term_color::*};

#[derive(Clone)]
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
    // Function {
    //     function_name: Box<ASTNode>,
    //     function_body: Box<ASTNode>,
    // },
    Function {
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
    TableConstructor(Option<Box<ASTNode>>),
    FieldList {
        field: Box<ASTNode>,
        separated_fields: Vec<(ASTNode, ASTNode)>,
        separator: Option<Box<ASTNode>>,
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
    // BinaryOperator(Box<ASTNode>),
    // UnaryOperator(Box<ASTNode>),
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

    fn peek_expression(&mut self, n: Option<usize>) -> MaybeASTNode {
        let mut fork = self.clone();
        for _ in 0..n.unwrap_or(0) {
            fork.advance();
        }
        fork.exp()
    }

    // fn peek_binop(&mut self, n: Option<usize>) -> MaybeASTNode {
    //     let mut fork = self.clone();
    //     for _ in 0..n.unwrap_or(0) {
    //         fork.advance();
    //     }
    //     fork.binop()
    // }

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

    fn backtrack(&mut self) {
        self.cursor -= 1;
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
        if !self.accept(token.clone()) {
            log_error!(
                "[auto: {}] expected symbol: {:?}, found {:?}.",
                colored("parser", Color::Grey),
                token,
                self.current(),
            );
            self.errored = true;
        }
    }

    fn explist1(&mut self) -> MaybeASTNode {
        let mut exp_list = Vec::new();

        while let Some(tree) = self.exp() {
            if self.accept(Token::COMMA) {
                exp_list.push(tree);
            } else {
                return Some(ASTNode::ExpressionList {
                    head_list: exp_list,
                    expression: Box::new(tree),
                });
            }
        }

        None
    }

    fn name(&mut self) -> MaybeASTNode {
        println!("{}", std::backtrace::Backtrace::force_capture());
        if let Token::NAME(s) = self.current() {
            self.advance();
            return Some(ASTNode::Name(s));
        }
        None
    }

    fn namelist(&mut self) -> MaybeASTNode {
        if let Some(name) = self.name() {
            let mut name_list = Vec::new();

            while self.accept(Token::COMMA) {
                let name = self.name().or_else(|| {
                    self.report_expected_error("<name>");
                    return None;
                })?;
                name_list.push(name);
            }

            return Some(ASTNode::NameList {
                name: Box::new(name),
                tail_list: name_list,
            });
        }

        None
    }

    fn fieldsep(&mut self) -> MaybeASTNode {
        if self.accept(Token::COMMA) {
            return Some(ASTNode::Fieldsep(Box::new(ASTNode::Token(Token::COMMA))));
        }

        if self.accept(Token::SEMICOLON) {
            return Some(ASTNode::Fieldsep(Box::new(ASTNode::Token(
                Token::SEMICOLON,
            ))));
        }

        None
    }

    fn field(&mut self) -> MaybeASTNode {
        if self.accept(Token::LEFT_BRACKET) {
            let exp1 = self.exp().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;

            self.expect(Token::RIGHT_BRACKET);
            self.expect(Token::ASSIGN);

            let exp2 = self.exp().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;

            return Some(ASTNode::Field(Box::new(ASTNode::FieldA {
                expression_a: Box::new(exp1),
                expression_b: Box::new(exp2),
            })));
        }

        if let Some(name) = self.name() {
            self.expect(Token::ASSIGN);

            let exp = self.exp().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;

            return Some(ASTNode::Field(Box::new(ASTNode::FieldB {
                name: Box::new(name),
                expression: Box::new(exp),
            })));
        }

        if let Some(exp) = self.exp() {
            return Some(ASTNode::Field(Box::new(exp)));
        }

        None
    }

    fn fieldlist(&mut self) -> MaybeASTNode {
        if let Some(field) = self.field() {
            let mut fieldseps = Vec::new();

            while let Some(fieldsep) = self.fieldsep() {
                let field = self.field().or_else(|| {
                    self.report_expected_error("<field>");
                    return None;
                })?;

                fieldseps.push((fieldsep, field))
            }

            let fieldsep = self.fieldsep();

            return Some(ASTNode::FieldList {
                field: Box::new(field),
                separated_fields: fieldseps,
                separator: fieldsep.map(Box::new),
            });
        }
        None
    }

    fn tableconstructor(&mut self) -> MaybeASTNode {
        if self.accept(Token::LEFT_BRACE) {
            let field_list = self.fieldlist();
            self.expect(Token::RIGHT_BRACE);
            return Some(ASTNode::TableConstructor(field_list.map(Box::new)));
        }

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
            let args = self.args().or_else(|| {
                self.report_expected_error("<args>");
                return None;
            })?;

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

        if let Some(tree) = self.functioncall() {
            return Some(ASTNode::PrefixExpression(Box::new(tree)));
        }

        if self.accept(Token::LEFT_PAREN) {
            let exp = match self.exp() {
                Some(exp) => exp,
                None => {
                    self.report_expected_error("<exp>");
                    return None;
                }
            };
            self.expect(Token::RIGHT_PAREN);
            return Some(ASTNode::PrefixExpression(Box::new(exp)));
        }

        None
    }

    fn funcbody(&mut self) -> MaybeASTNode {
        None
    }

    fn function(&mut self) -> Option<ASTNode> {
        if self.accept(Token::FUNCTION) {
            let funcbody = self.funcbody().or_else(|| {
                self.report_expected_error("<funcbody>");
                return None;
            })?;

            return Some(ASTNode::Function {
                function_body: Box::new(funcbody),
            });
        }
        None
    }

    // fn binop(&mut self) -> MaybeASTNode {
    //     // we now know it's a binary operator.
    //     if self.is_currently_binop() {
    //         let current_terminal = self.current();
    //
    //         // go back a token to get the first argument.
    //         self.backtrack();
    //
    //         let exp1 = self.exp().or_else(|| {
    //             self.report_expected_error("<exp>");
    //             return None;
    //         })?;
    //
    //         // advance twice to get to the right hand side operand.
    //         self.advance();
    //         self.advance();
    //
    //         let exp2 = self.exp().or_else(|| {
    //             self.report_expected_error("<exp>");
    //             return None;
    //         })?;
    //
    //         return Some(ASTNode::BinaryOp {
    //             left: Box::new(exp1),
    //             binary_operator: Box::new(ASTNode::BinaryOperator(Box::new(ASTNode::Token(
    //                 current_terminal,
    //             )))),
    //             right: Box::new(exp2),
    //         });
    //     }
    //     None
    // }
    //
    // fn unop(&mut self) -> MaybeASTNode {
    //     let found_terminal = match self.current() {
    //         Token::SUBTRACT | Token::NOT | Token::HASHTAG => true,
    //         _ => false,
    //     };
    //
    //     // we now know it's a unary operator.
    //     if found_terminal {
    //         let current_terminal = self.current();
    //
    //         self.advance();
    //
    //         let exp = self.exp().or_else(|| {
    //             self.report_expected_error("<exp>");
    //             return None;
    //         })?;
    //
    //         return Some(ASTNode::UnaryOp {
    //             unary_operator: Box::new(ASTNode::UnaryOperator(Box::new(ASTNode::Token(
    //                 current_terminal,
    //             )))),
    //             right: Box::new(exp),
    //         });
    //     }
    //     None
    // }

    fn exp_or(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_and() {
            println!("test");
            if self.accept(Token::OR) {
                let exp = self.exp_and().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                return Some(ASTNode::Expression(Box::new(ASTNode::BinaryOp {
                    left: Box::new(tree),
                    binary_operator: Box::new(ASTNode::Token(Token::OR)),
                    right: Box::new(exp),
                })));
            } else {
                return Some(tree);
            }
        }
        None
    }

    fn exp_and(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_eqaulity() {
            println!("test");
            if self.accept(Token::AND) {
                let exp = self.exp_eqaulity().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                return Some(ASTNode::Expression(Box::new(ASTNode::BinaryOp {
                    left: Box::new(tree),
                    binary_operator: Box::new(ASTNode::Token(Token::AND)),
                    right: Box::new(exp),
                })));
            } else {
                return Some(tree);
            }
        }
        None
    }

    fn exp_eqaulity(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_concat() {
            let current_token = self.current();
            if self.accept(Token::GREATER_THAN)
                || self.accept(Token::LESS_THAN)
                || self.accept(Token::LESS_EQUAL)
                || self.accept(Token::GREATER_EQUAL)
                || self.accept(Token::NEQ)
                || self.accept(Token::EQ)
            {
                let exp = self.exp_concat().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                return Some(ASTNode::Expression(Box::new(ASTNode::BinaryOp {
                    left: Box::new(tree),
                    binary_operator: Box::new(ASTNode::Token(current_token)),
                    right: Box::new(exp),
                })));
            } else {
                return Some(tree);
            }
        }
        None
    }

    // NOTE: make this right associative in a second.
    fn exp_concat(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_term() {
            if self.accept(Token::CONCAT) {
                let exp = self.exp_term().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                return Some(ASTNode::Expression(Box::new(ASTNode::BinaryOp {
                    left: Box::new(tree),
                    binary_operator: Box::new(ASTNode::Token(Token::CONCAT)),
                    right: Box::new(exp),
                })));
            } else {
                return Some(tree);
            }
        }
        None
    }

    fn exp_term(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_factor() {
            let current_token = self.current();
            if self.accept(Token::ADD) || self.accept(Token::SUBTRACT) {
                let exp = self.exp_factor().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                return Some(ASTNode::Expression(Box::new(ASTNode::BinaryOp {
                    left: Box::new(tree),
                    binary_operator: Box::new(ASTNode::Token(current_token)),
                    right: Box::new(exp),
                })));
            } else {
                return Some(tree);
            }
        }
        None
    }

    fn exp_factor(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_unary() {
            let current_token = self.current();
            if self.accept(Token::MULTIPLY)
                || self.accept(Token::DIVIDE)
                || self.accept(Token::MODULO)
            {
                let exp = self.exp_unary().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                return Some(ASTNode::Expression(Box::new(ASTNode::BinaryOp {
                    left: Box::new(tree),
                    binary_operator: Box::new(ASTNode::Token(current_token)),
                    right: Box::new(exp),
                })));
            } else {
                return Some(tree);
            }
        }
        None
    }

    fn exp_unary(&mut self) -> MaybeASTNode {
        let current_token = self.current();
        if self.accept(Token::NOT) || self.accept(Token::HASHTAG) || self.accept(Token::SUBTRACT) {
            let exp = self.exp_exponent().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;

            return Some(ASTNode::Expression(Box::new(ASTNode::UnaryOp {
                unary_operator: Box::new(ASTNode::Token(current_token)),
                right: Box::new(exp),
            })));
        }

        if let Some(tree) = self.exp_exponent() {
            return Some(tree);
        }

        None
    }

    fn exp_exponent(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_primary() {
            if self.accept(Token::XOR) {
                let exp = self.exp_primary().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                return Some(ASTNode::Expression(Box::new(ASTNode::BinaryOp {
                    left: Box::new(tree),
                    binary_operator: Box::new(ASTNode::Token(Token::XOR)),
                    right: Box::new(exp),
                })));
            } else {
                return Some(tree);
            }
        }
        None
    }

    fn exp_primary(&mut self) -> MaybeASTNode {
        let found_terminal = match self.current() {
            Token::NUMBER(_) => true,
            Token::STRING(_) => true,
            Token::NIL | Token::FALSE | Token::TRUE | Token::DOTS => true,
            _ => false,
        };

        if found_terminal {
            let current_token = self.current();
            self.advance();
            return Some(ASTNode::Expression(Box::new(ASTNode::Token(current_token))));
        }

        if self.accept(Token::LEFT_PAREN) {
            let exp = self.exp_or().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;
            self.expect(Token::RIGHT_PAREN);
            return Some(ASTNode::Expression(Box::new(exp)));
        }

        None
    }

    // parse an expression.
    fn exp(&mut self) -> Option<ASTNode> {
        // let found_terminal = match self.current() {
        //     Token::NUMBER(_) => true,
        //     Token::STRING(_) => true,
        //     Token::NIL | Token::FALSE | Token::TRUE | Token::DOTS => true,
        //     _ => false,
        // };
        //
        // if found_terminal {
        //     let current_token = self.current();
        //     self.advance();
        //     return Some(ASTNode::Expression(Box::new(ASTNode::Token(current_token))));
        // }

        if let Some(tree) = self.function() {
            return Some(ASTNode::Expression(Box::new(tree)));
        }

        if let Some(tree) = self.prefixexp() {
            return Some(ASTNode::Expression(Box::new(tree)));
        }

        if let Some(tree) = self.tableconstructor() {
            return Some(ASTNode::Expression(Box::new(tree)));
        }

        // if let Some(tree) = self.exp_or() {
        //     return Some(ASTNode::Expression(Box::new(tree)));
        // }

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

        if self.accept(Token::FUNCTION) {
            let exp = self.exp_or().or_else(|| {
                self.report_expected_error("DEBUG <exp>");
                return None;
            })?;

            return Some(ASTNode::Expression(Box::new(exp)));
        }

        if self.accept(Token::LOCAL) {
            if self.accept(Token::FUNCTION) {
                return None;
            }

            if let Some(name_list) = self.namelist() {
                let exp_list = if self.accept(Token::ASSIGN) {
                    self.explist1()
                } else {
                    None
                };
                return Some(ASTNode::Statement(Box::new(ASTNode::LocalVariable {
                    name_list: Box::new(name_list),
                    expression_list: exp_list.map(Box::new),
                })));
            }
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
        if let Some(tree) = self.chunk() {
            return Some(ASTNode::Block(Box::new(tree)));
        }
        None
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
