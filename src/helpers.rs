use serialize::json::{Json, Object};
use serialize::json::ToJson;
use collections::BTreeMap;

use mutable_json::MutableJson;

pub fn object_from_list(obj: &Json) -> Json {
    let list = obj.as_array().unwrap();
    let mut tree: BTreeMap<String,Json> = BTreeMap::new();

    for (idx, item) in list.iter().enumerate() {
        tree.insert(idx.to_string(), item.clone());
    }

    tree.to_json()
}

fn index(obj: &Object) -> Option<uint> {
    let mut index = 0;
    let mut has_index = false;
    for key in obj.keys() {
        let num_key: Option<uint> = from_str(key.as_slice());
        match num_key {
            Some(idx) if index <= idx => { 
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

pub fn next_index(obj: &Object) -> uint {
    match index(obj) {
        Some(idx) => idx + 1,
        None => 0
    }
}

pub fn create_array() -> Json {
    let vec: Vec<Json> = vec![];
    return Json::Array(vec);
}

pub fn push_item_to_array(array: &mut Json, item: Json) {
    let vec = array.as_array_mut().unwrap();
    vec.push(item);
}