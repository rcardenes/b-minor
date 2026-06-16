use std::{
    collections::HashMap,
    iter::Iterator,
};

use crate::{
    ast::{AstNode, ExprKind, NodeId},
    sym::{Strings, Symbol, SymbolKind, SymbolTable},
    syn::Parser,
    types::{
        Type, PrimType, Param,
        SCALAR_INT, SCALAR_BOOL, SCALAR_VOID,
    },
};

#[derive(Debug, PartialEq)]
pub enum SemanticError {
    NotEnoughArguments,
    TooManyArguments,
    UnmatchingType,
    NotAFunction,
    UndefinedSymbol,
    WrongOperandType,
    Redeclaration,
    WrongInitializerSize,
    InvalidCondition,
    IllegalReturnType,
    NonLiteralInitialization,
}

pub struct Semantic {
    errors: Vec<SemanticError>,
    table: SymbolTable,
    types: HashMap<NodeId, Type>,
    strings: Strings,
}

impl<I> From<Parser<I>> for Semantic
    where I: Iterator<Item = char>
{
    fn from(value: Parser<I>) -> Self {
        Semantic {
            errors: vec![],
            table: SymbolTable::default(),
            types: HashMap::new(),
            strings: value.into_strings(),
        }
    }
}

impl Semantic {
    pub fn get_num_errors(&self) -> usize {
        self.errors.len()
    }

    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    fn emit_error(&mut self, kind: SemanticError, message: String) {
        self.errors.push(kind);
        eprintln!("{}", message)
    }

    fn resolve_func_call(&mut self, name: usize, args: &[AstNode]) -> Type {
        let func_name = self.strings.get(name).unwrap();
        let nargs = args.len();
        match self.table.lookup(name) {
            Some(Symbol { dtype: Type::Function { dtype, params }, .. }) => {
                let dtype = dtype.as_ref().clone();
                let nparams = params.len();


                if nparams > nargs {
                    self.emit_error(SemanticError::NotEnoughArguments, format!("Too few arguments for {func_name}"));
                } else if nparams < nargs {
                    self.emit_error(SemanticError::TooManyArguments, format!("Too many arguments for {func_name}"));
                } else {
                    let params = params.clone();
                    for (arg, param) in args.iter().zip(params.iter()) {
                        let arg_type = self.resolve_expr(arg);
                        if param.dtype != arg_type {
                            let param_name = self.strings.get(param.name).unwrap();
                            self.emit_error(SemanticError::UnmatchingType, format!("Illegal {arg_type} being passed to {param_name} of type {}", param.dtype));
                        }
                    }
                }
                dtype
            },
            Some(Symbol { dtype, ..}) => {
                let dtype = dtype.clone();
                eprintln!("Not a function: {dtype:?}");
                self.emit_error(SemanticError::NotAFunction, format!("Not a function '{func_name}'"));
                dtype
            },
            None => {
                self.emit_error(SemanticError::UndefinedSymbol, format!("Undefined symbol {func_name}"));
                Type::Undeclared
            }
        }
    }

