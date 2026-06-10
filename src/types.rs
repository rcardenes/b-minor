use crate::ast::AstNode;

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
    Array { size: usize, dtype: Box<Type> },
    Function { dtype: Box<Type>, params: Vec<AstNode> },
}

impl Type {
    pub fn make_array(size: usize, dtype: Type) -> Type {
        let dtype = Box::new(dtype);
        Type::Array { size, dtype }
    }

    pub fn make_signature(dtype: Type, params: Vec<AstNode>) -> Type {
        let dtype = Box::new(dtype);
        Type::Function { dtype, params }
    }
}
