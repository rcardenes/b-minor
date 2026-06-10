use crate::{
    scan::{Token, TokenKind, fatal_tok},
    types::Type
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstNode {
    // Literals
    BoolVal(bool),
    CharLit(char),
    IntLit(i64),
    StringLit(usize),
    ArrayInitializer(Vec<AstNode>),

    Ident(usize),

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
    Return(Option<Box<AstNode>>),
    Assignment { lvalue: usize, expr: Box<AstNode> },

    // Expressions
    Minus(Box<AstNode>),
    Not(Box<AstNode>),
    Incr(usize),
    Decr(usize),
    And { left: Box<AstNode>, right: Box<AstNode> },
    Or  { left: Box<AstNode>, right: Box<AstNode> },
    Add { left: Box<AstNode>, right: Box<AstNode> },
    Sub { left: Box<AstNode>, right: Box<AstNode> },
    Mul { left: Box<AstNode>, right: Box<AstNode> },
    Div { left: Box<AstNode>, right: Box<AstNode> },
    Mod { left: Box<AstNode>, right: Box<AstNode> },
    Pow { left: Box<AstNode>, right: Box<AstNode> },
    Eql { left: Box<AstNode>, right: Box<AstNode> },
    Neq { left: Box<AstNode>, right: Box<AstNode> },
    Lss { left: Box<AstNode>, right: Box<AstNode> },
    Lte { left: Box<AstNode>, right: Box<AstNode> },
    Grt { left: Box<AstNode>, right: Box<AstNode> },
    Gte { left: Box<AstNode>, right: Box<AstNode> },
    FuncCall { id: usize, params: Vec<AstNode> },
    Subscript { id: usize, index: Box<AstNode> },
}

impl AstNode {
    pub fn make_literal(tok: Token) -> AstNode {
        match tok.kind {
            TokenKind::IntLit(val) => AstNode::IntLit(val),
            TokenKind::CharLit(val) => AstNode::CharLit(val),
            TokenKind::StringLit(index) => AstNode::StringLit(index),
            TokenKind::True => AstNode::BoolVal(true),
            TokenKind::False => AstNode::BoolVal(false),
            _ => fatal_tok("Expected a literal", tok),
        }
    }

    pub fn make_minus(node: AstNode) -> AstNode {
        AstNode::Minus(Box::new(node))
    }

    pub fn make_not(node: AstNode) -> AstNode {
        AstNode::Not(Box::new(node))
    }

    pub fn make_postfix(kind: TokenKind, id: usize) -> AstNode {
        match kind {
            TokenKind::Incr => AstNode::Incr(id),
            TokenKind::Decr => AstNode::Decr(id),
            _ => unreachable!("make_postfix is not supposed to get {kind}"),
        }
    }

    pub fn make_binary(tok: Token, left: AstNode, right: AstNode) -> AstNode {
        let left = Box::new(left);
        let right = Box::new(right);

        match tok.kind {
            TokenKind::And => AstNode::And { left, right },
            TokenKind::Or => AstNode::Or { left, right },
            TokenKind::Plus => AstNode::Add { left, right },
            TokenKind::Minus => AstNode::Sub { left, right },
            TokenKind::Star => AstNode::Mul { left, right },
            TokenKind::Slash => AstNode::Div { left, right },
            TokenKind::Mod => AstNode::Mod { left, right },
            TokenKind::Caret => AstNode::Pow { left, right },
            TokenKind::Eql => AstNode::Eql { left, right },
            TokenKind::Neq => AstNode::Neq { left, right },
            TokenKind::Lss => AstNode::Lss { left, right },
            TokenKind::Leq => AstNode::Lte { left, right },
            TokenKind::Gtr => AstNode::Grt { left, right },
            TokenKind::Geq => AstNode::Gte { left, right },
            _ => unreachable!("make_binary got a {}", tok.kind),
        }
    }

    pub fn make_subscript(id: usize, index: AstNode) -> AstNode {
        let index = Box::new(index);
        AstNode::Subscript { id, index }
    }

    pub fn make_func_call(id: usize, params: Vec<AstNode>) -> AstNode {
        AstNode::FuncCall { id, params }
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

    pub fn make_return(expr: Option<AstNode>) -> AstNode {
        AstNode::Return(expr.map(Box::new))
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