    pub fn resolve_expr(&mut self, node: &AstNode) -> Type {
        let AstNode::Expr { id, kind } = node else { unreachable!() };

        let dtype = match kind {
            ExprKind::Ident(name) => {
                let name = *name;
                let var_name = self.strings.get(name).unwrap();
                if let Some(sym) = self.table.lookup(name) {
                    sym.dtype.clone()
                } else {
                    self.emit_error(SemanticError::UndefinedSymbol, format!("Undefined symbol {var_name}"));
                    Type::Undeclared
                }
            },
            ExprKind::BoolVal(_) => SCALAR_BOOL.clone(),
            ExprKind::CharLit(_) => Type::Scalar(PrimType::Char),
            ExprKind::IntLit(_) => SCALAR_INT.clone(),
            ExprKind::StringLit(_) => Type::Scalar(PrimType::String),
            ExprKind::Incr(name)
            | ExprKind::Decr(name) => {
                match self.table.lookup(*name) {
                    Some(s) => {
                        let ret = s.dtype.clone();
                        if ret != SCALAR_INT {
                            self.emit_error(SemanticError::WrongOperandType, String::from("Postfix operators must be applied to int"));
                        }
                        ret
                    },
                    None => {
                        let var_name = self.strings.get(*name).unwrap();
                        self.emit_error(SemanticError::UndefinedSymbol, format!("Undeclared variable '{var_name}'"));
                        // We still need to return some type.
                        // There's no valid variable, so the inferred type will be
                        // the one associated to the operator
                        SCALAR_INT.clone()
                    }
                }

            },
            ExprKind::Unary(unary_op, ast_node) => {
                let dtype = self.resolve_expr(ast_node.as_ref());
                match *unary_op {
                    crate::ast::UnaryOp::Minus if dtype != SCALAR_INT => {
                        self.emit_error(SemanticError::WrongOperandType, String::from("Unary '-' must be followed by an integer expression"));
                    }
                    crate::ast::UnaryOp::Not if dtype != SCALAR_BOOL => {
                        self.emit_error(SemanticError::WrongOperandType, String::from("Unary '!' must be followed by a boolean expression"));
                    },
                    _ => {}
                }
                dtype
            }
            ExprKind::Binary { left, op, right } => {
                let (left, right) = (left.as_ref(), right.as_ref());
                let t_left = self.resolve_expr(left);
                let t_right = self.resolve_expr(right);
                match *op {
                    crate::ast::BinaryOp::And
                    | crate::ast::BinaryOp::Or => {
                        if t_left != SCALAR_BOOL || t_right != SCALAR_BOOL {
                            self.emit_error(SemanticError::WrongOperandType, format!("Operator {op} requires boolean operands"));
                        }
                        SCALAR_BOOL.clone()
                    },
                    crate::ast::BinaryOp::Add
                    | crate::ast::BinaryOp::Sub
                    | crate::ast::BinaryOp::Mul
                    | crate::ast::BinaryOp::Div
                    | crate::ast::BinaryOp::Mod
                    | crate::ast::BinaryOp::Pow => {
                        if t_left != SCALAR_INT || t_right != SCALAR_INT {
                            self.emit_error(SemanticError::WrongOperandType, format!("Operator {op} requires integer operands"));
                        }
                        SCALAR_INT.clone()
                    }
                    crate::ast::BinaryOp::Eql
                    | crate::ast::BinaryOp::Neq
                    | crate::ast::BinaryOp::Lss
                    | crate::ast::BinaryOp::Lte
                    | crate::ast::BinaryOp::Grt
                    | crate::ast::BinaryOp::Gte => {
                        if t_left != t_right {
                            self.emit_error(SemanticError::WrongOperandType, format!("Cannot apply {op} to types {t_left} and {t_right}"));
                        }
                        match t_left {
                            Type::Array { .. } => self.emit_error(SemanticError::WrongOperandType, format!("Cannot apply{op} on arrays")),
                            Type::Function { .. } => self.emit_error(SemanticError::WrongOperandType, format!("Cannot apply{op} on functions")),
                            Type::Scalar(PrimType::Void) => self.emit_error(SemanticError::WrongOperandType, format!("Cannot apply{op} on 'void'")),
                            _ => {}
                        }
                        match t_right {
                            Type::Array { .. } => self.emit_error(SemanticError::WrongOperandType, format!("Cannot apply{op} on arrays")),
                            Type::Function { .. } => self.emit_error(SemanticError::WrongOperandType, format!("Cannot apply{op} on functions")),
                            Type::Scalar(PrimType::Void) => self.emit_error(SemanticError::WrongOperandType, format!("Cannot apply{op} on 'void'")),
                            _ => {}
                        }
                        SCALAR_BOOL.clone()
                    },
                }
            },
            ExprKind::Assignment { lvalue, rvalue } => {
                let lvalue_str = self.strings.get(*lvalue).unwrap().clone();
                let rvalue_type = self.resolve_expr(rvalue);
                if let Some(Symbol { dtype, .. }) = self.table.lookup(*lvalue) {
                    let dtype = dtype.clone();
                    if dtype != rvalue_type {
                        self.emit_error(SemanticError::UnmatchingType, format!("Illegal assignment of {rvalue_type} to '{lvalue_str}' of type {dtype}"))
                    }
                    dtype
                } else {
                    self.emit_error(SemanticError::UndefinedSymbol, format!("Undefined symbol {lvalue_str}"));
                    Type::Undeclared
                }
            }
            ExprKind::FuncCall { name, params } => self.resolve_func_call(*name, params),
            ExprKind::Subscript { name, index } => {
                let var_name = self.strings.get(*name).unwrap().clone();
                match self.table.lookup(*name) {
                    Some(s) => {
                        let mut dtype = s.dtype.clone();
                        let mut index = index.as_slice();

                        while !index.is_empty() {
                            match dtype {
                                Type::Array { dtype: deref_type, .. } => {
                                    let index_type = self.resolve_expr(&index[0]);
                                    if !matches!(index_type, Type::Scalar(PrimType::Int)) {
                                        self.emit_error(SemanticError::WrongOperandType, String::from("Subscript index must be integer"));
                                    }
                                    dtype = deref_type.as_ref().clone();
                                    index = &index[1..];
                                },
                                _ => {
                                    self.emit_error(SemanticError::WrongOperandType, String::from("Dereferencing non-array"));
                                    dtype = Type::Undeclared;
                                    break;
                                }
                            }
                        }

                        dtype
                    },
                    None => {
                        self.emit_error(SemanticError::UndefinedSymbol, format!("Undeclared variable '{var_name}'"));
                        Type::Undeclared
                    }
                }
            },
        };

        self.types.insert(*id, dtype.clone());

        dtype
    }

