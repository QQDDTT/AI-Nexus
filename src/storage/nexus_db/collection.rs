use dashmap::DashMap;
use serde_json::Value;

pub struct Collection {
    name: String,
    docs: DashMap<String, Value>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            name,
            docs: DashMap::new(),
        }
    }

    pub fn insert(&self, key: String, value: Value) {
        self.docs.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.docs.get(key).map(|v| v.clone())
    }

    pub fn delete(&self, key: &str) {
        self.docs.remove(key);
    }

    pub fn iter(&self) -> Vec<(String, Value)> {
        self.docs.iter().map(|kv| (kv.key().clone(), kv.value().clone())).collect()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
