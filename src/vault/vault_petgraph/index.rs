use ahash::AHashMap;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

#[derive(Default)]
pub struct Index {
    pub(crate) full: HashMap<String, NodeIndex>,
    pub(crate) short: AHashMap<String, NodeIndex>,
}

impl Index {
    pub(crate) fn insert(&mut self, full: String, short: String, value: NodeIndex) {
        self.full.insert(full, value);
        self.short.entry(short).or_insert(value);
    }

    pub(crate) fn get(&self, key: &str) -> Option<&NodeIndex> {
        if key.contains('/') {
            self.full.get(key)
        } else {
            self.short.get(key)
        }
    }
}