    pub fn resolve_array_initializer(&mut self, _dtype: &Type, _values: &[AstNode]) -> (usize, Type) {
        todo!()
    }

    pub fn resolve_var_decl(&mut self, node: &AstNode, kind: SymbolKind) {
        let AstNode::VarDecl { name, dtype, init, .. } = node else { unreachable!() };
        let var_name = self.strings.get(*name)
            .expect("My shitty compiler tried to access a variable name that is not there!")
            .clone();

        let symbol = Symbol::create(*name, kind, dtype.clone());

        if self.table.bind(symbol).is_err() {
            self.errors.push(SemanticError::Redeclaration)
        }

        if let Some(node) = init {
            let node = node.as_ref();
            match node {
                AstNode::Expr { kind: expr_kind, .. } => {
                    let init_type = self.resolve_expr(node);
                    // TODO: This is too primitive. We should be able to initialize globals
                    //       with any constant value - at the time of initialization.
                    if kind == SymbolKind::Global {
                        match expr_kind {
                            ExprKind::BoolVal(_)
                            |ExprKind::CharLit(_)
                            |ExprKind::IntLit(_)
                            |ExprKind::StringLit(_) => {},
                            ExprKind::Unary(_, ast_node) => {
                                match ast_node.as_ref() {
                                    AstNode::Expr { kind, .. } if kind.is_literal() => {},
                                    _ => self.emit_error(SemanticError::NonLiteralInitialization, String::from("Global variables can only be initialized with literals")),
                                }
                            },
                            _ => self.emit_error(SemanticError::NonLiteralInitialization, String::from("Global variables can only be initialized with literals")),
                        }
                    }
                    if init_type != *dtype {
                        self.emit_error(SemanticError::UnmatchingType, format!("Initialization value for {var_name} is of type {init_type}, should be {dtype}"));
                    }
                },
                AstNode::ArrayInitializer(values) => {
                    let Type::Array { size, dtype } = dtype else { unreachable!("My shitty compiler didn't catch an array initialization on a scalar at parsing time") };
                    let (init_size, _init_type) = self.resolve_array_initializer(dtype.as_ref(), values);
                    if init_size != *size {
                        self.emit_error(SemanticError::WrongInitializerSize, format!("Initializer for {var_name} has size {init_size}, should be {size}"));
                    }
                },
                _ => unreachable!("Trying to resolve an initializer: {:#?}", node.clone()),
            }
        }

    }

    fn resolve_if(&mut self, ret_type: &Type, cond: &AstNode, t_branch: &AstNode, f_branch: &AstNode) {
        let cond_type = self.resolve_expr(cond);
        if cond_type != SCALAR_BOOL {
            self.emit_error(SemanticError::InvalidCondition, format!("Conditions must evalue to boolean, not {cond_type}"));
        }

        if t_branch != &AstNode::EmptyBlock {
            let AstNode::Block(statements) = t_branch else { unreachable!() };
            self.table.enter_scope();
            self.resolve_block(ret_type, statements);
            self.table.leave_scope();
        }

        if f_branch != &AstNode::EmptyBlock {
            let AstNode::Block(statements) = f_branch else { unreachable!() };
            self.table.enter_scope();
            self.resolve_block(ret_type, statements);
            self.table.leave_scope();
        }
    }

