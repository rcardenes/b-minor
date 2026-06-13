use std::iter::Iterator;

use crate::{
    ast::{AstNode},
    sym::{Strings, SymbolTable},
    syn::Parser,
};

pub struct Semantic {
    errors: usize,
    table: SymbolTable,
    strings: Strings,
}

impl<I> From<Parser<I>> for Semantic
    where I: Iterator<Item = char>
{
    fn from(value: Parser<I>) -> Self {
        Semantic {
            errors: 0,
            table: SymbolTable::default(),
            strings: value.into_strings(),
        }
    }
}

impl Semantic {
    pub fn get_num_errors(&self) -> usize {
        self.errors
    }

    pub fn resolve_var_decl(&mut self, node: &AstNode, global: bool) {
        todo!()
    }

    pub fn resolve_function_decl(&mut self, node: &AstNode) {
        todo!()
    }

    pub fn resolve(&mut self, tree: &Vec<AstNode>) {
        for decl in tree {
            match decl {
                AstNode::VarDecl {..} => self.resolve_var_decl(decl, true),
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
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn a2_assign_bool_to_int_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer; x = true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn a3_assign_char_to_int_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer; x = 'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn a4_assign_string_to_int_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer; x = \"hello\"; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn a5_assign_int_to_char_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: char; x = 42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn a6_assign_bool_to_char_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: char; x = false; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn a7_assign_int_to_string_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: string; x = 99; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
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
        assert_eq!(sem.get_num_errors(), 0);
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
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e2_unary_minus_on_char() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = -'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e3_unary_minus_on_string() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = -\"str\"; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e4_logical_not_on_int() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = !42; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e5_logical_not_on_char() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = !'a'; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e6_logical_not_on_string() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = !\"str\"; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e7_postinc_on_bool_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = false; x++; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e8_postinc_on_char_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: char = 'a'; x++; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn e9_postdec_on_bool_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: boolean = true; x--; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
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
        assert_eq!(sem.get_num_errors(), 1);
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
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn g2_global_init_with_ident() {
        let (ast, mut sem) = analyze("y: integer = 10; x: integer = y;\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn g3_global_init_with_unary_expr() {
        let (ast, mut sem) = analyze("x: integer = -5;\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
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

    // ── H: Variable and Function Resolution ──

    #[test]
    fn h1_ref_undefined_var() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = y; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
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
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn i2_duplicate_func_decl() {
        let (ast, mut sem) = analyze("foo: function integer();\nfoo: function void();\nmain: function integer() = { return 0; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn i3_var_shadows_outer_scope() {
        let (ast, mut sem) = analyze("main: function integer() = { x: integer = 1; if (true) { x: boolean = true; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    // ── J: Array Initialization Semantics ──

    #[test]
    fn j1_array_init_wrong_element_type() {
        let (ast, mut sem) = analyze("main: function integer() = { arr: array [3] integer = [1, true, 3]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn j2_valid_array_init_succeeds() {
        let (ast, mut sem) = analyze("main: function integer() = { arr: array [4] integer = [1, 2, 3, 4]; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 0);
    }

    #[test]
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
        assert_eq!(sem.get_num_errors(), 4);
    }

    #[test]
    fn k2_semantic_error_in_return_does_not_panic() {
        let (ast, mut sem) = analyze("foo: function integer() = { return true; }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }

    #[test]
    fn k3_semantic_error_in_for_cond_does_not_panic() {
        let (ast, mut sem) = analyze("main: function integer() = { for (; 42; ) { x: integer = 0; } }");
        sem.resolve(&ast);
        assert_eq!(sem.get_num_errors(), 1);
    }
}
