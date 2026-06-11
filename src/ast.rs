use crate::{
    scan::{Token, TokenKind, fatal_tok},
    types::Type
};

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
    FuncCall { id: usize, params: Vec<AstNode> },
    Subscript { id: usize, index: Box<AstNode> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstNode {
    Expr(ExprKind),
    Ident(usize),
    ArrayInitializer(Vec<AstNode>),

    // The 'id' field in the following is an index to a string table
    VarDecl { id: usize, dtype: Type, init: Option<Box<AstNode>> },
    Function { id: usize, signature: Type, body: Box<AstNode> },
    ForwardFunction { id: usize, signature: Type },

    // Statements
    EmptyBlock,
    Block(Vec<AstNode>),
    If { cond: Box<AstNode>, t_branch: Box<AstNode>, f_branch: Box<AstNode> },
    For { assign: Vec<AstNode>, cond: Box<AstNode>, post_op: Vec<AstNode>, body: Box<AstNode> },
    Print(Vec<AstNode>),
    VoidReturn,
    Return(Box<AstNode>),
    Assignment { lvalue: usize, expr: Box<AstNode> },

    // Expressions
}

impl AstNode {
    pub fn make_literal(tok: Token) -> AstNode {
        match tok.kind {
            TokenKind::IntLit(val) => AstNode::Expr(ExprKind::IntLit(val)),
            TokenKind::CharLit(val) => AstNode::Expr(ExprKind::CharLit(val)),
            TokenKind::StringLit(index) => AstNode::Expr(ExprKind::StringLit(index)),
            TokenKind::True => AstNode::Expr(ExprKind::BoolVal(true)),
            TokenKind::False => AstNode::Expr(ExprKind::BoolVal(false)),
            _ => fatal_tok("Expected a literal", tok),
        }
    }

    pub fn make_minus(node: AstNode) -> AstNode {
        AstNode::Expr(ExprKind::Unary(UnaryOp::Minus, Box::new(node)))
    }

    pub fn make_not(node: AstNode) -> AstNode {
        AstNode::Expr(ExprKind::Unary(UnaryOp::Not, Box::new(node)))
    }

    pub fn make_postfix(kind: TokenKind, id: usize) -> AstNode {
        match kind {
            TokenKind::Incr => AstNode::Expr(ExprKind::Incr(id)),
            TokenKind::Decr => AstNode::Expr(ExprKind::Decr(id)),
            _ => unreachable!("make_postfix is not supposed to get {kind}"),
        }
    }

    pub fn make_binary(tok: Token, left: AstNode, right: AstNode) -> AstNode {
        let left = Box::new(left);
        let right = Box::new(right);

        match tok.kind {
            TokenKind::And => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::And }),
            TokenKind::Or => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Or }),
            TokenKind::Plus => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Add }),
            TokenKind::Minus => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Sub }),
            TokenKind::Star => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Mul }),
            TokenKind::Slash => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Div }),
            TokenKind::Mod => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Mod }),
            TokenKind::Caret => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Pow }),
            TokenKind::Eql => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Eql }),
            TokenKind::Neq => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Neq }),
            TokenKind::Lss => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Lss }),
            TokenKind::Leq => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Lte }),
            TokenKind::Gtr => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Grt }),
            TokenKind::Geq => AstNode::Expr(ExprKind::Binary { left, right, op: BinaryOp::Gte }),
            _ => unreachable!("make_binary got a {}", tok.kind),
        }
    }

    pub fn make_subscript(id: usize, index: AstNode) -> AstNode {
        let index = Box::new(index);
        AstNode::Expr(ExprKind::Subscript { id, index })
    }

    pub fn make_func_call(id: usize, params: Vec<AstNode>) -> AstNode {
        AstNode::Expr(ExprKind::FuncCall { id, params })
    }

    pub fn make_var_decl(id: usize, dtype: Type, init: Option<AstNode>) -> AstNode {
        let init = init.map(Box::new);
        AstNode::VarDecl { id, dtype, init }
    }

    pub fn make_function(id: usize, signature: Type, body: AstNode) -> AstNode {
        let body = Box::new(body);
        AstNode::Function { id, signature, body }
    }

    pub fn make_assignment(lvalue: usize, expr: AstNode) -> AstNode {
        let expr = Box::new(expr);
        AstNode::Assignment { lvalue, expr }
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
