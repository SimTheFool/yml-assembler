use serde_yaml::Value;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

mod inject;
mod trim;

#[derive(Debug)]
pub struct MixIns(HashMap<String, Vec<Value>>);
impl Deref for MixIns {
    type Target = HashMap<String, Vec<Value>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for MixIns {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MixIns {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn merge(&mut self, other: &Self) {
        other.iter().for_each(|(key, value)| {
            self.add(key.clone(), value.clone());
        });
    }

    pub fn add(&mut self, key: String, value: Vec<Value>) {
        let entry = self.entry(key).or_insert_with(Vec::new);
        entry.append(&mut value.clone());
    }
}
