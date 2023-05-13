use serde_json::{Value, to_value};
use crate::helpers::{
    object_from_list,
    next_index,
    create_array,
    push_item_to_array
};

fn merge_object_and_array(to: &mut Value, from: &Value) -> Option<Value> {
    let tree = to.as_object_mut().unwrap();
    let vec = from.as_array().unwrap();
    let index = next_index(tree);

    for (idx, item) in vec.iter().enumerate() {
        tree.insert((index + idx).to_string(), item.clone());
    }

    None
}

fn merge_object_and_merger(to: &mut Value, from: &Value) -> Option<Value> {
    let to_tree = to.as_object_mut().unwrap();
    let from_tree = from.as_object().unwrap();

    let to_index = from_tree.get(&"__idx".to_string()).unwrap().as_u64().unwrap().to_string();
    let source_obj = from_tree.get(&"__object".to_string()).unwrap();
    let has_dest_obj = {
        match to_tree.get(&to_index) {
            Some(&Value::Object(_)) => true,
            Some(&Value::Array(_)) => true,
            Some(_) => false,
            None => false
        }
    };

    if has_dest_obj {
        let merge_result = merge(to_tree.get_mut(&to_index).unwrap(), source_obj);
        match merge_result {
            Some(result) => { to_tree.insert(to_index, result); },
            None => ()
        }
    } else {
        to_tree.insert(to_index, source_obj.clone());
    }

    None
}

fn merge_object_and_object(to: &mut Value, from: &Value) -> Option<Value> {
    let to_tree = to.as_object_mut().unwrap();
    let from_tree = from.as_object().unwrap();

    for (key, value) in from_tree.iter() {
        let has_dest_obj = {
            match to_tree.get(key) {
                Some(_) => true,
                None => false
            }
        };

        if has_dest_obj {
            let merge_result = merge(to_tree.get_mut(key).unwrap(), value);
            match merge_result {
                Some(result) => { to_tree.insert(key.to_string(), result); },
                None => ()
            }
        } else {
            let value = if is_merger(value) {
                let mut list = create_array();
                merge(&mut list, value).unwrap_or(list)
            } else {
                value.clone()
            };
            to_tree.insert(key.to_string(), value);
        }

    }

    None
}

fn merge_list_and_list(to: &mut Value, from: &Value) -> Option<Value> {
    let to_vec = to.as_array_mut().unwrap();
    let from_vec = from.as_array().unwrap();

    for value in from_vec.iter() {
        to_vec.push(value.clone());
    }

    None
}

fn merge_list_and_merger(to: &mut Value, from: &Value) -> Option<Value> {

    let to_vec = to.as_array_mut().unwrap();

    let from_tree = from.as_object().unwrap();
    let to_index = from_tree.get(&"__idx".to_string()).unwrap().as_u64().unwrap() as usize;
    let source_obj = from_tree.get(&"__object".to_string()).unwrap();

    if to_index < to_vec.len() {
        // merge existing item
        let merge_result = merge(&mut to_vec[to_index], source_obj);
        match merge_result {
            Some(result) => {
                to_vec.remove(to_index);
                to_vec.insert(to_index, result);
            },
            None => ()
        }
        None
    } else if to_index == to_vec.len() {
        to_vec.insert(to_index, source_obj.clone());
        None
    } else {
        let mut new_obj = object_from_list(&to_value(to_vec).expect("query string list merging failed"));
        merge_object_and_merger(&mut new_obj, from);
        return Some(new_obj);
    }
}

fn is_merger(obj: &Value) -> bool {
    if !obj.is_object() {
        return false;
    }

    let tree = obj.as_object().unwrap();
    let idx = tree.get(&"__idx".to_string());
    match idx {
        Some(idx) => idx.is_number(),
        None => false
    }
}

fn merge_list_and_object(to: &mut Value, from: &Value) -> Option<Value> {
    let mut to_tree = object_from_list(to);
    merge(&mut to_tree, from);
    Some(to_tree)
}

fn merge_string_and_json(to: &mut Value, from: &Value) -> Option<Value> {
    let mut list = create_array();
    push_item_to_array(&mut list, to.clone());
    push_item_to_array(&mut list, from.clone());

    Some(list)
}

fn merge_list_and_string(to: &mut Value, from: &Value) -> Option<Value> {
    push_item_to_array(to, from.clone());
    None
}

pub fn merge(to: &mut Value, from: &Value) -> Option<Value> {

    match to {
        &mut Value::Object(_) => {
            match from {
                &Value::Null => None,
                &Value::Array(_) => merge_object_and_array(to, from),
                &Value::Object(_) if is_merger(from) => merge_object_and_merger(to, from),
                &Value::Object(_) => merge_object_and_object(to, from),
                &Value::String(_) => Some(from.clone()),
                _ => panic!("Unknown merge")
            }
        }
        &mut Value::Array(_) => {
            match from {
                &Value::Null => None,
                &Value::Array(_) => merge_list_and_list(to, from),
                &Value::Object(_) if is_merger(from) => merge_list_and_merger(to, from),
                &Value::Object(_) => merge_list_and_object(to, from),
                &Value::String(_) => merge_list_and_string(to, from),
                _ => panic!("Unknown merge")
            }
        },
        &mut Value::String(_) => {
            merge_string_and_json(to, from)
        }
        _ => panic!("Unknown merge")
    }

}
