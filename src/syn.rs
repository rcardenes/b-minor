use std::iter::Iterator;
use crate::scan::{Scanner, Token, TokenKind, fatal};

// Primitive types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimType {
    Bool,
    Char,
    Int,
    String,
    Void,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(PrimType),
    Array(Box<Type>),
    Function(Box<Type>), // Temporary, must be extended to cover the whole signature
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstNode {
    // Literals
    CharList(char),
    IntLit(i64),
    StringLit(usize),
    True,
    False,

    // The 'id' field in the following is an index to a string table
    Ident { id: usize, dtype: Type },
    VarDecl { id: usize, dtype: Type },
    Function { id: usize, dtype: Type },
    ForwardFunction { id: usize, dtype: Type },

    // Statements
    Block(Vec<AstNode>),
    If { cond: Box<AstNode>, true_branch: Box<AstNode>, f_branch: Box<AstNode> },
    For { assign: Vec<AstNode>, cond: Box<AstNode>, post_op: Vec<AstNode>, body: Box<AstNode> },
    Print(Vec<AstNode>),
    Return(Box<AstNode>),
    Assignment { lvalue: usize, expr: Box<AstNode> },

    // Expressions
    Minus(Box<AstNode>),
    Not(Box<AstNode>),
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

pub struct Parser<I>
    where I: Iterator<Item=char>
{
    scanner: Scanner<I>,
}

impl<I> Parser<I>
    where I: Iterator<Item=char>
{
    fn new(scanner: Scanner<I>) -> Self {
        Parser { scanner }
    }

    fn scan_function(&mut self, id: usize) -> AstNode {
        todo!()
    }

    fn scan_var_decl(&mut self) -> Type {
        match self.scanner.scan() {
            Some(Token { kind: TokenKind::Array, .. }) => {
                self.scanner.must_be('[');
                self.scanner.must_be(']');

//                self.scan_var_decl(
            }
            Some(Token { kind: TokenKind::Integer, pos }) => {},
        }

        todo!()
    }

    fn declaration(&mut self) -> Option<AstNode> {
        let ident = match self.scanner.scan() {
            Some(Token { kind: TokenKind::Ident(id), .. }) => id,
            Some(Token { pos, .. }) => { fatal("Expected an identifier", pos) }
            None => return None,
        };

        self.scanner.must_be(':');

        Some(match self.scanner.scan() {
            Some(Token { kind: TokenKind::Function, .. }) => self.scan_function(ident),
            Some(t) if t.is_type() => {
                self.scanner.put_token(t);
                let dtype = self.scan_var_decl();
                AstNode::VarDecl { id, dtype }
            },
            Some(Token { pos, .. }) => fatal("Expected 'function' or a type", pos),
            None => panic!("Found EOF parsing a declaration"),
        })
    }

    pub fn parse_top(&mut self) -> Vec<AstNode> {
        std::iter::from_fn(|| self.declaration()).collect()
    }
}
