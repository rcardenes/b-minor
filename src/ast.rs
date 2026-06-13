use std::sync::atomic::{AtomicUsize, Ordering};

use derivative::Derivative;

use crate::{
    scan::{Token, TokenKind, fatal_tok},
    types::Type
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    FuncCall { name: usize, params: Vec<AstNode> },
    Subscript { name: usize, index: Box<AstNode> },
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
    Assignment {
        #[derivative(PartialEq="ignore")]
        id: NodeId,
        lvalue: usize, expr: Box<AstNode>
    },

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

    pub fn make_subscript(name: usize, index: AstNode) -> AstNode {
        let index = Box::new(index);
        AstNode::make_expr(ExprKind::Subscript { name, index })
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
        let expr = Box::new(expr);
        let id = Self::get_node_id();
        AstNode::Assignment { lvalue, id, expr }
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
