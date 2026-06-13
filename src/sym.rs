use std::collections::HashMap;

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

#[derive(Debug, Clone, Copy)]
pub enum SymbolKind {
    Local(usize),   // Local variable and its ordinal position
    Global,
    Param(usize),   // Parameter and its ordinal position
}

#[derive(Debug, Clone)]
pub struct Symbol {
    id: usize,
    kind: SymbolKind,
    dtype: Type,
}

#[derive(Debug, Clone, Copy)]
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

    pub fn bind(&mut self, sym: Symbol) {
        let sym_id = SymbolId(self.symbols.len());
        let string_id = sym.id;
        self.symbols.push(sym);
        self.scopes.last_mut()
            .unwrap()
            .add(string_id, sym_id);
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
    use rstest::rstest;

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
}