    fn resolve_for(&mut self, ret_type: &Type, assign: &[AstNode], cond: &AstNode, post_op: &[AstNode], body: &AstNode) {
        for expr in assign {
            let _ = self.resolve_expr(expr);
        }

        let cond_type = self.resolve_expr(cond);
        if cond_type != SCALAR_BOOL {
            self.emit_error(SemanticError::InvalidCondition, format!("Conditions must evalue to boolean, not {cond_type}"));
        }

        for expr in post_op {
            let _ = self.resolve_expr(expr);
        }

        if body != &AstNode::EmptyBlock {
            let AstNode::Block(statements) = body else { unreachable!() };
            self.table.enter_scope();
            self.resolve_block(ret_type, statements);
            self.table.leave_scope();
        }
    }

    fn resolve_print(&mut self, expressions: &[AstNode]) {
        for expr in expressions {
            self.resolve_expr(expr);
        }
    }

    fn resolve_block(&mut self, ret_type: &Type, block: &[AstNode]) {
        let mut local_count = 0;
        for statement in block {
            match statement {
                AstNode::Expr { .. } => {
                    // Dropping the value
                    let _ = self.resolve_expr(statement);
                },
                AstNode::VarDecl {..} => {
                    self.resolve_var_decl(statement, SymbolKind::Local(local_count));
                    local_count += 1;
                },
                AstNode::Block(ast_nodes) => {
                    self.table.enter_scope();
                    self.resolve_block(ret_type, ast_nodes);
                    self.table.leave_scope();
                }
                AstNode::If { cond, t_branch, f_branch } => {
                    self.resolve_if(ret_type, cond.as_ref(), t_branch.as_ref(), f_branch.as_ref());
                },
                AstNode::For { assign, cond, post_op, body } => {
                    self.resolve_for(ret_type, assign, cond.as_ref(), post_op, body.as_ref());
                },
                AstNode::Print(ast_nodes) => self.resolve_print(ast_nodes),
                AstNode::VoidReturn => {
                    if *ret_type != SCALAR_VOID {
                        self.emit_error(SemanticError::IllegalReturnType, String::from("Returning value from void function"))
                    }
                },
                AstNode::Return(ast_node) => {
                    let rtp = self.resolve_expr(ast_node);
                    if rtp != *ret_type {
                        self.emit_error(SemanticError::UnmatchingType, format!("Returning {rtp}, should be {ret_type}"))
                    }
                },

                AstNode::ArrayInitializer(_)
                | AstNode::EmptyBlock
                | AstNode::Ident {..}
                | AstNode::ForwardFunction {..}
                | AstNode::Function {..} => unreachable!(),
            }
        }
    }

    pub fn resolve_function_decl(&mut self, node: &AstNode) {
        let (_id, name, signature, body, kind) = match node {
            AstNode::Function { id, name, signature, body } => {
                (id, name, signature, Some(body.as_ref()), SymbolKind::Function)
            },
            AstNode::ForwardFunction { id, name, signature } => {
                (id, name, signature, None, SymbolKind::Forward)
            },
            _ => unreachable!()
        };

        let Type::Function { dtype, params } = signature else { unreachable!() };
        let func_name = self.strings.get(*name).unwrap().clone();
        let ret_type = dtype.as_ref();

        if self.table.bind(Symbol::create(*name, kind, signature.clone())).is_err() {
            self.errors.push(SemanticError::Redeclaration);
        }
 
        if !matches!(ret_type, Type::Scalar(..)) {
            self.emit_error(SemanticError::IllegalReturnType, format!("Non-scalar return type for {func_name}"));
        }

        self.table.enter_scope();
        for (param_count, Param { name, dtype }) in params.iter().enumerate() {
            let _ = self.table.bind(Symbol::create(*name, SymbolKind::Param(param_count), dtype.clone()));
        }
        match body {
            None | Some(AstNode::EmptyBlock) => {},
            Some(AstNode::Block(block)) => {
                self.resolve_block(ret_type, block);
            },
            _ => unreachable!()
        }
        self.table.leave_scope();

    }

