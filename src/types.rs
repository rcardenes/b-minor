// Some Constants for Quick Comparisons
pub static SCALAR_BOOL: Type = Type::Scalar(PrimType::Bool);
pub static SCALAR_CHAR: Type = Type::Scalar(PrimType::Char);
pub static SCALAR_INT: Type = Type::Scalar(PrimType::Int);
pub static SCALAR_STRING: Type = Type::Scalar(PrimType::String);
pub static SCALAR_VOID: Type = Type::Scalar(PrimType::Void);

// Primitive types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimType {
    Bool,
    Char,
    Int,
    String,
    Void,
}

impl std::fmt::Display for PrimType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimType::Bool => write!(f, "boolean"),
            PrimType::Char => write!(f, "char"),
            PrimType::Int => write!(f, "integer"),
            PrimType::String => write!(f, "string"),
            PrimType::Void => write!(f, "void"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub(crate) name: usize,
    pub(crate) dtype: Type,
}

#[derive(Clone, Debug, Eq)]
pub enum Type {
    Undeclared,
    Scalar(PrimType),
    Array { size: usize, dtype: Box<Type> },
    Function { dtype: Box<Type>, params: Vec<Param> },
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Undeclared, _) | (_, Self::Undeclared) => true,
            (Self::Scalar(l0), Self::Scalar(r0)) => l0 == r0,
            (Self::Array { size: l_size, dtype: l_dtype }, Self::Array { size: r_size, dtype: r_dtype }) => l_size == r_size && l_dtype == r_dtype,
            (Self::Function { dtype: l_dtype, params: l_params }, Self::Function { dtype: r_dtype, params: r_params }) => l_dtype == r_dtype && l_params == r_params,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Type {
    pub fn make_array(size: usize, dtype: Type) -> Type {
        let dtype = Box::new(dtype);
        Type::Array { size, dtype }
    }

    pub fn make_signature(dtype: Type, params: Vec<Param>) -> Type {
        let dtype = Box::new(dtype);
        Type::Function { dtype, params }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Undeclared => write!(f, "undeclared"),
            Type::Scalar(prim_type) => prim_type.fmt(f),
            Type::Array {..} => todo!(),
            Type::Function {..} => todo!(),
        }
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

    #[test]
    fn test_scalar_vs_array_not_equal() {
        assert_ne!(
            Type::Scalar(PrimType::Int),
            Type::make_array(5, Type::Scalar(PrimType::Int)),
        );
    }

    #[test]
    fn test_scalar_vs_function_not_equal() {
        assert_ne!(
            Type::Scalar(PrimType::Int),
            Type::make_signature(Type::Scalar(PrimType::Void), vec![]),
        );
    }

    #[test]
    fn test_undeclared_display() {
        assert_eq!(format!("{}", Type::Undeclared), "undeclared");
    }
}
