use std::iter::Iterator;

use crate::{
    ast::{AstNode, ExprKind},
    scan::{Scanner, Token, TokenKind, fatal, fatal_tok},
    sym::Strings,
    types::*,
};

fn binding_power(token: &Token, unary: bool) -> u8 {
    match token.kind {
        TokenKind::Assign => 10,
        TokenKind::And | TokenKind::Or => 20,
        TokenKind::Lss | TokenKind::Leq
        | TokenKind::Gtr | TokenKind::Geq
        | TokenKind::Eql | TokenKind::Neq => 30,
        TokenKind::Plus => 40,
        TokenKind::Minus if !unary => 40,
        TokenKind::Star | TokenKind::Slash | TokenKind::Mod => 50,
        TokenKind::Caret => 60,
        TokenKind::Minus if unary => 70,
        TokenKind::Not => 70,
        TokenKind::Incr | TokenKind::Decr => 80,
        TokenKind::LeftBrk => 90,
        TokenKind::LeftParen => 90,
        _ => 0,
    }
}


pub struct Parser<I>
    where I: Iterator<Item=char>
{
    scanner: Scanner<I>,
}

impl<I> Parser<I>
    where I: Iterator<Item=char>
{
    pub fn new(scanner: Scanner<I>) -> Self {
        Parser { scanner }
    }

    pub fn into_strings(self) -> Strings {
        self.scanner.into_strings()
    }

    fn parse_if(&mut self) -> AstNode {
        self.scanner.must_be_kind(TokenKind::If);
        self.scanner.must_be_kind(TokenKind::LeftParen);
        let cond = self.parse_expression(0);
        self.scanner.must_be_kind(TokenKind::RightParen);
        let true_body = self.parse_block();
        let false_body = if let Some(Token { kind: TokenKind::Else, .. }) = self.scanner.peek() {
            self.scanner.discard_token();
            self.parse_block()
        } else {
            AstNode::EmptyBlock
        };

        AstNode::make_if(cond, true_body, false_body)
    }

    fn parse_assign(&mut self) -> AstNode {
        // TODO: This needs help to generate error messages. We've lost the token
        let expr = self.parse_expression(0);
        match expr {
            AstNode::Expr { kind: ExprKind::Assignment {..}, .. } => expr,
            x => panic!("Expected an identifier, got {x:?}")
        }
    }

    fn parse_for(&mut self) -> AstNode {
        let mut init = vec![];
        let mut post = vec![];

        self.scanner.must_be_kind(TokenKind::For);
        self.scanner.must_be_kind(TokenKind::LeftParen);
        if !self.scanner.maybe_kind(TokenKind::Semi, true) { // Consume if matching
            loop {
                init.push(self.parse_assign());
                match self.scanner.scan() {
                    Some(Token { kind: TokenKind::Semi, .. }) => break,
                    Some(Token { kind: TokenKind::Comma, .. }) => {},
                    Some(t) => fatal_tok("Expected ',' or ';'", t),
                    None => panic!("EOF found when expecting ',' or ';'"),
                }
            }
        }
        let cond = self.parse_expression(0);
        self.scanner.must_be_kind(TokenKind::Semi);
        if !self.scanner.maybe_kind(TokenKind::RightParen, true) { // Consume if matching
            loop {
                post.push(self.parse_expression(0));
                match self.scanner.scan() {
                    Some(Token { kind: TokenKind::RightParen, .. }) => break,
                    Some(Token { kind: TokenKind::Comma, .. }) => {},
                    Some(t) => fatal_tok("Expected ',' or ')'", t),
                    None => panic!("EOF found when expecting ',' or ')'"),
                }
            }
        }
        let body = self.parse_block();

        AstNode::make_for(init, cond, post, body)
    }

    fn parse_print(&mut self) -> AstNode {
        let mut exprs = vec![];

        self.scanner.must_be_kind(TokenKind::Print);
        if !self.scanner.maybe_kind(TokenKind::Semi, true) {
            loop {
                exprs.push(self.parse_expression(0));
                match self.scanner.scan() {
                    Some(Token { kind: TokenKind::Semi, .. }) => break,
                    Some(Token { kind: TokenKind::Comma, .. }) => {},
                    Some(t) => fatal_tok("Expected ',' or ';'", t),
                    None => panic!("EOF found while expecting ',' or ';'"),
                }
            }
        }

        AstNode::Print(exprs)
    }

    fn parse_return(&mut self) -> AstNode {
        self.scanner.must_be_kind(TokenKind::Return);
        if self.scanner.maybe_kind(TokenKind::Semi, true) {
            AstNode::VoidReturn
        } else {
            let ret = AstNode::make_return(self.parse_expression(0));
            self.scanner.must_be_kind(TokenKind::Semi);
            ret
        }
    }

    fn parse_block(&mut self) -> AstNode {
        let mut statements = vec![];

        self.scanner.must_be_kind(TokenKind::LeftAngl);

        loop {
            let peek = self.scanner.peek();
            if matches!(peek, Some(Token { kind: TokenKind::RightAngl, .. })) {
                self.scanner.discard_token();
                break;
            }
            let statement = match peek {
                Some(Token { kind: TokenKind::LeftAngl, .. }) => {
                    self.parse_block()
                },
                Some(Token { kind: TokenKind::Ident(id), .. }) => {
                    let leader = self.scanner.scan().unwrap();
                    let next_tk = self.scanner.scan();
                    match next_tk {
                        // Some(Token { kind: TokenKind::Assign, .. }) => {
                        //     let expr = self.parse_expression(0);
                        //     self.scanner.must_be_kind(TokenKind::Semi);
                        //     AstNode::make_assignment(id, expr)
                        // },
                        Some(Token { kind: TokenKind::Colon, .. }) => {
                            let decl = self.parse_var_decl_with_id(id);
                            self.scanner.must_be_kind(TokenKind::Semi);
                            decl
                        },
                        Some(Token { kind: TokenKind::LeftParen, .. }) => {
                            let fcall = self.parse_func_arguments(id);
                            self.scanner.must_be_kind(TokenKind::Semi);
                            fcall
                        },
                        Some(t) => {
                            // Last option, it might be an expression!
                            self.scanner.put_token(t);
                            self.scanner.put_token(leader);
                            let expr = self.parse_expression(0);
                            self.scanner.must_be_kind(TokenKind::Semi);
                            expr
                        }
//                        Some(t) => fatal_tok("Expected '=', ':', '('", t),
                        None => panic!("Found EOF when expecting '=', ':', or '('"),
                    }
                },
                Some(Token { kind: TokenKind::If, .. }) => self.parse_if(),
                Some(Token { kind: TokenKind::For, .. }) => self.parse_for(),
                Some(Token { kind: TokenKind::Print, .. }) => self.parse_print(),
                Some(Token { kind: TokenKind::Return, .. }) => self.parse_return(),
                Some(_) => self.parse_expression(0),
                None => panic!("Found EOF expecting a statement"),
            };
            statements.push(statement);
        }

        if statements.is_empty() {
            AstNode::EmptyBlock
        } else {
            AstNode::Block(statements)
        }
    }

    fn parse_return_type(&mut self) -> Type {
        match self.scanner.scan() {
            Some(Token { kind: TokenKind::Integer, .. }) => Type::Scalar(PrimType::Int),
            Some(Token { kind: TokenKind::Char, .. }) => Type::Scalar(PrimType::Char),
            Some(Token { kind: TokenKind::Bool, .. }) => Type::Scalar(PrimType::Bool),
            Some(Token { kind: TokenKind::String, .. }) => Type::Scalar(PrimType::String),
            Some(Token { kind: TokenKind::Void, .. }) => Type::Scalar(PrimType::Void),
            Some(t) => fatal_tok("Expected a return type declaration", t),
            None => panic!("Found EOF while expecting a type declaration")
        }
    }

    fn parse_function_signature(&mut self) -> Type {
        let ret_type = self.parse_return_type();
        let mut params = vec![];
        self.scanner.must_be_kind(TokenKind::LeftParen);
        if let Some(Token { kind: TokenKind::RightParen, .. }) = self.scanner.peek() {
            self.scanner.discard_token();
        } else {
            loop {
                let AstNode::VarDecl { name, dtype, init, .. } = self.parse_var_decl()
                    else { unreachable!() };

                // TODO: We need span info to have a proper error message here
                if init.is_some() { panic!("No initialization allowed in function signatures") }

                params.push(Param { name, dtype });
                match self.scanner.scan() {
                    Some(Token { kind: TokenKind::Comma, .. }) => {},
                    Some(Token { kind: TokenKind::RightParen, .. }) => break,
                    Some(t) => fatal_tok("Expected ',' or ')'", t),
                    None => panic!("EOF found parsing function call arguments"),
                }
            }
        }

        Type::make_signature(ret_type, params)
    }

    fn parse_var_decl(&mut self) -> AstNode {
        let id = match self.scanner.scan() {
            Some(Token { kind: TokenKind::Ident(id), .. }) => id,
            Some(t) => fatal_tok("Expected a variable declaration", t),
            None => panic!("EOF found while expecting a declaration"),
        };

        self.scanner.must_be_kind(TokenKind::Colon);
        self.parse_var_decl_with_id(id)
    }

    fn parse_array_initializer(&mut self) -> AstNode {
        let mut args = vec![];
        loop {
            args.push(if self.scanner.maybe_kind(TokenKind::LeftBrk, true) {
                self.parse_array_initializer()
            } else {
                self.parse_expression(0)
            });
            match self.scanner.scan() {
                Some(Token { kind: TokenKind::Comma, .. }) => {},
                Some(Token { kind: TokenKind::RightBrk, .. }) => break,
                Some(t) => fatal_tok("Expected ',' or ']'", t),
                None => panic!("EOF found parsing array initialization"),
            }
        }
        AstNode::ArrayInitializer(args)
    }

    fn parse_var_decl_with_id(&mut self, ident: usize) -> AstNode {
        let dtype = self.parse_var_decl_type();
        let init = if let Some(Token { kind: TokenKind::Assign, .. }) = self.scanner.peek() {
            self.scanner.discard_token();
            if self.scanner.maybe_kind(TokenKind::LeftBrk, true) {
                Some(self.parse_array_initializer())
            } else if self.scanner.peek().is_none() {
                panic!("Found EOF parsing an initializer")
            } else {
                Some(self.parse_expression(0))
            }
        } else {
            None
        };
        AstNode::make_var_decl(ident, dtype, init)
    }

    fn parse_var_decl_type(&mut self) -> Type {
        match self.scanner.scan() {
            Some(Token { kind: TokenKind::Array, .. }) => {
                self.scanner.must_be_kind(TokenKind::LeftBrk);
                let size = match self.scanner.scan() {
                    Some(Token { kind: TokenKind::IntLit(val), .. }) => val as usize,
                    Some(t) => fatal_tok("Expected array size", t),
                    None => panic!("Found EOF while expecting array size"),
                };
                self.scanner.must_be_kind(TokenKind::RightBrk);

                let inner = self.parse_var_decl_type();
                Type::make_array(size, inner)
            },
            Some(Token { kind: TokenKind::Integer, .. }) => Type::Scalar(PrimType::Int),
            Some(Token { kind: TokenKind::Char, .. }) => Type::Scalar(PrimType::Char),
            Some(Token { kind: TokenKind::Bool, .. }) => Type::Scalar(PrimType::Bool),
            Some(Token { kind: TokenKind::String, .. }) => Type::Scalar(PrimType::String),
            Some(t) => fatal_tok("Expected a type declaration", t),
            None => panic!("Found EOF while expecting a type declaration")
        }
    }

    fn null_denotation(&mut self, tok: Token) -> AstNode {
        let Token { kind, .. } = tok;
        match kind {
            TokenKind::LeftParen => {
                let expr = self.parse_expression(0);
                self.scanner.must_be_kind(TokenKind::RightParen);
                expr
            },
            TokenKind::Not => AstNode::make_not(self.parse_expression(binding_power(&tok, true))),
            TokenKind::Minus => AstNode::make_minus(self.parse_expression(binding_power(&tok, true))),
            TokenKind::IntLit(_)
            | TokenKind::CharLit(_)
            | TokenKind::StringLit(_)
            | TokenKind::True
            | TokenKind::False => AstNode::make_literal(tok),
            TokenKind::Ident(id) => AstNode::make_expr(ExprKind::Ident(id)),
            _ => fatal_tok("Expected to find '(', '!', '-', a literal, or an identifier", tok),
        }
    }

    fn parse_func_arguments(&mut self, id: usize) -> AstNode {
        let mut args = vec![];
        if let Some(Token { kind: TokenKind::RightParen, .. }) = self.scanner.peek() {
            self.scanner.discard_token();
        } else {
            loop {
                args.push(self.parse_expression(0));
                match self.scanner.scan() {
                    Some(Token { kind: TokenKind::Comma, .. }) => {},
                    Some(Token { kind: TokenKind::RightParen, .. }) => break,
                    Some(t) => fatal_tok("Expected ',' or ')'", t),
                    None => panic!("EOF found parsing function call arguments"),
                }
            }
        }

        AstNode::make_func_call(id, args)
    }

    fn parse_expression(&mut self, min_bp: u8) -> AstNode {
        let mut left_node = match self.scanner.scan() {
            Some(t) => self.null_denotation(t),
            None => panic!("Found EOF expecting an expression"),
        };

        while let Some(next_tk) = self.scanner.peek() && binding_power(&next_tk, false) > min_bp {
            self.scanner.discard_token(); // Consume the next_tk

            // Pratt "left denotation"
            let result = match next_tk.kind {
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Mod
                | TokenKind::Caret
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Eql
                | TokenKind::Neq
                | TokenKind::Lss
                | TokenKind::Leq
                | TokenKind::Gtr
                | TokenKind::Geq => {
                    AstNode::make_binary(next_tk, left_node,
                        self.parse_expression(binding_power(&next_tk, false)))
                },

                TokenKind::Incr
                | TokenKind::Decr => {
                    match left_node {
                        AstNode::Expr {kind: ExprKind::Ident(name), ..} => AstNode::make_postfix(next_tk.kind, name),
                        _ => fatal_tok("Expected identifier before", next_tk)
                    }
                },

                TokenKind::LeftBrk => {
                    let index = self.parse_expression(0);
                    self.scanner.must_be_kind(TokenKind::RightBrk);
                    match left_node {
                        AstNode::Expr {kind: ExprKind::Ident(_), .. }
                        | AstNode::Expr { kind: ExprKind::Subscript {..}, .. } => AstNode::make_subscript(left_node, index),
                        _ => fatal("Expected identifier or ']' before subscript", next_tk.pos)
                    }
                },
                TokenKind::LeftParen => {
                    let AstNode::Expr {kind: ExprKind::Ident(name), ..} = left_node else { 
                        fatal("Expected identifier before '('", next_tk.pos)
                    };
                    self.parse_func_arguments(name)
                },
                TokenKind::Assign => {
                    let AstNode::Expr {kind: ExprKind::Ident(name), ..} = left_node else {
                        fatal("Expected identifier before '='", next_tk.pos)
                    };
                    AstNode::make_assignment(name, self.parse_expression(binding_power(&next_tk, false)))
                },

                _ => fatal_tok("Expected binary operator, (, or [", next_tk)
            };
            left_node = result;
        }

        left_node
    }

    fn declaration(&mut self) -> Option<AstNode> {
        let ident = match self.scanner.scan() {
            Some(Token { kind: TokenKind::Ident(id), .. }) => id,
            Some(Token { pos, .. }) => { fatal("Expected an identifier", pos) }
            None => return None,
        };

        self.scanner.must_be_kind(TokenKind::Colon);

        Some(match self.scanner.scan() {
            Some(Token { kind: TokenKind::Function, .. }) => {
                let signature = self.parse_function_signature();
                match self.scanner.scan() {
                    Some(Token { kind: TokenKind::Semi, .. }) => AstNode::make_forward(ident, signature ),
                    Some(Token { kind: TokenKind::Assign, .. }) => {
                        AstNode::make_function(ident, signature, self.parse_block())
                    },
                    Some(t) => fatal_tok("Expected ';' or '='", t),
                    None => panic!("Found EOF when parsing a function"),
                }
            },
            Some(t) if t.is_type() => {
                self.scanner.put_token(t);
                let decl = self.parse_var_decl_with_id(ident);
                self.scanner.must_be_kind(TokenKind::Semi);
                decl
            },
            Some(Token { pos, .. }) => fatal("Expected 'function' or a type", pos),
            None => panic!("Found EOF parsing a declaration"),
        })
    }

    pub fn parse_top(&mut self) -> Vec<AstNode> {
        std::iter::from_fn(|| self.declaration()).collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        ast::{BinaryOp, ExprKind},
        scan::{Scanner, Pos},
    };
    use std::string::IntoChars;
    use rstest::rstest;

    type TestParser = Parser<IntoChars>;

    fn parser(code: &str) -> TestParser {
        let s = String::from(code);
        Parser::new(Scanner::new(s.into_chars().peekable()))
    }

    fn parse(s: &str) -> AstNode {
        parser(s).parse_expression(0)
    }

    // ---- Literals ----

    #[test]
    fn int_lit() {
        assert_eq!(parse("42"), AstNode::make_expr(ExprKind::IntLit(42)));
    }

    #[test]
    fn char_lit() {
        assert_eq!(parse("'a'"), AstNode::make_expr(ExprKind::CharLit('a')));
    }

    #[test]
    fn string_lit() {
        assert!(matches!(parse("\"hello\""), AstNode::Expr{ kind: ExprKind::StringLit(_), .. }))
    }

    #[test]
    fn bool_true() {
        assert_eq!(parse("true"), AstNode::make_expr(ExprKind::BoolVal(true)));
    }

    #[test]
    fn bool_false() {
        assert_eq!(parse("false"), AstNode::make_expr(ExprKind::BoolVal(false)));
    }

    #[test]
    fn ident() {
        assert!(matches!(parse("x"), AstNode::Expr{ kind: ExprKind::Ident(_), .. }));
    }

    // ---- Grouping ----

    #[test]
    fn grouped_expr() {
        assert_eq!(parse("(42)"), AstNode::make_expr(ExprKind::IntLit(42)));
    }

    #[test]
    fn nested_grouped_expr() {
        assert_eq!(parse("((42))"), AstNode::make_expr(ExprKind::IntLit(42)));
    }

    #[test]
    fn grouped_binary() {
        assert_eq!(
            parse("(1 + 2) * 3"),
            AstNode::make_binary(
                Token::at(TokenKind::Star, Pos::at(1, 1)),
                AstNode::make_binary(
                    Token::at(TokenKind::Plus, Pos::at(1, 1)),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2))),
                AstNode::make_expr(ExprKind::IntLit(3))),
        );
    }

    // ---- Unary ----

    #[test]
    fn unary_minus_int() {
        assert_eq!(
            parse("-5"),
            AstNode::make_minus(AstNode::make_expr(ExprKind::IntLit(5)))
        );
    }

    #[test]
    fn unary_not_true() {
        assert_eq!(
            parse("!true"),
            AstNode::make_not(AstNode::make_expr(ExprKind::BoolVal(true)))
        );
    }

    #[test]
    fn unary_minus_grouped() {
        assert_eq!(
            parse("-(1 + 2)"),
            AstNode::make_minus(
                AstNode::make_binary(
                    Token::at(TokenKind::Plus, Pos::at(1, 1)),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2)),
                ))
        );
    }

    // ---- Postfix ----

    #[test]
    fn post_increment() {
        assert_eq!(
            parse("a++"),
            AstNode::make_expr(ExprKind::Incr(0))
        );
    }

    #[test]
    fn post_decrement() {
        assert_eq!(
            parse("a--"),
            AstNode::make_expr(ExprKind::Decr(0))
        );
    }

    // ---- Binary Operators ----

    #[rstest]
    #[case("+", AstNode::make_binary(Token::at(TokenKind::Plus, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("-", AstNode::make_binary(Token::at(TokenKind::Minus, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("*", AstNode::make_binary(Token::at(TokenKind::Star, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("/", AstNode::make_binary(Token::at(TokenKind::Slash, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("%", AstNode::make_binary(Token::at(TokenKind::Mod, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("^", AstNode::make_binary(Token::at(TokenKind::Caret, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("==", AstNode::make_binary(Token::at(TokenKind::Eql, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("!=", AstNode::make_binary(Token::at(TokenKind::Neq, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("<", AstNode::make_binary(Token::at(TokenKind::Lss, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("<=", AstNode::make_binary(Token::at(TokenKind::Leq, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case(">", AstNode::make_binary(Token::at(TokenKind::Gtr, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case(">=", AstNode::make_binary(Token::at(TokenKind::Geq, Pos::at(1, 1)), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2))))]
    #[case("&&", AstNode::make_binary(Token::at(TokenKind::And, Pos::at(1, 1)), AstNode::make_expr(ExprKind::BoolVal(true)), AstNode::make_expr(ExprKind::BoolVal(false))))]
    #[case("||", AstNode::make_binary(Token::at(TokenKind::Or, Pos::at(1, 1)), AstNode::make_expr(ExprKind::BoolVal(true)), AstNode::make_expr(ExprKind::BoolVal(false))))]
    fn binary_ops(#[case] op: &str, #[case] expected: AstNode) {
        let input = format!("1 {} 2", op);
        let input_bool = format!("true {} false", op);
        let s = if op == "&&" || op == "||" { &input_bool } else { &input };
        assert_eq!(parse(s), expected);
    }

    // ---- Precedence & Associativity ----

    #[test]
    fn prec_add_mul() {
        assert_eq!(
            parse("1 + 2 * 3"),
            AstNode::make_binary(TokenKind::Plus.into(),
                                 AstNode::make_expr(ExprKind::IntLit(1)),
                                 AstNode::make_binary(TokenKind::Star.into(),
                                                      AstNode::make_expr(ExprKind::IntLit(2)),
                                                      AstNode::make_expr(ExprKind::IntLit(3))))
        );
    }

    #[test]
    fn prec_mul_add() {
        assert_eq!(
            parse("1 * 2 + 3"),
            AstNode::make_binary(TokenKind::Plus.into(),
                                 AstNode::make_binary(TokenKind::Star.into(),
                                                      AstNode::make_expr(ExprKind::IntLit(1)),
                                                      AstNode::make_expr(ExprKind::IntLit(2))),
                                 AstNode::make_expr(ExprKind::IntLit(3))),
        );
    }

    #[test]
    fn prec_unary_minus_add() {
        assert_eq!(
            parse("-1 + 2"),
            AstNode::make_binary(TokenKind::Plus.into(),
                                 AstNode::make_minus(AstNode::make_expr(ExprKind::IntLit(1))),
                                 AstNode::make_expr(ExprKind::IntLit(2)))
        );
    }

    #[test]
    fn prec_not_eq() {
        assert_eq!(
            parse("!true == false"),
            AstNode::make_binary(TokenKind::Eql.into(),
                AstNode::make_not(AstNode::make_expr(ExprKind::BoolVal(true))),
                AstNode::make_expr(ExprKind::BoolVal(false)))
        );
    }

    #[test]
    fn prec_caret() {
        // '^' has highest precedence (60), so 1 ^ 2 + 3 -> (1 ^ 2) + 3
        assert_eq!(
            parse("1 ^ 2 + 3"),
            AstNode::make_binary(TokenKind::Plus.into(),
                AstNode::make_binary(TokenKind::Caret.into(),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2))),
                AstNode::make_expr(ExprKind::IntLit(3))),
        );
    }

    #[test]
    fn left_assoc_add() {
        assert_eq!(
            parse("1 + 2 + 3"),
            AstNode::make_binary(TokenKind::Plus.into(),
                AstNode::make_binary(TokenKind::Plus.into(),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2))),
                AstNode::make_expr(ExprKind::IntLit(3)))
        );
    }

    #[test]
    fn left_assoc_mul() {
        assert_eq!(
            parse("1 * 2 * 3"),
            AstNode::make_binary(TokenKind::Star.into(),
                AstNode::make_binary(TokenKind::Star.into(),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2))),
                AstNode::make_expr(ExprKind::IntLit(3))),
        );
    }

    #[test]
    fn left_assoc_compare() {
        assert_eq!(
            parse("1 < 2 < 3"),
            AstNode::make_binary(TokenKind::Lss.into(),
                AstNode::make_binary(TokenKind::Lss.into(),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2))),
                AstNode::make_expr(ExprKind::IntLit(3))),
        );
    }

    #[test]
    fn mixed_prec_complex() {
        assert_eq!(
            parse("-1 + 2 * 3 ^ 4"),
            AstNode::make_binary(TokenKind::Plus.into(),
                AstNode::make_minus(AstNode::make_expr(ExprKind::IntLit(1))),
                AstNode::make_binary(TokenKind::Star.into(),
                    AstNode::make_expr(ExprKind::IntLit(2)),
                    AstNode::make_binary(TokenKind::Caret.into(),
                        AstNode::make_expr(ExprKind::IntLit(3)),
                        AstNode::make_expr(ExprKind::IntLit(4)))))
        );
    }

    // ---- Function Calls ----

    #[test]
    fn func_call_no_args() {
        let result = parse("foo()");
        assert!(matches!(result, AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } if params.is_empty()));
    }

    #[test]
    fn func_call_one_arg() {
        let result = parse("foo(42)");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } => {
                assert_eq!(params, vec![AstNode::make_expr(ExprKind::IntLit(42))]);
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn func_call_multiple_args() {
        let result = parse("foo(1, 2, 3)");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } => {
                assert_eq!(params, vec![
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2)),
                    AstNode::make_expr(ExprKind::IntLit(3)),
                ]);
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn func_call_nested() {
        let result = parse("foo(bar())");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } => {
                assert_eq!(params.len(), 1);
                assert!(matches!(params[0], AstNode::Expr { kind: ExprKind::FuncCall { ref params, .. }, ..} if params.is_empty()));
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn func_call_arg_with_expr() {
        let result = parse("foo(1 + 2)");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, ..  }, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(
                    params[0],
                    AstNode::make_binary(TokenKind::Plus.into(),
                        AstNode::make_expr(ExprKind::IntLit(1)),
                        AstNode::make_expr(ExprKind::IntLit(2))))
            }
            _ => panic!("expected FuncCall"),
        }
    }

    // ---- Subscript ----

    #[test]
    fn subscript_zero() {
        let result = parse("arr[0]");
        match result {
            AstNode::Expr { kind: ExprKind::Subscript { index, .. }, .. } => {
                assert_eq!(index, vec![AstNode::make_expr(ExprKind::IntLit(0))]);
            }
            _ => panic!("expected Subscript"),
        }
    }

    #[test]
    fn subscript_with_expr() {
        let result = parse("arr[i + 1]");
        match result {
            AstNode::Expr { kind: ExprKind::Subscript { index, .. }, .. } => {
                assert!(matches!(index.as_slice(), [AstNode::Expr { kind: ExprKind::Binary { op: BinaryOp::Add, .. }, .. }]))
            }
            _ => panic!("expected Subscript"),
        }
    }

    #[test]
    fn nested_subscript() {
        let result = parse("arr[0][2]");
        match result {
            AstNode::Expr { kind: ExprKind::Subscript {index, .. }, .. } => {
                assert_eq!(index.as_slice(), [AstNode::make_expr(ExprKind::IntLit(0)), AstNode::make_expr(ExprKind::IntLit(2))]);
            }
            _ => panic!("expected Subscript"),
        }
    }

    // ---- min_bp filtering ----

    #[test]
    fn min_bp_filters_lower() {
        // parse_expression with min_bp=50 should stop before '+'
        // So "1 + 2" parsed with min_bp=50 returns just IntLit(1)
        let mut p = parser("1 + 2");
        assert_eq!(p.parse_expression(50), AstNode::make_expr(ExprKind::IntLit(1)));
    }

    #[test]
    fn min_bp_zero_full_expr() {
        // parse_expression with min_bp=0 should consume full expression
        let mut p = parser("1 + 2");
        assert_eq!(
            p.parse_expression(0),
            AstNode::make_binary(TokenKind::Plus.into(),
                AstNode::make_expr(ExprKind::IntLit(1)),
                AstNode::make_expr(ExprKind::IntLit(2))),
        );
    }

    // ---- Error / Panic cases ----

    #[test]
    #[should_panic(expected = "Found EOF expecting an expression")]
    fn parse_empty() {
        parse("");
    }

    #[test]
    #[should_panic(expected = "Found EOF while expecting token ')'")]
    fn missing_close_paren_at_eof() {
        parse("(1 + 2");
    }

    #[test]
    #[should_panic(expected = "Expected ')'")]
    fn missing_close_paren() {
        parse("(1 + 2]");
    }

    #[test]
    #[should_panic(expected = "Found EOF while expecting token ']'")]
    fn subscript_missing_brk_at_eof() {
        parser("arr[0").parse_expression(0);
    }
    #[test]
    #[should_panic(expected = "Expected ']'")]
    fn subscript_missing_brk() {
        parser("arr[0)").parse_expression(0);
    }


    #[test]
    #[should_panic(expected = "Expected ',' or ')'")]
    fn func_call_missing_comma_or_paren() {
        parser("foo(1 2)").parse_expression(0);
    }

    #[test]
    #[should_panic(expected = "Expected identifier before '('")]
    fn func_call_on_non_ident() {
        parser("42()").parse_expression(0);
    }

    #[test]
    #[should_panic(expected = "Expected identifier or ']' before subscript")]
    fn subscript_on_non_ident() {
        parser("42[0]").parse_expression(0);
    }

    #[test]
    #[should_panic(expected = "Expected to find")]
    fn invalid_starts() {
        parser(";").parse_expression(0);
    }

    #[test]
    #[should_panic(expected = "EOF found parsing function call arguments")]
    fn func_call_unclosed() {
        parser("foo(1, 2").parse_expression(0);
    }

    #[test]
    #[should_panic(expected = "Expected binary operator")]
    fn illegal_binary_operator() {
        parser("1 * 2 ! 3").parse_expression(0);
    }

    #[test]
    #[should_panic(expected = "Expected identifier before")]
    fn illegal_incr() {
        parser("1++").parse_expression(0);
    }

    #[test]
    #[should_panic(expected = "Expected identifier before")]
    fn illegal_decr() {
        parser("1--").parse_expression(0);
    }

    // ---- parse_var_decl_type ----

    #[test]
    fn parse_var_decl_type_int() {
        let mut p = parser("integer");
        assert_eq!(p.parse_var_decl_type(), Type::Scalar(PrimType::Int));
    }

    #[test]
    fn parse_var_decl_type_char() {
        let mut p = parser("char");
        assert_eq!(p.parse_var_decl_type(), Type::Scalar(PrimType::Char));
    }

    #[test]
    fn parse_var_decl_type_bool() {
        let mut p = parser("boolean");
        assert_eq!(p.parse_var_decl_type(), Type::Scalar(PrimType::Bool));
    }

    #[test]
    fn parse_var_decl_type_string() {
        let mut p = parser("string");
        assert_eq!(p.parse_var_decl_type(), Type::Scalar(PrimType::String));
    }

    #[test]
    fn parse_var_decl_type_array() {
        let mut p = parser("array [5] integer");
        assert_eq!(
            p.parse_var_decl_type(),
            Type::make_array(5, Type::Scalar(PrimType::Int))
        );
    }

    #[test]
    fn parse_var_decl_type_nested_array() {
        let mut p = parser("array [3] array [3] char");
        assert_eq!(
            p.parse_var_decl_type(),
            Type::make_array(3, Type::make_array(3, Type::Scalar(PrimType::Char)))
        );
    }

    #[test]
    #[should_panic(expected = "Expected a type declaration")]
    fn parse_var_decl_type_invalid() {
        parser("foo").parse_var_decl_type();
    }

    #[test]
    #[should_panic(expected = "Found EOF while expecting a type declaration")]
    fn parse_var_decl_type_eof() {
        parser("").parse_var_decl_type();
    }

    // ---- declaration ----

    fn parse_decl(s: &str) -> AstNode {
        parser(s).declaration().expect("expected a declaration")
    }

    #[test]
    fn var_decl_no_init() {
        assert_eq!(
            parse_decl("x: integer;"),
            AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), None)
        );
    }

    #[test]
    fn var_decl_int_init() {
        assert_eq!(
            parse_decl("x: integer = 42;"),
            AstNode::make_var_decl(
                0,
                Type::Scalar(PrimType::Int),
                Some(AstNode::make_expr(ExprKind::IntLit(42))),
            )
        );
    }

    #[test]
    fn var_decl_char_init() {
        assert_eq!(
            parse_decl("x: char = 'a';"),
            AstNode::make_var_decl(0, Type::Scalar(PrimType::Char), Some(AstNode::make_expr(ExprKind::CharLit('a'))))
        );
    }

    #[test]
    fn var_decl_bool_true() {
        assert_eq!(
            parse_decl("x: boolean = true;"),
            AstNode::make_var_decl(0, Type::Scalar(PrimType::Bool), Some(AstNode::make_expr(ExprKind::BoolVal(true))))
        );
    }

    #[test]
    fn var_decl_bool_false() {
        assert_eq!(
            parse_decl("x: boolean = false;"),
            AstNode::make_var_decl(0, Type::Scalar(PrimType::Bool), Some(AstNode::make_expr(ExprKind::BoolVal(false))))
        );
    }

    #[test]
    fn var_decl_string_init() {
        let result = parse_decl("x: string = \"hello\";");
        match result {
            AstNode::VarDecl { name: 0, dtype: Type::Scalar(PrimType::String), init: Some(init), .. } => {
                assert!(matches!(*init, AstNode::Expr { kind: ExprKind::StringLit(_), .. }));
            }
            _ => panic!("expected VarDecl with StringLit init"),
        }
    }

    #[test]
    fn var_decl_array_type() {
        assert_eq!(
            parse_decl("x: array [10] integer;"),
            AstNode::make_var_decl(0, Type::make_array(10, Type::Scalar(PrimType::Int)), None)
        );
    }

    #[test]
    fn var_decl_array_init() {
        assert_eq!(
            parse_decl("x: array [3] integer = [42, 20];"), // Valid syntax, even if semantically
                                                             // incorrect
            AstNode::make_var_decl(0, Type::make_array(3, Type::Scalar(PrimType::Int)),
                Some(AstNode::ArrayInitializer(vec![AstNode::make_expr(ExprKind::IntLit(42)), AstNode::make_expr(ExprKind::IntLit(20))])))
        );
    }

    // ---- parse_top ----

    #[test]
    fn parse_top_empty() {
        assert_eq!(parser("").parse_top(), vec![]);
    }

    #[test]
    fn parse_top_single() {
        assert_eq!(
            parser("x: integer;").parse_top(),
            vec![AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), None)]
        );
    }

    #[test]
    fn parse_top_multiple() {
        assert_eq!(
            parser("x: integer; y: boolean;").parse_top(),
            vec![
                AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), None),
                AstNode::make_var_decl(1, Type::Scalar(PrimType::Bool), None),
            ]
        );
    }

    // ---- Error cases ----

    #[test]
    fn decl_return_none_on_eof() {
        assert!(parser("").declaration().is_none());
    }

    #[test]
    #[should_panic(expected = "Expected ':'")]
    fn decl_missing_colon() {
        parser("x integer;").declaration();
    }

    #[test]
    #[should_panic(expected = "Expected 'function' or a type")]
    fn decl_invalid_type() {
        parser("x: foo;").declaration();
    }

    #[test]
    #[should_panic(expected = "Expected array size")]
    fn decl_array_unclosed_brk() {
        parser("x: array [ integer;").declaration();
    }

    #[test]
    #[should_panic(expected = "Expected to find '(', '!', '-', a literal, or an identifier")]
    fn decl_missing_init_expr() {
        parser("x: integer = ;").declaration();
    }

    #[test]
    #[should_panic(expected = "Found EOF parsing an initializer")]
    fn decl_eof_after_assign() {
        parser("x: integer =").declaration();
    }

    #[test]
    #[should_panic(expected = "Found EOF parsing a declaration")]
    fn decl_eof_after_colon() {
        parser("x:").declaration();
    }

    #[test]
    #[should_panic(expected = "Expected an identifier at")]
    fn no_declaration_at_top_level() {
        parser("(1 + 2)").parse_top();
    }

    #[test]
    #[should_panic(expected = "Expected array size")]
    fn var_decl_array_decl_missing_size() {
        parse_decl("x: array [] integer = [42];");
    }

    // ---- parse_return_type ----

    fn parse_return_type(s: &str) -> Type {
        parser(s).parse_return_type()
    }

    #[test]
    fn return_type_int() {
        assert_eq!(parse_return_type("integer"), Type::Scalar(PrimType::Int));
    }

    #[test]
    fn return_type_char() {
        assert_eq!(parse_return_type("char"), Type::Scalar(PrimType::Char));
    }

    #[test]
    fn return_type_bool() {
        assert_eq!(parse_return_type("boolean"), Type::Scalar(PrimType::Bool));
    }

    #[test]
    fn return_type_string() {
        assert_eq!(parse_return_type("string"), Type::Scalar(PrimType::String));
    }

    #[test]
    fn return_type_void() {
        assert_eq!(parse_return_type("void"), Type::Scalar(PrimType::Void));
    }

    #[test]
    #[should_panic(expected = "Expected a return type declaration")]
    fn return_type_invalid() {
        parser("foo").parse_return_type();
    }

    #[test]
    #[should_panic(expected = "Found EOF while expecting a type declaration")]
    fn return_type_eof() {
        parser("").parse_return_type();
    }

    // ---- parse_function_signature ----

    fn parse_func_sig(s: &str) -> Type {
        parser(s).parse_function_signature()
    }

    #[test]
    fn func_sig_no_params() {
        assert_eq!(
            parse_func_sig("integer()"),
            Type::Function {
                dtype: Box::new(Type::Scalar(PrimType::Int)),
                params: vec![],
            }
        );
    }

    #[test]
    fn func_sig_void_return() {
        assert_eq!(
            parse_func_sig("void()"),
            Type::Function {
                dtype: Box::new(Type::Scalar(PrimType::Void)),
                params: vec![],
            }
        );
    }

    #[test]
    fn func_sig_one_param() {
        assert_eq!(
            parse_func_sig("integer(x: integer)"),
            Type::make_signature(
                Type::Scalar(PrimType::Int),
                vec![Param { name: 0, dtype: Type::Scalar(PrimType::Int) }],
            )
        );
    }

    #[test]
    fn func_sig_multi_params() {
        assert_eq!(
            parse_func_sig("char(x: boolean, y: string)"),
            Type::make_signature(
                Type::Scalar(PrimType::Char),
                vec![Param { name: 0, dtype: Type::Scalar(PrimType::Bool) },
                     Param { name: 1, dtype: Type::Scalar(PrimType::String) }],
                ),
        );
    }

    #[test]
    #[should_panic(expected = "EOF found while expecting a declaration")]
    fn func_sig_missing_paren() {
        parser("integer(").parse_function_signature();
    }

    #[test]
    #[should_panic(expected = "Expected ',' or ')'")]
    fn func_sig_bad_separator() {
        parser("integer(x: char y: char)").parse_function_signature();
    }

    // ---- parse_block ----

    fn parse_block(s: &str) -> AstNode {
        parser(s).parse_block()
    }

    #[test]
    fn block_empty() {
        assert_eq!(parse_block("{}"), AstNode::EmptyBlock);
    }

    #[test]
    fn block_nested_empty() {
        assert_eq!(
            parse_block("{{}}"),
            AstNode::Block(vec![AstNode::EmptyBlock])
        );
    }

    #[test]
    #[should_panic(expected = "Found EOF expecting a statement")]
    fn block_unclosed() {
        parse_block("{");
    }

    // ---- Block with variable declarations ----

    #[test]
    fn block_var_decl_no_init() {
        assert_eq!(
            parse_block("{x: integer;}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), None)
            ])
        );
    }

    #[test]
    fn block_var_decl_int_init() {
        assert_eq!(
            parse_block("{x: integer = 42;}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), Some(AstNode::make_expr(ExprKind::IntLit(42))))
            ])
        );
    }

    #[test]
    fn block_var_decl_bool_init() {
        assert_eq!(
            parse_block("{x: boolean = true;}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(0, Type::Scalar(PrimType::Bool), Some(AstNode::make_expr(ExprKind::BoolVal(true))))
            ])
        );
    }

    #[test]
    fn block_var_decl_char_init() {
        assert_eq!(
            parse_block("{x: char = 'a';}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(0, Type::Scalar(PrimType::Char), Some(AstNode::make_expr(ExprKind::CharLit('a'))))
            ])
        );
    }

    #[test]
    fn block_var_decl_string_init() {
        let result = parse_block("{x: string = \"hello\";}");
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => {
                match &stmts[0] {
                    AstNode::VarDecl { dtype: Type::Scalar(PrimType::String), init: Some(v), .. }
                        if matches!(**v, AstNode::Expr { kind: ExprKind::StringLit(_), .. }) => {},
                    _ => panic!("expected VarDecl with StringLit"),
                }
            }
            _ => panic!("expected Block with one VarDecl"),
        }
    }

    #[test]
    fn block_var_decl_array() {
        assert_eq!(
            parse_block("{x: array [3] integer = [1, 2, 3];}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(0,
                    Type::make_array(3, Type::Scalar(PrimType::Int)),
                    Some(AstNode::ArrayInitializer(vec![
                        AstNode::make_expr(ExprKind::IntLit(1)),
                        AstNode::make_expr(ExprKind::IntLit(2)),
                        AstNode::make_expr(ExprKind::IntLit(3)),
                    ]))),
            ])
        );
    }

    #[test]
    fn block_var_decl_nested_array() {
        assert_eq!(
            parse_block("{x: array [2] array [3] char = ['a', 'b'];	}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(
                    0,
                    Type::make_array(2, Type::make_array(3, Type::Scalar(PrimType::Char))),
                    Some(AstNode::ArrayInitializer(vec![
                        AstNode::make_expr(ExprKind::CharLit('a')),
                        AstNode::make_expr(ExprKind::CharLit('b')),
                    ])),
                )
            ])
        );
    }

    #[test]
    fn block_multiple_var_decls() {
        assert_eq!(
            parse_block("{a: integer = 1; b: boolean = false;}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(
                    0,
                    Type::Scalar(PrimType::Int),
                    Some(AstNode::make_expr(ExprKind::IntLit(1))),
                ),
                AstNode::make_var_decl(
                    1,
                    Type::Scalar(PrimType::Bool),
                    Some(AstNode::make_expr(ExprKind::BoolVal(false))),
                ),
            ])
        );
    }

    #[test]
    fn block_nested_with_decl() {
        assert_eq!(
            parse_block("{{x: integer = 5;}}"),
            AstNode::Block(vec![
                AstNode::Block(vec![
                    AstNode::make_var_decl(
                        0,
                        Type::Scalar(PrimType::Int),
                        Some(AstNode::make_expr(ExprKind::IntLit(5))),
                    )
                ])
            ])
        );
    }

    // ---- Block with assignment statements ----

    #[test]
    fn block_assign_int() {
        assert_eq!(
            parse_block("{x = 42;}"),
            AstNode::Block(vec![
                AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(42))),
            ])
        );
    }

    #[test]
    fn block_assign_expr() {
        assert_eq!(
            parse_block("{x = 1 + 2;}"),
            AstNode::Block(vec![
                AstNode::make_assignment(0,
                    AstNode::make_binary(TokenKind::Plus.into(),
                        AstNode::make_expr(ExprKind::IntLit(1)),
                        AstNode::make_expr(ExprKind::IntLit(2)))),
            ])
        );
    }

    #[test]
    fn block_assign_bool() {
        assert_eq!(
            parse_block("{x = true;}"),
            AstNode::Block(vec![AstNode::make_assignment(0, AstNode::make_expr(ExprKind::BoolVal(true)))])
        );
    }

    #[test]
    fn block_assign_char() {
        assert_eq!(
            parse_block("{x = 'a';}"),
            AstNode::Block(vec![AstNode::make_assignment(0, AstNode::make_expr(ExprKind::CharLit('a')))])
        );
    }

    #[test]
    fn block_assign_string() {
        let result = parse_block("{x = \"hello\";}");
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => {
                match &stmts[0] {
                    AstNode::Expr { kind: ExprKind::Assignment { lvalue: 0, rvalue, .. }, .. } if matches!(**rvalue, AstNode::Expr { kind: ExprKind::StringLit(_), .. }) => {},
                    _ => panic!("expected Assignment with StringLit"),
                }
            }
            _ => panic!("expected Block with one Assignment"),
        }
    }

    #[test]
    fn block_assign_unary_minus() {
        assert_eq!(
            parse_block("{x = -5;}"),
            AstNode::Block(vec![
                AstNode::make_assignment(0,
                    AstNode::make_minus(AstNode::make_expr(ExprKind::IntLit(5)))),
            ])
        );
    }

    #[test]
    fn block_assign_not() {
        assert_eq!(
            parse_block("{x = !true;}"),
            AstNode::Block(vec![
                AstNode::make_assignment(0,
                    AstNode::make_not(AstNode::make_expr(ExprKind::BoolVal(true)))),
            ])
        );
    }

    #[test]
    fn block_assign_func_call() {
        let result = parse_block("{x = foo(42);}");
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => {
                match &stmts[0] {
                    AstNode::Expr { kind: ExprKind::Assignment { lvalue: 0, rvalue }, .. } => {
                        assert!(matches!(**rvalue, AstNode::Expr { kind: ExprKind::FuncCall { .. }, .. }));
                    }
                    _ => panic!("expected Assignment with FuncCall"),
                }
            }
            _ => panic!("expected Block with one Assignment"),
        }
    }

    #[test]
    fn block_assign_subscript() {
        let result = parse_block("{x = arr[0];}");
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => {
                match &stmts[0] {
                    AstNode::Expr { kind: ExprKind::Assignment { lvalue: 0, rvalue }, .. } => {
                        assert!(matches!(**rvalue, AstNode::Expr { kind: ExprKind::Subscript { .. }, .. }));
                    },
                    _ => panic!("expected Assignment with Subscript"),
                }
            }
            _ => panic!("expected Block with one Assignment"),
        }
    }

    #[test]
    fn block_assign_grouped() {
        assert_eq!(
            parse_block("{x = (42);}"),
            AstNode::Block(vec![AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(42)))])
        );
    }

    #[test]
    fn block_assign_multiple() {
        assert_eq!(
            parse_block("{x = 1; y = 2;}"),
            AstNode::Block(vec![
                AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(1))),
                AstNode::make_assignment(1, AstNode::make_expr(ExprKind::IntLit(2))),
            ])
        );
    }

    #[test]
    fn block_mixed_assign_and_decl() {
        assert_eq!(
            parse_block("{x: integer = 1; y = x;}"),
            AstNode::Block(vec![
                AstNode::make_var_decl(
                    0,
                    Type::Scalar(PrimType::Int),
                    Some(AstNode::make_expr(ExprKind::IntLit(1))),
                ),
                AstNode::make_assignment(1, AstNode::make_ident_expr(0))
            ])
        );
    }

    #[test]
    fn block_nested_assign() {
        assert_eq!(
            parse_block("{{x = 42;}}"),
            AstNode::Block(vec![
                AstNode::Block(vec![
                    AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(42)))
                ])
            ])
        );
    }

    #[test]
    #[should_panic(expected = "Expected ';'")]
    fn block_assign_missing_semi() {
        parse_block("{x = 42}");
    }

    #[test]
    #[should_panic(expected = "Expected to find")]
    fn block_assign_missing_expr() {
        parse_block("{x = ;}");
    }

    #[test]
    #[should_panic(expected = "Expected ';', found ,")]
    fn block_assign_invalid_token() {
        parse_block("{x , 42;}");
    }

    #[test]
    #[should_panic(expected = "Found EOF while expecting token ';'")]
    fn block_assign_eof_after_expr() {
        parse_block("{x = 42");
    }

    // ---- if statements ----

    #[test]
    fn if_only() {
        assert_eq!(
            parse_block("{if (true) { x = 1; }}"),
            AstNode::Block(vec![
                AstNode::make_if(
                    AstNode::make_expr(ExprKind::BoolVal(true)),
                    AstNode::Block(vec![
                        AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(1)))
                    ]),
                    AstNode::EmptyBlock,
                )
            ])
        );
    }

    #[test]
    fn if_else() {
        assert_eq!(
            parse_block("{if (true) { x = 1; } else { y = 2; }}"),
            AstNode::Block(vec![
                AstNode::make_if(
                    AstNode::make_expr(ExprKind::BoolVal(true)),
                    AstNode::Block(vec![
                        AstNode::make_assignment(
                            0,
                            AstNode::make_expr(ExprKind::IntLit(1)),
                        )
                    ]),
                    AstNode::Block(vec![
                        AstNode::make_assignment(1, AstNode::make_expr(ExprKind::IntLit(2)))
                    ]),
                )
            ])
        );
    }

    #[test]
    fn if_with_complex_cond() {
        assert_eq!(
            parse_block("{if (1 + 2) { x = 1; }}"),
            AstNode::Block(vec![
                AstNode::make_if(
                    AstNode::make_binary(TokenKind::Plus.into(),
                        AstNode::make_expr(ExprKind::IntLit(1)),
                        AstNode::make_expr(ExprKind::IntLit(2))),
                    AstNode::Block(vec![
                        AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(1))),
                        ]),
                    AstNode::EmptyBlock,
                ),
            ])
        );
    }

    #[test]
    fn if_with_var_decl_in_body() {
        assert_eq!(
            parse_block("{if (a) { x: integer = 42; }}"),
            AstNode::Block(vec![
                AstNode::make_if(
                    AstNode::make_ident_expr(0),
                    AstNode::Block(vec![
                        AstNode::make_var_decl(1, Type::Scalar(PrimType::Int), Some(AstNode::make_expr(ExprKind::IntLit(42))))
                    ]),
                    AstNode::EmptyBlock,
                )
            ])
        );
    }

    #[test]
    fn if_else_if() {
        assert_eq!(
            parse_block("{if (a) { x = 1; } else { if (b) { y = 2; } }}"),
            AstNode::Block(vec![
                AstNode::make_if(
                    AstNode::make_ident_expr(0),
                    AstNode::Block(vec![
                        AstNode::make_assignment(1, AstNode::make_expr(ExprKind::IntLit(1)))
                    ]),
                    AstNode::Block(vec![
                        AstNode::If {
                            cond: Box::new(AstNode::make_ident_expr(2)),
                            t_branch: Box::new(AstNode::Block(vec![
                                AstNode::make_assignment(3, AstNode::make_expr(ExprKind::IntLit(2)))
                            ])),
                            f_branch: Box::new(AstNode::EmptyBlock),
                        }
                    ]),
                )
            ])
        );
    }

    #[test]
    #[should_panic(expected = "Expected '('")]
    fn if_missing_lparen() {
        parse_block("{if true { x = 1; }}");
    }

    #[test]
    #[should_panic(expected = "Expected ')'")]
    fn if_missing_rparen() {
        parse_block("{if (true { x = 1; }}");
    }

    // ---- for statements ----

    fn parse_for_stmt(s: &str) -> AstNode {
        let result = parse_block(s);
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => stmts.into_iter().next().unwrap(),
            _ => panic!("expected Block with one statement"),
        }
    }

    #[test]
    fn for_minimal() {
        assert_eq!(
            parse_for_stmt("{for (; true;) {}}"),
            AstNode::For {
                assign: vec![],
                cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
                post_op: vec![],
                body: Box::new(AstNode::EmptyBlock),
            }
        );
    }

    #[test]
    fn for_with_assign_init() {
        assert_eq!(
            parse_for_stmt("{for (x = 0; true;) {}}"),
            AstNode::make_for(
                vec![AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(0)))],
                AstNode::make_expr(ExprKind::BoolVal(true)),
                vec![],
                AstNode::EmptyBlock,
            )
        );
    }

    #[test]
    fn for_with_multiple_assign_init() {
        assert_eq!(
            parse_for_stmt("{for (x = 0, y = 1; true;) {}}"),
            AstNode::make_for(
                vec![
                    AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(0))),
                    AstNode::make_assignment(1, AstNode::make_expr(ExprKind::IntLit(1))),
                ],
                AstNode::make_expr(ExprKind::BoolVal(true)),
                vec![],
                AstNode::EmptyBlock,
            )
        );
    }

    #[test]
    fn for_with_cond() {
        assert_eq!(
            parse_for_stmt("{for (; a < b;) {}}"),
            AstNode::make_for(
                vec![],
                AstNode::make_binary(TokenKind::Lss.into(), AstNode::make_ident_expr(0), AstNode::make_ident_expr(1)),
                vec![],
                AstNode::EmptyBlock),
        );
    }

    #[test]
    fn for_with_post_incr() {
        assert_eq!(
            parse_for_stmt("{for (; true; i++) {}}"),
            AstNode::For {
                assign: vec![],
                cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
                post_op: vec![AstNode::make_expr(ExprKind::Incr(0))],
                body: Box::new(AstNode::EmptyBlock),
            }
        );
    }

    #[test]
    fn for_with_post_decr() {
        assert_eq!(
            parse_for_stmt("{for (; true; i--) {}}"),
            AstNode::For {
                assign: vec![],
                cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
                post_op: vec![AstNode::make_expr(ExprKind::Decr(0))],
                body: Box::new(AstNode::EmptyBlock),
            }
        );
    }

    #[test]
    fn for_with_multiple_post_ops() {
        assert_eq!(
            parse_for_stmt("{for (; true; i++, j--) {}}"),
            AstNode::For {
                assign: vec![],
                cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
                post_op: vec![AstNode::make_expr(ExprKind::Incr(0)), AstNode::make_expr(ExprKind::Decr(1))],
                body: Box::new(AstNode::EmptyBlock),
            }
        );
    }

    #[test]
    fn for_full() {
        assert_eq!(
            parse_for_stmt("{for (x = 0; x < 10; x++) { x = x * 2; }}"),
            AstNode::make_for(
                vec![AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(0)))],
                AstNode::make_binary(TokenKind::Lss.into(),
                    AstNode::make_ident_expr(0),
                    AstNode::make_expr(ExprKind::IntLit(10))),
                vec![AstNode::make_expr(ExprKind::Incr(0))],
                AstNode::Block(vec![
                    AstNode::make_assignment(0,
                        AstNode::make_binary(TokenKind::Star.into(),
                            AstNode::make_ident_expr(0),
                            AstNode::make_expr(ExprKind::IntLit(2)))),
                ])),
        );
    }

    #[test]
    fn for_non_empty_body() {
        assert_eq!(
            parse_for_stmt("{for (; true;) { x: integer = 42; y = x; }}"),
            AstNode::For {
                assign: vec![],
                cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
                post_op: vec![],
                body: Box::new(AstNode::Block(vec![
                    AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), Some(AstNode::make_expr(ExprKind::IntLit(42)))),
                    AstNode::make_assignment(1, AstNode::make_ident_expr(0)),
                ])),
            }
        );
    }

    #[test]
    fn for_nested() {
        assert_eq!(
            parse_for_stmt("{for (; true;) { for (; true;) {} }}"),
            AstNode::For {
                assign: vec![],
                cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
                post_op: vec![],
                body: Box::new(AstNode::Block(vec![
                    AstNode::For {
                        assign: vec![],
                        cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
                        post_op: vec![],
                        body: Box::new(AstNode::EmptyBlock),
                    }
                ])),
            }
        );
    }

    #[test]
    #[should_panic(expected = "Expected '('")]
    fn for_missing_lparen() {
        parse_block("{for ;; {}}");
    }

    #[test]
    #[should_panic(expected = "Expected ';'")]
    fn for_missing_semi() {
        parse_block("{for (; x < 10) {}}");
    }

    #[test]
    #[should_panic(expected = "Expected ',' or ')'")]
    fn for_missing_rparen() {
        parse_block("{for (; true; x++ {}}");
    }

    // ---- function declarations ----

    #[test]
    fn forward_func_no_params() {
        assert_eq!(
            parse_decl("foo: function integer();"),
            AstNode::make_forward(
                0,
                Type::Function {
                    dtype: Box::new(Type::Scalar(PrimType::Int)),
                    params: vec![],
                },
            )
        );
    }

    #[test]
    fn forward_func_with_params() {
        let sig = Type::make_signature(
            Type::Scalar(PrimType::Void),
            vec![
                Param { name: 1, dtype: Type::Scalar(PrimType::Int) },
                Param { name: 2, dtype: Type::Scalar(PrimType::Char) },
            ],
        );
        assert_eq!(
            parse_decl("foo: function void(x: integer, y: char);"),
            AstNode::make_forward(
                0,
                sig,
            )
        );
    }

    #[test]
    fn func_def_empty_body() {
        let signature = Type::Function { dtype: Box::new(Type::Scalar(PrimType::Int)),
                                               params: vec![] };
        assert_eq!(
            parse_decl("foo: function integer() = {}"),
            AstNode::make_function(0, signature, AstNode::EmptyBlock)
        );
    }

    #[test]
    fn func_def_non_empty_body() {
        let body = AstNode::Block(vec![
            AstNode::make_var_decl(1, Type::Scalar(PrimType::Int), Some(AstNode::make_expr(ExprKind::IntLit(42))))
        ]);
        let signature = Type::Function { dtype: Box::new(Type::Scalar(PrimType::Int)),
                                               params: vec![] };
        assert_eq!(
            parse_decl("foo: function integer() = {x: integer = 42;}"),
            AstNode::make_function(0, signature, body)
        );
    }

    #[test]
    fn func_def_void_body() {
        let body = AstNode::Block(vec![
            AstNode::make_var_decl(1, Type::Scalar(PrimType::Char), Some(AstNode::make_expr(ExprKind::CharLit('a'))))
        ]);
        let signature = Type::Function {
            dtype: Box::new(Type::Scalar(PrimType::Void)),
            params: vec![],
        };
        assert_eq!(
            parse_decl("main: function void() = {x: char = 'a';}"),
            AstNode::make_function(0, signature, body)
        );
    }

    #[test]
    #[should_panic(expected = "Found EOF when parsing a function")]
    fn func_decl_missing_semi_or_assign() {
        parser("foo: function integer()").declaration();
    }

    #[test]
    #[should_panic(expected = "Expected ';' or '='")]
    fn func_decl_unexpected_after_sig() {
        parser("foo: function integer() x").declaration();
    }

    // ---- parse_top with functions ----

    #[test]
    fn parse_top_func_only() {
        assert_eq!(
            parser("foo: function integer();").parse_top(),
            vec![AstNode::make_forward(
                0,
                Type::Function {
                    dtype: Box::new(Type::Scalar(PrimType::Int)),
                    params: vec![],
                },
            )]
        );
    }

    #[test]
    fn parse_top_var_decl_and_func() {
        assert_eq!(
            parser("x: integer; foo: function integer();").parse_top(),
            vec![
                AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), None),
                AstNode::make_forward(
                    1,
                    Type::Function {
                        dtype: Box::new(Type::Scalar(PrimType::Int)),
                        params: vec![],
                    },
                ),
            ]
        );
    }

    // ---- parse_top with function definitions ----

    #[test]
    fn parse_top_func_def_empty() {
        let signature = Type::Function {
            dtype: Box::new(Type::Scalar(PrimType::Int)),
            params: vec![],
        };
        assert_eq!(
            parser("foo: function integer() = {}").parse_top(),
            vec![AstNode::make_function(0, signature, AstNode::EmptyBlock)]
        );
    }

    #[test]
    fn parse_top_func_def_with_body() {
        let body = AstNode::Block(vec![
            AstNode::make_var_decl(1, Type::Scalar(PrimType::Int), Some(AstNode::make_expr(ExprKind::IntLit(42))))
        ]);
        let signature = Type::Function {
            dtype: Box::new(Type::Scalar(PrimType::Int)),
            params: vec![],
        };
        assert_eq!(
            parser("foo: function integer() = {x: integer = 42;}").parse_top(),
            vec![AstNode::make_function(0, signature, body)]
        );
    }

    #[test]
    fn parse_top_mixed_forward_and_def() {
        let result = parser("foo: function integer(); bar: function void() = {}").parse_top();
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], AstNode::ForwardFunction { name: 0, .. }));
        assert!(matches!(result[1], AstNode::Function { name: 1, .. }));
    }

    // ---- print statements ----

    fn parse_print_stmt(s: &str) -> AstNode {
        let result = parse_block(s);
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => stmts.into_iter().next().unwrap(),
            _ => panic!("expected Block with one statement"),
        }
    }

    #[test]
    fn print_no_args() {
        assert_eq!(
            parse_print_stmt("{print;}"),
            AstNode::Print(vec![])
        );
    }

    #[test]
    fn print_one_int() {
        assert_eq!(
            parse_print_stmt("{print 42;}"),
            AstNode::Print(vec![AstNode::make_expr(ExprKind::IntLit(42))])
        );
    }

    #[test]
    fn print_one_string() {
        let result = parse_print_stmt("{print \"hello\";}");
        match result {
            AstNode::Print(args) => {
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], AstNode::Expr { kind: ExprKind::StringLit(_), .. }));
            }
            _ => panic!("expected Print"),
        }
    }

    #[test]
    fn print_multi_expr() {
        assert_eq!(
            parse_print_stmt("{print 1, 2, 3;}"),
            AstNode::Print(vec![
                AstNode::make_expr(ExprKind::IntLit(1)),
                AstNode::make_expr(ExprKind::IntLit(2)),
                AstNode::make_expr(ExprKind::IntLit(3)),
            ])
        );
    }

    #[test]
    fn print_with_binary() {
        assert_eq!(
            parse_print_stmt("{print 1 + 2;}"),
            AstNode::Print(vec![
                AstNode::make_binary(TokenKind::Plus.into(),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2))),
            ])
        );
    }

    #[test]
    fn print_identifiers() {
        let result = parse_print_stmt("{print a, b, c;}");
        match result {
            AstNode::Print(args) => {
                assert_eq!(args.len(), 3);
                assert!(args.iter().all(|a| matches!(a, AstNode::Expr{kind: ExprKind::Ident(_), ..})));
            }
            _ => panic!("expected Print"),
        }
    }

    #[test]
    fn print_with_postfix() {
        let result = parse_print_stmt("{print x++;}");
        match result {
            AstNode::Print(args) => {
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], AstNode::Expr { kind: ExprKind::Incr(_), .. }));
            }
            _ => panic!("expected Print"),
        }
    }

    // ---- return statements ----

    fn parse_return_stmt(s: &str) -> AstNode {
        let result = parse_block(s);
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => stmts.into_iter().next().unwrap(),
            _ => panic!("expected Block with one statement"),
        }
    }

    #[test]
    fn return_no_expr() {
        assert_eq!(
            parse_return_stmt("{return;}"),
            AstNode::VoidReturn
        );
    }

    #[test]
    fn return_int() {
        assert_eq!(
            parse_return_stmt("{return 42;}"),
            AstNode::make_return(AstNode::make_expr(ExprKind::IntLit(42)))
        );
    }

    #[test]
    fn return_ident() {
        assert_eq!(
            parse_return_stmt("{return x;}"),
            AstNode::make_return(AstNode::make_ident_expr(0))
        );
    }

    #[test]
    fn return_complex_expr() {
        assert_eq!(
            parse_return_stmt("{return 1 + 2;}"),
            AstNode::make_return(
                AstNode::make_binary(TokenKind::Plus.into(),
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2)))),
        );
    }

    #[test]
    fn return_bool() {
        assert_eq!(
            parse_return_stmt("{return true;}"),
            AstNode::make_return(AstNode::make_expr(ExprKind::BoolVal(true)))
        );
    }

    #[test]
    fn return_char() {
        assert_eq!(
            parse_return_stmt("{return 'a';}"),
            AstNode::make_return(AstNode::make_expr(ExprKind::CharLit('a')))
        );
    }

    #[test]
    fn return_string() {
        let result = parse_return_stmt("{return \"hello\";}");
        match result {
            AstNode::Return(expr) => {
                assert!(matches!(*expr, AstNode::Expr{ kind: ExprKind::StringLit(_), ..}));
            }
            _ => panic!("expected Return with StringLit"),
        }
    }

    #[test]
    fn return_unary_minus() {
        assert_eq!(
            parse_return_stmt("{return -5;}"),
            AstNode::make_return(AstNode::make_minus(AstNode::make_expr(ExprKind::IntLit(5))))
        );
    }

    #[test]
    fn return_func_call() {
        let result = parse_return_stmt("{return foo(42);}");
        match result {
            AstNode::Return(expr) => {
                assert!(matches!(*expr, AstNode::Expr { kind: ExprKind::FuncCall { .. }, .. }));
            }
            _ => panic!("expected Return with FuncCall"),
        }
    }

    #[test]
    fn return_post_incr() {
        let result = parse_return_stmt("{return x++;}");
        match result {
            AstNode::Return(expr) => {
                assert!(matches!(*expr, AstNode::Expr { kind: ExprKind::Incr(_), .. }))
            }
            _ => panic!("expected Return with Incr"),
        }
    }

    #[test]
    #[should_panic(expected = "Expected to find")]
    fn return_missing_semi_no_expr() {
        parse_block("{return}");
    }

    #[test]
    #[should_panic(expected = "Expected ';'")]
    fn return_missing_semi_with_expr() {
        parse_block("{return 42}");
    }

    // ---- function call statements (NOT YET IMPLEMENTED) ----
    // These tests assert the correct AST once the feature is implemented.
    // They currently fail because parse_block cannot handle `ident(...)` as a statement.

    fn parse_func_call_stmt(s: &str) -> AstNode {
        let result = parse_block(s);
        match result {
            AstNode::Block(stmts) if stmts.len() == 1 => stmts.into_iter().next().unwrap(),
            _ => panic!("expected Block with one statement"),
        }
    }

    #[test]
    fn func_call_stmt_no_args() {
        let result = parse_func_call_stmt("{foo();}");
        assert!(matches!(result, AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } if params.is_empty()));
    }

    #[test]
    fn func_call_stmt_with_args() {
        let result = parse_func_call_stmt("{foo(42);}");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } => {
                assert_eq!(params, vec![AstNode::make_expr(ExprKind::IntLit(42))]);
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn func_call_stmt_multiple_args() {
        let result = parse_func_call_stmt("{foo(1, 2, 3);}");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } => {
                assert_eq!(params, vec![
                    AstNode::make_expr(ExprKind::IntLit(1)),
                    AstNode::make_expr(ExprKind::IntLit(2)),
                    AstNode::make_expr(ExprKind::IntLit(3)),
                ]);
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn func_call_stmt_arg_with_expr() {
        let result = parse_func_call_stmt("{foo(1 + 2);}");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(
                    params[0],
                    AstNode::make_binary(TokenKind::Plus.into(),
                        AstNode::make_expr(ExprKind::IntLit(1)),
                        AstNode::make_expr(ExprKind::IntLit(2))))
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn func_call_stmt_nested() {
        let result = parse_func_call_stmt("{foo(bar());}");
        match result {
            AstNode::Expr { kind: ExprKind::FuncCall { params, .. }, .. } => {
                assert_eq!(params.len(), 1);
                assert!(matches!(params[0], AstNode::Expr { kind: ExprKind::FuncCall { ref params, .. }, .. } if params.is_empty()));
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn func_call_stmt_after_var_decl() {
        let result = parse_block("{x: integer = 1; foo();}");
        match result {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 2);
                assert!(matches!(stmts[0], AstNode::VarDecl { .. }));
                assert!(matches!(stmts[1], AstNode::Expr { kind: ExprKind::FuncCall { .. }, .. }));
            }
            _ => panic!("expected Block"),
        }
    }

    #[test]
    fn func_call_stmt_before_assign() {
        let result = parse_block("{foo(); x = 1;}");
        match result {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 2);
                assert!(matches!(stmts[0], AstNode::Expr { kind: ExprKind::FuncCall { .. }, .. }));
                assert!(matches!(stmts[1], AstNode::Expr { kind: ExprKind::Assignment { .. }, .. }));
            }
            _ => panic!("expected Block"),
        }
    }

    #[test]
    fn func_call_stmt_multiple_calls() {
        let result = parse_block("{foo(); bar(); baz();}");
        match result {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 3);
                assert!(stmts.iter().all(|s| matches!(s, AstNode::Expr { kind: ExprKind::FuncCall { .. }, .. })));
            }
            _ => panic!("expected Block"),
        }
    }

    // ---- Expression statements in blocks ----

    #[test]
    fn block_expr_stmt_literal() {
        assert_eq!(
            parse_block("{ 42 }"),
            AstNode::Block(vec![AstNode::make_expr(ExprKind::IntLit(42))])
        );
    }

    #[test]
    fn block_expr_stmt_ident() {
        assert_eq!(
            parse_block("{ y; }"),
            AstNode::Block(vec![AstNode::make_ident_expr(0)])
        );
    }

    #[test]
    fn block_expr_stmt_binary() {
        assert_eq!(
            parse_block("{ x + 1; }"),
            AstNode::Block(vec![
                AstNode::make_binary(TokenKind::Plus.into(),
                    AstNode::make_ident_expr(0),
                    AstNode::make_expr(ExprKind::IntLit(1)))
            ])
        );
    }

    // ---- parse_assign with non-assignment expression ----

    #[test]
    #[should_panic(expected = "Expected an identifier, got")]
    fn test_parse_assign_non_assign() {
        parse_block("{for (x + 1; true;) {}}");
    }

    // ---- parse_function_signature with initializer in param ----

    #[test]
    #[should_panic(expected = "No initialization allowed in function signatures")]
    fn test_func_sig_init_in_param() {
        parser("integer(x: integer = 42)").parse_function_signature();
    }

    // ---- parse_var_decl with non-ident start ----

    #[test]
    #[should_panic(expected = "Expected a variable declaration")]
    fn test_parse_var_decl_invalid_start() {
        parser("integer(42: integer)").parse_function_signature();
    }

    // ---- parse_top with only comments ----

    #[test]
    fn test_parse_top_line_comment() {
        assert_eq!(parser("// just a comment\n").parse_top(), vec![]);
    }

    #[test]
    fn test_parse_top_block_comment() {
        assert_eq!(parser("/* block comment */").parse_top(), vec![]);
    }

    // ---- parse_var_decl_type with non-array brackets ----

    #[test]
    #[should_panic(expected = "Expected array size")]
    fn test_parse_var_decl_type_array_non_int_size() {
        parser("array [true] integer").parse_var_decl_type();
    }

    #[test]
    #[should_panic(expected = "Found EOF while expecting array size")]
    fn test_parse_var_decl_type_array_size_eof() {
        parser("array [").parse_var_decl_type();
    }

    // ---- Parser error handling: for-loop EOF in post section ----

    #[test]
    #[should_panic(expected = "EOF found when expecting ',' or ')'")]
    fn test_for_post_eof() {
        parse_block("{for (; true; x++");
    }

    // ---- Parser error handling: print statement ----

    #[test]
    #[should_panic(expected = "Expected ',' or ';'")]
    fn test_print_bad_separator() {
        parse_block("{print 1 2;}");
    }

    #[test]
    #[should_panic(expected = "EOF found while expecting ',' or ';'")]
    fn test_print_eof() {
        parse_block("{print 1");
    }

    // ---- Parser error handling: function signature ----

    #[test]
    #[should_panic(expected = "EOF found parsing function call arguments")]
    fn test_func_sig_eof_after_param() {
        parser("foo: function integer(x: integer").declaration();
    }

    // ---- Parser error handling: array initializer ----

    #[test]
    #[should_panic(expected = "Expected ',' or ']'")]
    fn test_array_init_bad_separator() {
        parse_decl("x: array [3] integer = [1 2");
    }

    #[test]
    #[should_panic(expected = "EOF found parsing array initialization")]
    fn test_array_init_eof() {
        parse_decl("x: array [3] integer = [1, 2");
    }
}
