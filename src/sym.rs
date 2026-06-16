use std::collections::HashMap;
use anyhow::{Result, bail};
use crate::types::Type;

pub struct Strings {
    strings: Vec<String>,
    mapping: HashMap<String, usize>,
}

impl Strings {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Strings {
            strings: vec![],
            mapping: HashMap::new(),
        }
    }

    pub fn add(&mut self, st: String) -> usize {
        if let Some(&id) = self.mapping.get(&st) {
            id
        } else {
            let id = self.strings.len();
            self.mapping.insert(st.clone(), id);
            self.strings.push(st);
            id
        }
    }

    pub fn get(&self, id: usize) -> Option<&String> {
        self.strings.get(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolKind {
    Local(usize),   // Local variable and its ordinal position
    Global,
    Param(usize),   // Parameter and its ordinal position
    Forward,
    Function,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub(crate) name: usize,
    pub(crate) kind: SymbolKind,
    pub(crate) dtype: Type,
}

impl Symbol {
    pub fn create(name: usize, kind: SymbolKind, dtype: Type) -> Self {
        Symbol { name, kind, dtype }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolId(usize);

#[derive(Debug, Default)]
struct Scope {
    bindings: HashMap<usize, SymbolId>,
}

impl Scope {
    fn add(&mut self, string_id: usize, sym_id: SymbolId) {
        self.bindings.insert(string_id, sym_id);
    }

    fn contains(&self, string_id: usize) -> bool {
        self.bindings.contains_key(&string_id)
    }

    fn get(&self, string_id: usize) -> Option<&SymbolId> {
        self.bindings.get(&string_id)
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn levels(&self) -> usize {
        self.scopes.len()
    }

    pub fn at_global(&self) -> bool {
        self.levels() == 1
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn leave_scope(&mut self) {
        // We can't leave the global scope!
        assert!(self.levels() > 1);
        self.scopes.pop();
    }

    pub fn bind(&mut self, sym: Symbol) -> Result<()> {
        let sym_id = SymbolId(self.symbols.len());
        let string_id = sym.name;

        if let Some(current) = self.lookup_current(string_id) {
            match (current.kind, sym.kind) {
                (SymbolKind::Forward, SymbolKind::Function) => {}, // Only aceptable redefinition
                _ => bail!("Already defined symbol with string {string_id}")
            }
        }

        self.symbols.push(sym);
        self.scopes.last_mut()
            .unwrap()
            .add(string_id, sym_id);

        Ok(())
    }

    pub fn lookup(&self, id: usize) -> Option<&Symbol> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.get(id))
            .next()
            .and_then(|&id| self.symbols.get(id.0))
    }

    pub fn lookup_current(&self, id: usize) -> Option<&Symbol> {
        self.scopes
            .last()
            .and_then(|scope| scope.get(id))
            .and_then(|&id| self.symbols.get(id.0))
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        // We start with the global scope in place
        Self {
            symbols: vec![],
            scopes: vec![Scope::default()]
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrimType;
    use rstest::rstest;

    // ── Strings tests ──

    #[test]
    fn new_creates_empty() {
        let s = Strings::new();
        assert_eq!(s.strings.len(), 0);
        assert!(s.mapping.is_empty());
    }

    #[test]
    fn add_first_string_returns_zero() {
        let mut s = Strings::new();
        assert_eq!(s.add(String::from("hello")), 0);
    }

    #[test]
    fn add_sequential_strings_increment_ids() {
        let mut s = Strings::new();
        assert_eq!(s.add(String::from("a")), 0);
        assert_eq!(s.add(String::from("b")), 1);
        assert_eq!(s.add(String::from("c")), 2);
    }

    #[test]
    fn add_duplicate_string_returns_same_id() {
        let mut s = Strings::new();
        let id1 = s.add(String::from("dup"));
        let id2 = s.add(String::from("dup"));
        assert_eq!(id1, id2);
    }

    #[rstest]
    #[case("", 0)]
    #[case("hello", 0)]
    #[case("world", 0)]
    fn add_empty_string_works(#[case] input: &str, #[case] expected: usize) {
        let mut s = Strings::new();
        assert_eq!(s.add(String::from(input)), expected);
    }

    #[test]
    fn get_valid_id_returns_some() {
        let mut s = Strings::new();
        s.add(String::from("foo"));
        assert_eq!(s.get(0), Some(&String::from("foo")));
    }

    #[test]
    fn get_invalid_id_returns_none() {
        let s = Strings::new();
        assert_eq!(s.get(0), None);
        assert_eq!(s.get(usize::MAX), None);
    }

    #[test]
    fn get_after_multiple_adds() {
        let mut s = Strings::new();
        s.add(String::from("x"));
        s.add(String::from("y"));
        s.add(String::from("z"));
        assert_eq!(s.get(0), Some(&String::from("x")));
        assert_eq!(s.get(1), Some(&String::from("y")));
        assert_eq!(s.get(2), Some(&String::from("z")));
    }

    #[test]
    fn interning_property_same_id_for_same_string() {
        let mut s = Strings::new();
        let id = s.add(String::from("shared"));
        s.add(String::from("other"));
        let id2 = s.add(String::from("shared"));
        assert_eq!(id, id2);
        assert_eq!(s.get(id), Some(&String::from("shared")));
    }

    // ── SymbolKind tests ──

    #[rstest]
    #[case(SymbolKind::Local(0), SymbolKind::Local(0), true)]
    #[case(SymbolKind::Local(0), SymbolKind::Local(1), false)]
    #[case(SymbolKind::Param(0), SymbolKind::Param(0), true)]
    #[case(SymbolKind::Param(0), SymbolKind::Param(1), false)]
    #[case(SymbolKind::Global, SymbolKind::Global, true)]
    #[case(SymbolKind::Forward, SymbolKind::Forward, true)]
    #[case(SymbolKind::Function, SymbolKind::Function, true)]
    #[case(SymbolKind::Global, SymbolKind::Forward, false)]
    #[case(SymbolKind::Forward, SymbolKind::Function, false)]
    #[case(SymbolKind::Local(0), SymbolKind::Global, false)]
    fn test_symbol_kind_eq(#[case] a: SymbolKind, #[case] b: SymbolKind, #[case] eq: bool) {
        assert_eq!(a == b, eq);
    }

    // ── Symbol tests ──

    #[test]
    fn test_symbol_create() {
        let sym = Symbol::create(5, SymbolKind::Param(2), Type::Scalar(PrimType::Bool));
        assert_eq!(sym.name, 5);
        assert_eq!(sym.kind, SymbolKind::Param(2));
        assert_eq!(sym.dtype, Type::Scalar(PrimType::Bool));
    }

    // ── SymbolTable tests ──

    fn sym(name: usize, kind: SymbolKind) -> Symbol {
        Symbol::create(name, kind, Type::Scalar(PrimType::Int))
    }

    #[test]
    fn test_default_has_global_scope() {
        let st = SymbolTable::default();
        assert_eq!(st.levels(), 1);
        assert!(st.at_global());
    }

    #[test]
    fn test_levels() {
        let mut st = SymbolTable::default();
        assert_eq!(st.levels(), 1);
        st.enter_scope();
        assert_eq!(st.levels(), 2);
        st.enter_scope();
        assert_eq!(st.levels(), 3);
        st.leave_scope();
        assert_eq!(st.levels(), 2);
    }

    #[test]
    fn test_at_global() {
        let mut st = SymbolTable::default();
        assert!(st.at_global());
        st.enter_scope();
        assert!(!st.at_global());
        st.leave_scope();
        assert!(st.at_global());
    }

    #[test]
    fn test_scope_enter_leave() {
        let mut st = SymbolTable::default();
        st.enter_scope();
        st.enter_scope();
        assert_eq!(st.levels(), 3);
        st.leave_scope();
        assert_eq!(st.levels(), 2);
        st.leave_scope();
        assert_eq!(st.levels(), 1);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_leave_global_panics() {
        let mut st = SymbolTable::default();
        st.leave_scope();
    }

    #[test]
    fn test_bind_new_symbol() {
        let mut st = SymbolTable::default();
        assert!(st.bind(sym(0, SymbolKind::Global)).is_ok());
    }

    #[test]
    fn test_bind_duplicate_fails() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Global)).unwrap();
        let result = st.bind(sym(0, SymbolKind::Global));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Already defined"));
    }

    #[test]
    fn test_bind_forward_then_function() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Forward)).unwrap();
        assert!(st.bind(sym(0, SymbolKind::Function)).is_ok());
    }

    #[test]
    fn test_bind_forward_then_forward_fails() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Forward)).unwrap();
        assert!(st.bind(sym(0, SymbolKind::Forward)).is_err());
    }

    #[test]
    fn test_lookup_undefined() {
        let st = SymbolTable::default();
        assert!(st.lookup(0).is_none());
        assert!(st.lookup_current(0).is_none());
    }

    #[test]
    fn test_lookup_current_scope() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Global)).unwrap();
        let found = st.lookup(0);
        assert!(found.is_some());
        assert_eq!(found.unwrap().kind, SymbolKind::Global);
    }

    #[test]
    fn test_lookup_current_innermost() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Global)).unwrap();
        st.enter_scope();
        st.bind(sym(0, SymbolKind::Local(0))).unwrap();
        assert_eq!(st.lookup_current(0).unwrap().kind, SymbolKind::Local(0));
        assert_eq!(st.lookup(0).unwrap().kind, SymbolKind::Local(0));
    }

    #[test]
    fn test_lookup_outer_scope() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Global)).unwrap();
        st.enter_scope();
        assert!(st.lookup(0).is_some());
    }

    #[test]
    fn test_lookup_current_not_in_outer() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Global)).unwrap();
        st.enter_scope();
        assert!(st.lookup_current(0).is_none());
    }

    #[test]
    fn test_bind_in_nested_scope() {
        let mut st = SymbolTable::default();
        st.enter_scope();
        st.bind(sym(5, SymbolKind::Local(0))).unwrap();
        assert!(st.lookup(5).is_some());
        assert!(st.lookup_current(5).is_some());
    }

    #[test]
    fn test_lookup_after_leave_scope() {
        let mut st = SymbolTable::default();
        st.enter_scope();
        st.bind(sym(0, SymbolKind::Local(0))).unwrap();
        st.leave_scope();
        assert!(st.lookup(0).is_none());
    }

    #[test]
    fn test_bind_disjoint_names() {
        let mut st = SymbolTable::default();
        st.bind(sym(0, SymbolKind::Global)).unwrap();
        st.bind(sym(1, SymbolKind::Global)).unwrap();
        assert_eq!(st.lookup(0).unwrap().name, 0);
        assert_eq!(st.lookup(1).unwrap().name, 1);
    }

    // ── Scope tests ──

    #[test]
    fn test_scope_add_contains_get() {
        let mut scope = Scope::default();
        let sid = SymbolId(42);
        assert!(!scope.contains(0));
        assert!(scope.get(0).is_none());
        scope.add(0, sid);
        assert!(scope.contains(0));
        assert!(!scope.contains(1));
        assert_eq!(scope.get(0), Some(&sid));
        assert!(scope.get(1).is_none());
    }
}
