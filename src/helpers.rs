use serde_json::{Value};
use std::collections::BTreeMap;

pub type Object = BTreeMap<String, Value>;

pub fn object_from_list(obj: &Value) -> Value {
    let list = obj.as_array().unwrap();
    let mut tree: BTreeMap<String,Value> = BTreeMap::new();

    for (idx, item) in list.iter().enumerate() {
        tree.insert(idx.to_string(), item.clone());
    }

    Value::Object(tree)
}

fn index(obj: &Object) -> Option<usize> {
    let mut index = 0;
    let mut has_index = false;
    for key in obj.keys() {
        let num_key = key[..].parse();
        match num_key {
            Ok(idx) if index <= idx => {
                index = idx;
                has_index = true;
            },
            _ => ()
        }
    }

    if has_index {
        Some(index)
    } else {
        None
    }
}

pub fn next_index(obj: &Object) -> usize {
    match index(obj) {
        Some(idx) => idx + 1,
        None => 0
    }
}

pub fn create_array() -> Value {
    let vec: Vec<Value> = vec![];
    return Value::Array(vec);
}

pub fn push_item_to_array(array: &mut Value, item: Value) {
    let vec = array.as_array_mut().unwrap();
    vec.push(item);
}
