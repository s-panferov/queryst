use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;
use collections::treemap::TreeMap;

use mutable_json::MutableJson;

pub fn object_from_list(obj: &Json) -> Json {
    let list = obj.as_list().unwrap();
    let mut tree: TreeMap<String,Json> = TreeMap::new();

    for (idx, item) in list.iter().enumerate() {
        tree.insert(idx.to_string(), item.clone());
    }

    tree.to_json()
}

fn index(obj: &JsonObject) -> Option<uint> {
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

pub fn next_index(obj: &JsonObject) -> uint {
    match index(obj) {
        Some(idx) => idx + 1,
        None => 0
    }
}

pub fn create_array() -> Json {
    let vec: Vec<Json> = vec![];
    return json::List(vec);
}

pub fn push_item_to_array(array: &mut Json, item: Json) {
    let vec = array.as_list_mut().unwrap();
    vec.push(item);
}