    pub fn resolve(&mut self, tree: &[AstNode]) {
        for decl in tree {
            match decl {
                AstNode::VarDecl {..} => self.resolve_var_decl(decl, SymbolKind::Global),
                AstNode::ForwardFunction {..}
                | AstNode::Function {..} => self.resolve_function_decl(decl),
                _ => unreachable!("Got illegal {decl:?} at the global scope. Parsing has failed")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::Scanner;

    fn analyze(code: &str) -> (Vec<AstNode>, Semantic) {
        let s = String::from(code);
        let mut p = Parser::new(Scanner::new(s.into_chars().peekable()));
        let ast = p.parse_top();
        let sem = Semantic::from(p);
        (ast, sem)
    }

    // ── A: Type Compatibility in Assignments ──

    #[test]
    fn a1_assign_int_expr_to_bool_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean; x = 1 + 2; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn a2_assign_bool_to_int_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer; x = true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn a3_assign_char_to_int_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer; x = 'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn a4_assign_string_to_int_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer; x = \"hello\"; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn a5_assign_int_to_char_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: char; x = 42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn a6_assign_bool_to_char_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: char; x = false; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn a7_assign_int_to_string_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: string; x = 99; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn a8_type_compat_assignments_succeed() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { a: integer = 5; b: integer; c: char = 'x'; d: boolean = true; e: string = \"ok\"; b = a; a = 10; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── B: Type Compatibility in Variable Initialization ──

    #[test]
    fn b1_init_bool_with_int_lit() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = 42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn b2_init_int_with_char_lit() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn b3_init_char_with_bool_lit() {
        let (ast, mut sem) = analyze("main: function integer() = { x: char = true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn b4_init_string_with_int_lit() {
        let (ast, mut sem) = analyze("main: function integer() = { x: string = 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn b5_init_matching_type_succeeds() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { a: integer = 100; b: char = 'z'; c: boolean = false; d: string = \"ok\"; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── C: Type Compatibility in Return Statements ──

    #[test]
    fn c1_return_int_from_void() {
        let (ast, mut sem) = analyze("foo: function void() = { return 42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn c2_return_char_from_int_func() {
        let (ast, mut sem) = analyze("foo: function integer() = { return 'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn c3_return_bool_from_int_func() {
        let (ast, mut sem) = analyze("foo: function integer() = { return true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn c4_return_nothing_from_non_void() {
        // B-Minor does not require a non-void function to return a value
        let (ast, mut sem) = analyze("foo: function integer() = { return; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::IllegalReturnType]);
    }

    #[test]
    fn c5_return_expr_from_void() {
        let (ast, mut sem) = analyze("foo: function void() = { return 1 + 2; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn c6_return_matching_type_succeeds() {
        let (ast, mut sem) = analyze(
            "foo: function integer() = { return 42; } bar: function void() = { return; } baz: function boolean() = { return true; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── D: Type Compatibility in Binary Expressions ──

    #[test]
    fn d1_add_int_and_bool() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 1 + true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn d2_add_bool_and_int() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = false + 1; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn d3_sub_char_from_int() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 'a' - 1; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn d4_mul_string_by_int() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = \"str\" * 2; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn d5_cmp_int_with_bool() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = 1 == true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn d6_cmp_char_with_int() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = 'a' < 100; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn d7_cmp_string_with_string() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = \"abc\" == \"def\"; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn d8_all_integer_arithmetic_succeeds() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { x: integer = 1 + 2 * 3 ^ 4; y: integer = x / 2; z: integer = y % 3; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn d9_all_boolean_logic_succeeds() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { a: boolean = true && false; b: boolean = a || true; c: boolean = !a; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── E: Operator Compatibility ──

    #[test]
    fn e1_unary_minus_on_bool() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = -true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType, SemanticError::UnmatchingType]);
    }

    #[test]
    fn e2_unary_minus_on_char() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = -'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType, SemanticError::UnmatchingType]);
    }

    #[test]
    fn e3_unary_minus_on_string() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = -\"str\"; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType, SemanticError::UnmatchingType]);
    }

    #[test]
    fn e4_logical_not_on_int() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = !42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType, SemanticError::UnmatchingType]);
    }

    #[test]
    fn e5_logical_not_on_char() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = !'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType, SemanticError::UnmatchingType]);
    }

    #[test]
    fn e6_logical_not_on_string() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = !\"str\"; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType, SemanticError::UnmatchingType]);
    }

    #[test]
    fn e7_postinc_on_bool_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = false; x++; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType]);
    }

