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
        elseif: Vec<(ASTNode, ASTNode)>,
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
    FunctionStatement {
        func_name: Box<ASTNode>,
        function_body: Box<ASTNode>,
    },
    LocalFunction {
        // NOTE: this is not a funcname, rather a NAME.
        name: Box<ASTNode>,
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
    PrefixExpressionBracketsExpression {
        prefix_expression: Box<ASTNode>,
        expression: Box<ASTNode>,
    },
    PrefixExpressionDotName {
        prefix_expression: Box<ASTNode>,
        name: Box<ASTNode>,
    },
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
    // This is just one of the possible productions.
    ParameterListA {
        name_list: Box<ASTNode>,
        variadic: bool,
    },
    ParameterListB(Box<ASTNode>),
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
    Args(Box<ASTNode>),
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

    fn varlist(&mut self) -> MaybeASTNode {
        if let Some(var) = self.var() {
            let mut var_list = Vec::new();

            while self.accept(Token::COMMA) {
                let var = self.var().or_else(|| {
                    self.report_expected_error("<var>");
                    return None;
                })?;
                var_list.push(var);
            }

            return Some(ASTNode::VariableList {
                variable: Box::new(var),
                tail_list: var_list,
            });
        }

        None
    }

    fn funcname(&mut self) -> MaybeASTNode {
        if let Some(name) = self.name() {
            let mut name_list = Vec::new();

            while self.accept(Token::DOT) {
                let name = self.name().or_else(|| {
                    self.report_expected_error("<name>");
                    return None;
                })?;
                name_list.push(name);
            }

            let col_name = if self.accept(Token::COLON) {
                Some(self.name().or_else(|| {
                    self.report_expected_error("<name>");
                    return None;
                })?)
            } else {
                None
            };

            return Some(ASTNode::FunctionName {
                name: Box::new(name),
                members: name_list,
                colon: col_name.map(Box::new),
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
            return Some(ASTNode::Args(Box::new(ASTNode::ArgsParamList(
                match exp_list {
                    Some(t) => Some(Box::new(t)),
                    None => None,
                },
            ))));
        }

        if let Some(table_constructor) = self.tableconstructor() {
            return Some(ASTNode::Args(Box::new(table_constructor)));
        }

        if let Token::STRING(s) = self.current() {
            self.advance();
            return Some(ASTNode::Args(Box::new(ASTNode::Token(Token::STRING(s)))));
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

        if let Some(tree) = self.prefixexp() {
            if self.accept(Token::LEFT_BRACKET) {
                let exp = self.exp().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                self.expect(Token::RIGHT_BRACKET);

                return Some(ASTNode::Variable(Box::new(
                    ASTNode::PrefixExpressionBracketsExpression {
                        prefix_expression: Box::new(tree),
                        expression: Box::new(exp),
                    },
                )));
            }

            if self.accept(Token::DOT) {
                let name = self.name().or_else(|| {
                    self.report_expected_error("<name>");
                    return None;
                })?;

                return Some(ASTNode::Variable(Box::new(
                    ASTNode::PrefixExpressionDotName {
                        prefix_expression: Box::new(tree),
                        name: Box::new(name),
                    },
                )));
            }
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

    fn parlist1(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.namelist() {
            let variadic = if self.accept(Token::COMMA) {
                self.expect(Token::DOTS);
                true
            } else {
                false
            };

            return Some(ASTNode::ParameterListA {
                name_list: Box::new(tree),
                variadic,
            });
        }

        if self.accept(Token::DOTS) {
            return Some(ASTNode::ParameterListB(Box::new(ASTNode::Token(
                Token::DOTS,
            ))));
        }

        None
    }

    fn funcbody(&mut self) -> MaybeASTNode {
        if self.accept(Token::LEFT_PAREN) {
            let parameter_list = self.parlist1();
            self.expect(Token::RIGHT_PAREN);

            let block = self.block().or_else(|| {
                self.report_expected_error("<block>");
                return None;
            })?;

            return Some(ASTNode::FunctionBody {
                parameter_list: parameter_list.map(Box::new),
                block: Box::new(block),
            });
        }
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

    fn exp_or(&mut self) -> MaybeASTNode {
        if let Some(tree) = self.exp_and() {
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
            Token::NAME(_) => true,
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
        if let Some(tree) = self.exp_or() {
            return Some(ASTNode::Expression(Box::new(tree)));
        }

        if let Some(tree) = self.function() {
            return Some(ASTNode::Expression(Box::new(tree)));
        }

        if let Some(tree) = self.tableconstructor() {
            return Some(ASTNode::Expression(Box::new(tree)));
        }

        if let Some(tree) = self.prefixexp() {
            return Some(ASTNode::Expression(Box::new(tree)));
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

        if self.accept(Token::WHILE) {
            let exp = self.exp().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;
            self.expect(Token::DO);
            let block = match self.block() {
                Some(block) => block,
                None => {
                    self.report_expected_error("<block>");
                    return None;
                }
            };

            self.expect(Token::END);

            return Some(ASTNode::Statement(Box::new(ASTNode::While {
                expression: Box::new(exp),
                do_block: Box::new(block),
            })));
        }

        if self.accept(Token::REPEAT) {
            let block = self.block().or_else(|| {
                self.report_expected_error("<block>");
                return None;
            })?;

            self.expect(Token::UNTIL);

            let exp = self.exp().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;

            self.expect(Token::END);

            return Some(ASTNode::Statement(Box::new(ASTNode::Repeat {
                block: Box::new(block),
                expression: Box::new(exp),
            })));
        }

        if self.accept(Token::IF) {
            let exp = self.exp().or_else(|| {
                self.report_expected_error("<exp>");
                return None;
            })?;

            self.expect(Token::THEN);

            let block = self.block().or_else(|| {
                self.report_expected_error("<block>");
                return None;
            })?;

            let mut else_ifs = Vec::new();

            while self.accept(Token::ELSEIF) {
                let exp = self.exp().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                self.expect(Token::THEN);

                let block = self.block().or_else(|| {
                    self.report_expected_error("<block>");
                    return None;
                })?;

                else_ifs.push((exp, block));
            }

            let else_block = if self.accept(Token::ELSE) {
                Some(self.block().or_else(|| {
                    self.report_expected_error("<block>");
                    return None;
                })?)
            } else {
                None
            };

            self.expect(Token::END);

            return Some(ASTNode::Statement(Box::new(ASTNode::If {
                expression: Box::new(exp),
                block: Box::new(block),
                elseif: else_ifs,
                then_else: else_block.map(Box::new),
            })));
        }

        if self.accept(Token::FOR) {
            // numeric for.
            if let Some(name) = self.name() {
                self.expect(Token::ASSIGN);
                let exp = self.exp().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;
                self.expect(Token::COMMA);
                let exp2 = self.exp().or_else(|| {
                    self.report_expected_error("<exp>");
                    return None;
                })?;

                let exp3 = if self.accept(Token::COMMA) {
                    Some(self.exp().or_else(|| {
                        self.report_expected_error("<exp>");
                        return None;
                    })?)
                } else {
                    None
                };

                self.expect(Token::DO);

                let block = self.block().or_else(|| {
                    self.report_expected_error("<block>");
                    return None;
                })?;

                self.expect(Token::END);

                return Some(ASTNode::Statement(Box::new(ASTNode::ForNumeric {
                    name: Box::new(name),
                    from_expression: Box::new(exp),
                    to_expression: Box::new(exp2),
                    step_expression: exp3.map(Box::new),
                    do_block: Box::new(block),
                })));
            }

            // generic for.
            if let Some(name_list) = self.namelist() {
                self.expect(Token::IN);

                let exp_list = self.explist1().or_else(|| {
                    self.report_expected_error("<explist1>");
                    return None;
                })?;

                self.expect(Token::DO);

                let block = self.block().or_else(|| {
                    self.report_expected_error("<block>");
                    return None;
                })?;

                self.expect(Token::END);

                // return Some(ASTNode::Statement(Box::new()));
                return Some(ASTNode::Statement(Box::new(ASTNode::ForGeneric {
                    name_list: Box::new(name_list),
                    expression_list_1: Box::new(exp_list),
                    do_block: Box::new(block),
                })));
            }
        }

        if self.accept(Token::FUNCTION) {
            let func_name = self.funcname().or_else(|| {
                self.report_expected_error("<funcname>");
                return None;
            })?;

            let func_body = self.funcbody().or_else(|| {
                self.report_expected_error("<funcbody>");
                return None;
            })?;

            return Some(ASTNode::Statement(Box::new(ASTNode::FunctionStatement {
                func_name: Box::new(func_name),
                function_body: Box::new(func_body),
            })));
        }

        if self.accept(Token::LOCAL) {
            if self.accept(Token::FUNCTION) {
                let name = self.name().or_else(|| {
                    self.report_expected_error("<name>");
                    return None;
                })?;
                let func_body = self.funcbody().or_else(|| {
                    self.report_expected_error("<funcbody>");
                    return None;
                })?;

                return Some(ASTNode::Statement(Box::new(ASTNode::LocalFunction {
                    name: Box::new(name),
                    function_body: Box::new(func_body),
                })));
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

            if let Some(function_call) = self.functioncall() {
                return Some(ASTNode::Statement(Box::new(function_call)));
            }

            // varlist1 `=´ explist1.
            if let Some(var_list) = self.varlist() {
                self.expect(Token::ASSIGN);

                let exp_list = self.explist1().or_else(|| {
                    self.report_expected_error("<explist1>");
                    return None;
                })?;

                return Some(ASTNode::Statement(Box::new(ASTNode::LValueAssign {
                    var_list: Box::new(var_list),
                    expression_list: Box::new(exp_list),
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

        println!("{:#?}", chunk);

        if self.errored {
            None
        } else {
            chunk
        }
    }
}
