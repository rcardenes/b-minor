use std::collections::HashMap;

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
