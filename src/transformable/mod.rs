use std::ops::{Deref, DerefMut};

use evalexpr::Value;

pub mod from_to_value;
pub mod transformation;

#[derive(Clone, PartialEq, Debug)]
pub struct TransformableList {
    list: Vec<(String, Value)>,
    operations: Option<Vec<String>>,
}
impl Deref for TransformableList {
    type Target = Vec<(String, Value)>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}
impl DerefMut for TransformableList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
impl TransformableList {
    pub fn new(operations: Option<Vec<String>>) -> Self {
        TransformableList {
            list: vec![],
            operations,
        }
    }

    fn set(&mut self, key: String, value: Value) {
        let index = self.get_index(&key);
        match index {
            None => self.push((key, value)),
            Some(index) => self.list[index] = (key, value),
        }
    }

    fn get(&self, key: &str) -> Option<&Value> {
        self.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn get_index(&self, key: &str) -> Option<usize> {
        self.iter().position(|(k, _)| k == key)
    }

    fn get_operations(&self) -> Option<Vec<String>> {
        self.operations.clone()
    }
}
