use std::{
    fmt::Display,
    sync::atomic::{AtomicUsize, Ordering}
};

use derivative::Derivative;

use crate::{
    scan::{Token, TokenKind, fatal_tok},
    types::Type
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eql,
    Neq,
    Lss,
    Lte,
    Grt,
    Gte,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => "^",
            BinaryOp::Eql => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::Lss => "<",
            BinaryOp::Lte => "<=",
            BinaryOp::Grt => ">",
            BinaryOp::Gte => ">="
        };

        write!(f, "{}", name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprKind {
    Ident(usize),
    BoolVal(bool),
    CharLit(char),
    IntLit(i64),
    StringLit(usize),
    Incr(usize),
    Decr(usize),
    Unary(UnaryOp, Box<AstNode>),
    Binary { left: Box<AstNode>, op: BinaryOp, right: Box<AstNode> },
    Assignment { lvalue: usize, rvalue: Box<AstNode> },
    FuncCall { name: usize, params: Vec<AstNode> },
    Subscript { name: usize, index: Vec<AstNode> },
}

impl ExprKind {
    pub fn is_literal(&self) -> bool {
        matches!(self,
            ExprKind::BoolVal(_)
            | ExprKind::CharLit(_)
            | ExprKind::IntLit(_)
            | ExprKind::StringLit(_)
        )
    }
}

#[derive(Clone, Debug, Derivative, Eq)]
#[derivative(PartialEq)]
pub enum AstNode {
    Expr {
        #[derivative(PartialEq="ignore")]
        id: NodeId,
        kind: ExprKind
    },
    Ident {
        #[derivative(PartialEq="ignore")]
        id: NodeId,
        name: usize
    },
    ArrayInitializer(Vec<AstNode>),

    // The 'id' field in the following is an index to a string table
    VarDecl {
        #[derivative(PartialEq="ignore")]
        id: NodeId,
        name: usize, dtype: Type, init: Option<Box<AstNode>>
    },
    Function {
        #[derivative(PartialEq="ignore")]
        id: NodeId,
        name: usize, signature: Type, body: Box<AstNode>
    },
    ForwardFunction {
        #[derivative(PartialEq="ignore")]
        id: NodeId,
        name: usize, signature: Type
    },

    // Statements
    EmptyBlock,
    Block(Vec<AstNode>),
    If { cond: Box<AstNode>, t_branch: Box<AstNode>, f_branch: Box<AstNode> },
    For { assign: Vec<AstNode>, cond: Box<AstNode>, post_op: Vec<AstNode>, body: Box<AstNode> },
    Print(Vec<AstNode>),
    VoidReturn,
    Return(Box<AstNode>),

    // Expressions
}

static NODE_ID: AtomicUsize = AtomicUsize::new(0);

impl AstNode {
    pub fn get_node_id() -> NodeId {
        NodeId(NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
    pub fn make_expr(kind: ExprKind) -> AstNode {
        AstNode::Expr { id: Self::get_node_id(), kind }
    }

    pub fn make_ident_expr(name: usize) -> AstNode {
        AstNode::make_expr(ExprKind::Ident(name))
    }

    pub fn make_ident(name: usize) -> AstNode {
        AstNode::Ident { id: Self::get_node_id(), name }
    }

    pub fn make_literal(tok: Token) -> AstNode {
        match tok.kind {
            TokenKind::IntLit(val) => AstNode::make_expr(ExprKind::IntLit(val)),
            TokenKind::CharLit(val) => AstNode::make_expr(ExprKind::CharLit(val)),
            TokenKind::StringLit(index) => AstNode::make_expr(ExprKind::StringLit(index)),
            TokenKind::True => AstNode::make_expr(ExprKind::BoolVal(true)),
            TokenKind::False => AstNode::make_expr(ExprKind::BoolVal(false)),
            _ => fatal_tok("Expected a literal", tok),
        }
    }

    pub fn make_minus(node: AstNode) -> AstNode {
        AstNode::make_expr(ExprKind::Unary(UnaryOp::Minus, Box::new(node)))
    }

    pub fn make_not(node: AstNode) -> AstNode {
        AstNode::make_expr(ExprKind::Unary(UnaryOp::Not, Box::new(node)))
    }

    pub fn make_postfix(kind: TokenKind, id: usize) -> AstNode {
        match kind {
            TokenKind::Incr => AstNode::make_expr(ExprKind::Incr(id)),
            TokenKind::Decr => AstNode::make_expr(ExprKind::Decr(id)),
            _ => unreachable!("make_postfix is not supposed to get {kind}"),
        }
    }

    pub fn make_binary(tok: Token, left: AstNode, right: AstNode) -> AstNode {
        let left = Box::new(left);
        let right = Box::new(right);

        match tok.kind {
            TokenKind::And => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::And }),
            TokenKind::Or => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Or }),
            TokenKind::Plus => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Add }),
            TokenKind::Minus => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Sub }),
            TokenKind::Star => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Mul }),
            TokenKind::Slash => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Div }),
            TokenKind::Mod => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Mod }),
            TokenKind::Caret => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Pow }),
            TokenKind::Eql => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Eql }),
            TokenKind::Neq => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Neq }),
            TokenKind::Lss => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Lss }),
            TokenKind::Leq => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Lte }),
            TokenKind::Gtr => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Grt }),
            TokenKind::Geq => AstNode::make_expr(ExprKind::Binary { left, right, op: BinaryOp::Gte }),
            _ => unreachable!("make_binary got a {}", tok.kind),
        }
    }

    pub fn make_subscript(node: AstNode, new_index: AstNode) -> AstNode {
        match node {
            AstNode::Expr { kind: ExprKind::Ident(name), .. } => {
                AstNode::make_expr(ExprKind::Subscript { name, index: vec![new_index] })
            },
            AstNode::Expr { kind: ExprKind::Subscript { name, index }, id } => {
                let mut index = index;
                index.push(new_index);
                AstNode::Expr { id, kind: ExprKind::Subscript { name, index } }
            },
            _ => unreachable!()
        }
    }

    pub fn make_func_call(name: usize, params: Vec<AstNode>) -> AstNode {
        AstNode::make_expr(ExprKind::FuncCall { name, params })
    }

    pub fn make_var_decl(name: usize, dtype: Type, init: Option<AstNode>) -> AstNode {
        let init = init.map(Box::new);
        let id = Self::get_node_id();
        AstNode::VarDecl { id, name, dtype, init }
    }

    pub fn make_forward(name: usize, signature: Type) -> AstNode {
        let id = Self::get_node_id();
        AstNode::ForwardFunction { id, name, signature }
    }

    pub fn make_function(name: usize, signature: Type, body: AstNode) -> AstNode {
        let body = Box::new(body);
        let id = Self::get_node_id();
        AstNode::Function { id, name, signature, body }
    }

    pub fn make_assignment(lvalue: usize, expr: AstNode) -> AstNode {
        let rvalue = Box::new(expr);
        AstNode::make_expr(ExprKind::Assignment{ lvalue, rvalue })
    }

    pub fn make_return(expr: AstNode) -> AstNode {
        AstNode::Return(Box::new(expr))
    }

    pub fn make_if(cond: AstNode, true_branch: AstNode, false_branch: AstNode) -> AstNode {
        let cond = Box::new(cond);
        let t_branch = Box::new(true_branch);
        let f_branch = Box::new(false_branch);
        AstNode::If { cond, t_branch, f_branch }
    }

    pub fn make_for(assign: Vec<AstNode>, cond: AstNode, post_op: Vec<AstNode>, body: AstNode) -> AstNode {
        let cond = Box::new(cond);
        let body = Box::new(body);
        AstNode::For { assign, cond, post_op, body }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::Pos;
    use crate::types::PrimType;
    use rstest::rstest;

    fn tok(kind: TokenKind) -> Token {
        Token::at(kind, Pos::at(0, 0))
    }

    // ---- NodeId ----

    #[test]
    fn test_node_id_increments() {
        let a = AstNode::get_node_id();
        let b = AstNode::get_node_id();
        assert_ne!(a, b);
    }

    // ---- ExprKind::is_literal ----

    #[rstest]
    #[case(ExprKind::BoolVal(true))]
    #[case(ExprKind::BoolVal(false))]
    #[case(ExprKind::CharLit('a'))]
    #[case(ExprKind::IntLit(42))]
    #[case(ExprKind::StringLit(0))]
    fn test_is_literal_true(#[case] kind: ExprKind) {
        assert!(kind.is_literal());
    }

    #[rstest]
    #[case(ExprKind::Ident(0))]
    #[case(ExprKind::Incr(0))]
    #[case(ExprKind::Decr(0))]
    #[case(ExprKind::Unary(UnaryOp::Minus, Box::new(AstNode::make_expr(ExprKind::IntLit(0)))))]
    #[case(ExprKind::Binary { left: Box::new(AstNode::make_expr(ExprKind::IntLit(0))), op: BinaryOp::Add, right: Box::new(AstNode::make_expr(ExprKind::IntLit(1))) })]
    #[case(ExprKind::Assignment { lvalue: 0, rvalue: Box::new(AstNode::make_expr(ExprKind::IntLit(0))) })]
    #[case(ExprKind::FuncCall { name: 0, params: vec![] })]
    #[case(ExprKind::Subscript { name: 0, index: vec![] })]
    fn test_is_literal_false(#[case] kind: ExprKind) {
        assert!(!kind.is_literal());
    }

    // ---- AstNode::make_expr ----

    #[test]
    fn test_make_expr() {
        let node = AstNode::make_expr(ExprKind::IntLit(7));
        assert_eq!(node, AstNode::make_expr(ExprKind::IntLit(7)));
    }

    // ---- AstNode::make_ident_expr ----

    #[test]
    fn test_make_ident_expr() {
        let node = AstNode::make_ident_expr(5);
        assert_eq!(node, AstNode::make_expr(ExprKind::Ident(5)));
    }

    // ---- AstNode::make_ident ----

    #[test]
    fn test_make_ident() {
        let node = AstNode::make_ident(3);
        assert!(matches!(node, AstNode::Ident { name: 3, .. }));
    }

    // ---- AstNode::make_literal ----

    #[test]
    fn test_make_literal_int() {
        let node = AstNode::make_literal(tok(TokenKind::IntLit(42)));
        assert_eq!(node, AstNode::make_expr(ExprKind::IntLit(42)));
    }

    #[test]
    fn test_make_literal_char() {
        let node = AstNode::make_literal(tok(TokenKind::CharLit('z')));
        assert_eq!(node, AstNode::make_expr(ExprKind::CharLit('z')));
    }

    #[test]
    fn test_make_literal_string() {
        let node = AstNode::make_literal(tok(TokenKind::StringLit(7)));
        assert_eq!(node, AstNode::make_expr(ExprKind::StringLit(7)));
    }

    #[test]
    fn test_make_literal_true() {
        let node = AstNode::make_literal(tok(TokenKind::True));
        assert_eq!(node, AstNode::make_expr(ExprKind::BoolVal(true)));
    }

    #[test]
    fn test_make_literal_false() {
        let node = AstNode::make_literal(tok(TokenKind::False));
        assert_eq!(node, AstNode::make_expr(ExprKind::BoolVal(false)));
    }

    #[test]
    #[should_panic(expected = "Expected a literal")]
    fn test_make_literal_non_literal() {
        AstNode::make_literal(tok(TokenKind::Plus));
    }

    // ---- AstNode::make_minus ----

    #[test]
    fn test_make_minus() {
        let inner = AstNode::make_expr(ExprKind::IntLit(5));
        let node = AstNode::make_minus(inner);
        assert_eq!(node, AstNode::make_expr(ExprKind::Unary(UnaryOp::Minus, Box::new(AstNode::make_expr(ExprKind::IntLit(5))))));
    }

    // ---- AstNode::make_not ----

    #[test]
    fn test_make_not() {
        let inner = AstNode::make_expr(ExprKind::BoolVal(true));
        let node = AstNode::make_not(inner);
        assert_eq!(node, AstNode::make_expr(ExprKind::Unary(UnaryOp::Not, Box::new(AstNode::make_expr(ExprKind::BoolVal(true))))));
    }

    // ---- AstNode::make_postfix ----

    #[test]
    fn test_make_postfix_incr() {
        let node = AstNode::make_postfix(TokenKind::Incr, 3);
        assert_eq!(node, AstNode::make_expr(ExprKind::Incr(3)));
    }

    #[test]
    fn test_make_postfix_decr() {
        let node = AstNode::make_postfix(TokenKind::Decr, 7);
        assert_eq!(node, AstNode::make_expr(ExprKind::Decr(7)));
    }

    #[test]
    #[should_panic(expected = "make_postfix is not supposed to get")]
    fn test_make_postfix_unreachable() {
        AstNode::make_postfix(TokenKind::Plus, 0);
    }

    // ---- AstNode::make_binary ----

    #[rstest]
    #[case(TokenKind::And, BinaryOp::And)]
    #[case(TokenKind::Or, BinaryOp::Or)]
    #[case(TokenKind::Plus, BinaryOp::Add)]
    #[case(TokenKind::Minus, BinaryOp::Sub)]
    #[case(TokenKind::Star, BinaryOp::Mul)]
    #[case(TokenKind::Slash, BinaryOp::Div)]
    #[case(TokenKind::Mod, BinaryOp::Mod)]
    #[case(TokenKind::Caret, BinaryOp::Pow)]
    #[case(TokenKind::Eql, BinaryOp::Eql)]
    #[case(TokenKind::Neq, BinaryOp::Neq)]
    #[case(TokenKind::Lss, BinaryOp::Lss)]
    #[case(TokenKind::Leq, BinaryOp::Lte)]
    #[case(TokenKind::Gtr, BinaryOp::Grt)]
    #[case(TokenKind::Geq, BinaryOp::Gte)]
    fn test_make_binary(#[case] kind: TokenKind, #[case] op: BinaryOp) {
        let left = AstNode::make_expr(ExprKind::IntLit(1));
        let right = AstNode::make_expr(ExprKind::IntLit(2));
        let result = AstNode::make_binary(tok(kind), left, right);
        let expected = AstNode::make_expr(ExprKind::Binary {
            left: Box::new(AstNode::make_expr(ExprKind::IntLit(1))),
            op,
            right: Box::new(AstNode::make_expr(ExprKind::IntLit(2))),
        });
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "make_binary got a")]
    fn test_make_binary_invalid() {
        AstNode::make_binary(tok(TokenKind::Semi), AstNode::make_expr(ExprKind::IntLit(1)), AstNode::make_expr(ExprKind::IntLit(2)));
    }

    // ---- AstNode::make_subscript ----

    #[test]
    fn test_make_subscript_new() {
        let ident = AstNode::make_ident_expr(0);
        let index = AstNode::make_expr(ExprKind::IntLit(5));
        let result = AstNode::make_subscript(ident, index);
        assert_eq!(result, AstNode::make_expr(ExprKind::Subscript {
            name: 0,
            index: vec![AstNode::make_expr(ExprKind::IntLit(5))],
        }));
    }

    #[test]
    fn test_make_subscript_chained() {
        let ident = AstNode::make_ident_expr(0);
        let idx1 = AstNode::make_expr(ExprKind::IntLit(5));
        let idx2 = AstNode::make_expr(ExprKind::IntLit(7));
        let sub1 = AstNode::make_subscript(ident, idx1);
        let result = AstNode::make_subscript(sub1, idx2);
        assert_eq!(result, AstNode::make_expr(ExprKind::Subscript {
            name: 0,
            index: vec![AstNode::make_expr(ExprKind::IntLit(5)), AstNode::make_expr(ExprKind::IntLit(7))],
        }));
    }

    #[test]
    #[should_panic]
    fn test_make_subscript_invalid() {
        AstNode::make_subscript(AstNode::EmptyBlock, AstNode::make_expr(ExprKind::IntLit(0)));
    }

    // ---- AstNode::make_func_call ----

    #[test]
    fn test_make_func_call_no_args() {
        let result = AstNode::make_func_call(0, vec![]);
        assert_eq!(result, AstNode::make_expr(ExprKind::FuncCall { name: 0, params: vec![] }));
    }

    #[test]
    fn test_make_func_call_with_args() {
        let result = AstNode::make_func_call(1, vec![AstNode::make_expr(ExprKind::IntLit(42))]);
        assert_eq!(result, AstNode::make_expr(ExprKind::FuncCall {
            name: 1,
            params: vec![AstNode::make_expr(ExprKind::IntLit(42))],
        }));
    }

    // ---- AstNode::make_var_decl ----

    #[test]
    fn test_make_var_decl_no_init() {
        let result = AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), None);
        assert_eq!(result, AstNode::VarDecl {
            id: NodeId(0),
            name: 0,
            dtype: Type::Scalar(PrimType::Int),
            init: None,
        });
    }

    #[test]
    fn test_make_var_decl_with_init() {
        let init = AstNode::make_expr(ExprKind::IntLit(42));
        let result = AstNode::make_var_decl(0, Type::Scalar(PrimType::Int), Some(init));
        assert_eq!(result, AstNode::VarDecl {
            id: NodeId(0),
            name: 0,
            dtype: Type::Scalar(PrimType::Int),
            init: Some(Box::new(AstNode::make_expr(ExprKind::IntLit(42)))),
        });
    }

    // ---- AstNode::make_forward ----

    #[test]
    fn test_make_forward() {
        let sig = Type::make_signature(Type::Scalar(PrimType::Int), vec![]);
        let result = AstNode::make_forward(0, sig.clone());
        assert_eq!(result, AstNode::ForwardFunction {
            id: NodeId(0),
            name: 0,
            signature: sig,
        });
    }

    // ---- AstNode::make_function ----

    #[test]
    fn test_make_function() {
        let sig = Type::make_signature(Type::Scalar(PrimType::Void), vec![]);
        let body = AstNode::EmptyBlock;
        let result = AstNode::make_function(0, sig.clone(), body);
        assert_eq!(result, AstNode::Function {
            id: NodeId(0),
            name: 0,
            signature: sig,
            body: Box::new(AstNode::EmptyBlock),
        });
    }

    // ---- AstNode::make_assignment ----

    #[test]
    fn test_make_assignment() {
        let expr = AstNode::make_expr(ExprKind::IntLit(7));
        let result = AstNode::make_assignment(0, expr);
        assert_eq!(result, AstNode::make_expr(ExprKind::Assignment {
            lvalue: 0,
            rvalue: Box::new(AstNode::make_expr(ExprKind::IntLit(7))),
        }));
    }

    // ---- AstNode::make_return ----

    #[test]
    fn test_make_return() {
        let expr = AstNode::make_expr(ExprKind::BoolVal(false));
        let result = AstNode::make_return(expr);
        assert_eq!(result, AstNode::Return(Box::new(AstNode::make_expr(ExprKind::BoolVal(false)))));
    }

    // ---- AstNode::make_if ----

    #[test]
    fn test_make_if() {
        let cond = AstNode::make_expr(ExprKind::BoolVal(true));
        let t_branch = AstNode::EmptyBlock;
        let f_branch = AstNode::EmptyBlock;
        let result = AstNode::make_if(cond, t_branch, f_branch);
        assert_eq!(result, AstNode::If {
            cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
            t_branch: Box::new(AstNode::EmptyBlock),
            f_branch: Box::new(AstNode::EmptyBlock),
        });
    }

    // ---- AstNode::make_for ----

    #[test]
    fn test_make_for() {
        let assign = vec![AstNode::make_assignment(0, AstNode::make_expr(ExprKind::IntLit(0)))];
        let cond = AstNode::make_expr(ExprKind::BoolVal(true));
        let post_op = vec![AstNode::make_expr(ExprKind::Incr(0))];
        let body = AstNode::EmptyBlock;
        let result = AstNode::make_for(assign.clone(), cond, post_op.clone(), body);
        assert_eq!(result, AstNode::For {
            assign,
            cond: Box::new(AstNode::make_expr(ExprKind::BoolVal(true))),
            post_op,
            body: Box::new(AstNode::EmptyBlock),
        });
    }

    // ---- BinaryOp::Display ----

    #[rstest]
    #[case(BinaryOp::And, "&&")]
    #[case(BinaryOp::Or, "||")]
    #[case(BinaryOp::Add, "+")]
    #[case(BinaryOp::Sub, "-")]
    #[case(BinaryOp::Mul, "*")]
    #[case(BinaryOp::Div, "/")]
    #[case(BinaryOp::Mod, "%")]
    #[case(BinaryOp::Pow, "^")]
    #[case(BinaryOp::Eql, "==")]
    #[case(BinaryOp::Neq, "!=")]
    #[case(BinaryOp::Lss, "<")]
    #[case(BinaryOp::Lte, "<=")]
    #[case(BinaryOp::Grt, ">")]
    #[case(BinaryOp::Gte, ">=")]
    fn test_binary_op_display(#[case] op: BinaryOp, #[case] expected: &str) {
        assert_eq!(format!("{}", op), expected);
    }
}
