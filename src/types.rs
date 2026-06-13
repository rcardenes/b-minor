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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // Scalar types are straightforward, but Array and Function involve nested types

    #[rstest]
    #[case(5, Type::Scalar(PrimType::Bool))]
    #[case(10, Type::Scalar(PrimType::Int))]
    #[case(2, Type::make_array(20, Type::Scalar(PrimType::Char)))]
    fn test_array_type_equality(#[case] size: usize, #[case] dtype: Type) {
        assert_eq!(
            Type::make_array(size, dtype.clone()),
            Type::make_array(size, dtype)
            );
    }
}
