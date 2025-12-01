use petgraph::graph::NodeIndex;
use std::collections::HashMap;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Index {
    full: HashMap<String, NodeIndex>,
    short: HashMap<String, NodeIndex>,
}

impl Index {
    pub(crate) fn insert(&mut self, full_path: String, short_path: String, value: NodeIndex) {
        self.full.insert(full_path, value);
        self.short.entry(short_path).or_insert(value);
    }

    #[inline]
    pub(crate) fn full(&self, full_path: &str) -> Option<&NodeIndex> {
        self.full.get(full_path)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&NodeIndex> {
        if key.contains('/') {
            self.full(key)
        } else {
            self.short.get(key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    #[cfg(feature = "petgraph")]
    fn insert_and_get() {
        let mut index = Index::default();
        index.insert("123/123".to_string(), "123".to_string(), NodeIndex::new(3));

        assert_eq!(index.get("123"), Some(&NodeIndex::new(3)));
        assert_eq!(index.get("123/123"), Some(&NodeIndex::new(3)));
    }

    #[cfg_attr(feature = "tracing", tracing_test::traced_test)]
    #[test]
    #[cfg(feature = "petgraph")]
    fn full() {
        let mut index = Index::default();
        index.insert("123/123".to_string(), "123".to_string(), NodeIndex::new(3));

        assert_eq!(index.full("123/123"), Some(&NodeIndex::new(3)));
        assert_eq!(index.full("123"), None);
        assert_eq!(index.get("123"), Some(&NodeIndex::new(3)));
    }
}