    #[test]
    fn e8_postinc_on_char_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: char = 'a'; x++; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType]);
    }

    #[test]
    fn e9_postdec_on_bool_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = true; x--; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType]);
    }

    #[test]
    fn e10_postinc_on_int_var_succeeds() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 0; x++; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn e11_subscript_on_int_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 5; y: integer = x[0]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e12_subscript_on_string_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: string = \"hello\"; y: char = x[0]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType]);
    }

    #[test]
    fn e13_subscript_on_bool_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = true; y: integer = x[0]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e14_subscript_on_array_succeeds() {
        let (ast, mut sem) = analyze("main: function integer() = { arr: array [10] integer; x: integer = arr[0]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn e15_subscript_index_must_be_int() {
        let (ast, mut sem) = analyze("main: function integer() = { arr: array [10] integer; x: integer = arr[true]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e16_subscript_with_char_index() {
        let (ast, mut sem) = analyze("main: function integer() = { arr: array [10] integer; x: integer = arr['a']; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e17_nested_array_subscript() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { mat: array [5] array [10] integer; row: array [10] integer = mat[0]; val: integer = mat[0][1]; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── F: Boolean Conditions ──

    #[test]
    fn f1_if_cond_is_int_expr() {
        let (ast, mut sem) = analyze("main: function integer() = { if (1 + 2) { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn f2_if_cond_is_char() {
        let (ast, mut sem) = analyze("main: function integer() = { if ('a') { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn f3_if_cond_is_string() {
        let (ast, mut sem) = analyze("main: function integer() = { if (\"str\") { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn f4_if_cond_is_bool_succeeds() {
        let (ast, mut sem) = analyze("main: function integer() = { if (true) { x: integer = 0; } if (1 < 2) { y: integer = 1; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn f5_for_cond_is_int_expr() {
        let (ast, mut sem) = analyze("main: function integer() = { for (; 1 + 2; ) { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn f6_for_cond_is_string() {
        let (ast, mut sem) = analyze("main: function integer() = { for (; \"loop\"; ) { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn f7_for_cond_is_bool_succeeds() {
        let (ast, mut sem) = analyze("main: function integer() = { for (; true; ) { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── G: Scope-Dependent Initialization ──

    #[test]
    fn g1_global_init_with_binary_expr() {
        let (ast, mut sem) = analyze("x: integer = 1 + 2;\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::NonLiteralInitialization]);
    }

    #[test]
    fn g2_global_init_with_ident() {
        let (ast, mut sem) = analyze("y: integer = 10; x: integer = y;\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::NonLiteralInitialization]);
    }

    #[test]
    fn g3_global_init_with_unary_expr() {
        let (ast, mut sem) = analyze("x: integer = -5;\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn g4_global_init_with_literal_succeeds() {
        let (ast, mut sem) = analyze(
            "x: integer = 42; y: boolean = true; z: char = 'a'; w: string = \"global\";\nmain: function integer() = { return 0; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn g5_block_init_with_expr_succeeds() {
        let (ast, mut sem) = analyze("main: function integer() = { a: integer = 5; b: integer = a + 1; c: integer = b * 2; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn g6_global_init_with_non_literal_unary_expr() {
        let (ast, mut sem) = analyze("y: integer = 5; x: integer = -y;\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::NonLiteralInitialization]);
    }

    // ── H: Variable and Function Resolution ──

    #[test]
    fn h0_basic_function_structure() {
        let (ast, mut sem) = analyze("main: function integer() = { }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn h1_ref_undefined_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = y; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UndefinedSymbol]);
    }

    #[test]
    fn h2_assign_to_undefined_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x = 42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn h3_call_undefined_func() {
        let (ast, mut sem) = analyze("main: function integer() = { foo(42); }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn h4_call_func_wrong_arg_count() {
        let (ast, mut sem) = analyze("foo: function integer(x: integer, y: integer);\nmain: function integer() = { foo(42); }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn h5_call_func_extra_args() {
        let (ast, mut sem) = analyze("foo: function integer(x: integer);\nmain: function integer() = { foo(1, 2); }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn h6_call_func_wrong_arg_type() {
        let (ast, mut sem) = analyze("foo: function integer(x: integer);\nmain: function integer() = { foo(true); }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn h7_valid_func_call_succeeds() {
        let (ast, mut sem) = analyze(
            "foo: function integer(x: integer, y: boolean);\nmain: function integer() = { z: integer = foo(42, true); }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── I: Duplicate Declarations ──

    #[test]
    fn i1_duplicate_var_in_same_scope() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 1; x: boolean = true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::Redeclaration]);
    }

    #[test]
    fn i2_duplicate_func_decl() {
        let (ast, mut sem) = analyze("foo: function integer();\nfoo: function void();\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::Redeclaration]);
    }

    #[test]
    fn i3_var_shadows_outer_scope() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 1; if (true) { x: boolean = true; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![]);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── J: Array Initialization Semantics ──

    #[test]
    #[should_panic]
    fn j1_array_init_wrong_element_type() {
        let (ast, mut sem) = analyze("main: function integer() = { arr: array [3] integer = [1, true, 3]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    #[should_panic]
    fn j2_valid_array_init_succeeds() {
        let (ast, mut sem) = analyze("main: function integer() = { arr: array [4] integer = [1, 2, 3, 4]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    #[should_panic]
    fn j3_nested_array_init_wrong_inner_type() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { mat: array [2] array [2] integer = [[1, true], [3, 4]]; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    // ── K: Error Handling Behavior ──

    #[test]
    fn k1_multiple_errors_all_reported() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { x: integer = true; y: char = 1; z: boolean = \"str\"; if (1) { a: integer = false; } }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![
            SemanticError::UnmatchingType,
            SemanticError::UnmatchingType,
            SemanticError::UnmatchingType,
            SemanticError::InvalidCondition,
            SemanticError::UnmatchingType,
        ]);
    }

    #[test]
    fn k2_unmatching_return_type() {
        let (ast, mut sem) = analyze("foo: function integer() = { return true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UnmatchingType]);
    }

    #[test]
    fn k3_for_cond_must_be_boolean() {
        let (ast, mut sem) = analyze("main: function integer() = { for (; 42; ) { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::InvalidCondition]);
    }

    // ── C: Calling non-function symbol as function ──

    #[test]
    fn c7_call_non_function() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 5; x(); }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::NotAFunction]);
    }

    // ── E: Postfix on undeclared variable ──

    #[test]
    fn e18_postinc_on_undeclared() {
        let (ast, mut sem) = analyze("main: function integer() = { x++; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UndefinedSymbol]);
    }

    #[test]
    fn e19_postdec_on_undeclared() {
        let (ast, mut sem) = analyze("main: function integer() = { x--; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UndefinedSymbol]);
    }

    // ── D: Logical operators on non-boolean operands ──

    #[test]
    fn d10_and_with_non_bool() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = 1 && 2; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType]);
    }

    #[test]
    fn d11_or_with_non_bool() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = 'a' || 'b'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::WrongOperandType]);
    }

    #[test]
    fn d12_compare_arrays() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { arr1: array [5] integer; arr2: array [5] integer; x: boolean = arr1 == arr2; }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![
            SemanticError::WrongOperandType,
            SemanticError::WrongOperandType,
        ]);
    }

    // ── E: Subscript on undeclared variable ──

    #[test]
    fn e20_subscript_on_undeclared() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = y[0]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UndefinedSymbol]);
    }

    // ── F: If-else with non-empty else branch ──

    #[test]
    fn f8_if_else_both_branches() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { if (true) { x: integer = 1; } else { y: boolean = false; } }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── F: For-loop with init and post expressions ──

    #[test]
    fn f9_for_with_init_and_post() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer; for (x = 0; true; x++) {} }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── H: Print statement ──

    #[test]
    fn h8_print_statement() {
        let (ast, mut sem) = analyze("main: function integer() = { print 42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
    fn h9_print_undeclared() {
        let (ast, mut sem) = analyze("main: function integer() = { print x; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_errors(), vec![SemanticError::UndefinedSymbol]);
    }

    // ── I: Nested block scope management ──

    #[test]
    fn i4_nested_block_shadowing() {
        let (ast, mut sem) = analyze(
            "main: function integer() = { x: integer = 1; { x: boolean = true; } }",
        );
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── M: Non-scalar return type (manual AST construction) ──

    #[test]
    fn m1_function_array_return_type() {
        use std::string::IntoChars;
        let scanner = Scanner::new(String::new().into_chars().peekable());
        let mut sem = Semantic::from(Parser::new(scanner));
        sem.strings.add("foo".to_string());
        let sig = Type::make_signature(
            Type::make_array(5, Type::Scalar(PrimType::Int)),
            vec![],
        );
        let func = AstNode::make_function(0, sig, AstNode::EmptyBlock);
        sem.resolve(&[func]);
        assert_eq!(sem.get_errors(), vec![SemanticError::IllegalReturnType]);
    }
}
