use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;

use mutable_json::MutableJson;

fn object_from_list(obj: &Json) -> Json {
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

fn next_index(obj: &JsonObject) -> uint {
	match index(obj) {
		Some(idx) => idx + 1,
		None => 0
	}
}

fn merge_object_and_array(to: &mut Json, from: &Json) -> Option<Json> {
	let tree = to.as_object_mut().unwrap();
	let vec = from.as_list().unwrap();
	let index = next_index(tree);

	for (idx, item) in vec.iter().enumerate() {
		tree.insert((index + idx).to_string(), item.clone());
	}

	None
}

fn merge_object_and_merger(to: &mut Json, from: &Json) -> Option<Json> {
	
	let to_tree = to.as_object_mut().unwrap();
	let from_tree = from.as_object().unwrap();

	let to_index = from_tree.find(&"__idx".to_string()).unwrap().as_u64().unwrap().to_string();
	let source_obj = from_tree.find(&"__object".to_string()).unwrap();
	let has_dest_obj = {
		match to_tree.find(&to_index) {
			Some(&json::Object(_)) => true,
			Some(&json::List(_)) => true,
			Some(_) => false,
			None => false
		}
	};

	if has_dest_obj {
		let merge_result = merge(to_tree.find_mut(&to_index).unwrap(), source_obj);
		match merge_result {
			Some(result) => { to_tree.insert(to_index, result); },
			None => ()
		}
	} else {
		to_tree.insert(to_index, source_obj.clone());
	}

	None
}

fn merge_object_and_object(to: &mut Json, from: &Json) -> Option<Json> {
	let to_tree = to.as_object_mut().unwrap();
	let from_tree = from.as_object().unwrap();

	for (key, value) in from_tree.iter() {
		let has_dest_obj = {
			match to_tree.find(key) {
				Some(&json::Object(_)) => true,
				Some(&json::List(_)) => true,
				Some(_) => false,
				None => false
			}
		};

		if has_dest_obj {
			let merge_result = merge(to_tree.find_mut(key).unwrap(), value);
			match merge_result {
				Some(result) => { to_tree.insert(key.to_string(), result); },
				None => ()
			}
		} else {
			to_tree.insert(key.to_string(), value.clone());
		}
	    
	}

	None
}

fn merge_list_and_list(to: &mut Json, from: &Json) -> Option<Json> {
	let to_vec = to.as_list_mut().unwrap();
	let from_vec = from.as_list().unwrap();

	for value in from_vec.iter() {
		to_vec.push(value.clone());
	}

	None
}

fn merge_list_and_merger(to: &mut Json, from: &Json) -> Option<Json> {
	
	let to_vec = to.as_list_mut().unwrap();

	let from_tree = from.as_object().unwrap();
	let to_index = from_tree.find(&"__idx".to_string()).unwrap().as_u64().unwrap() as uint;
	let source_obj = from_tree.find(&"__object".to_string()).unwrap();

	if to_index < to_vec.len() {
		// merge existing item
		let merge_result = merge(to_vec.get_mut(to_index), source_obj);
		match merge_result {
			Some(result) => { 
				to_vec.remove(to_index);
				println!("Insert {}", result);
				to_vec.insert(to_index, result);
			},
			None => ()
		}
		None
	} else if to_index == to_vec.len() {
		to_vec.insert(to_index, source_obj.clone());
		None
	} else {
		let mut new_obj = object_from_list(&to_vec.to_json());
		merge_object_and_merger(&mut new_obj, from);
		return Some(new_obj);
	}
}

fn is_merger(obj: &Json) -> bool {
	let tree = obj.as_object().unwrap();
	let idx = tree.find(&"__idx".to_string());
	match idx {
		Some(idx) => idx.is_number(),
		None => false
	}
}

fn merge_list_and_object(to: &mut Json, from: &Json) -> Option<Json> {
	let mut to_tree = object_from_list(to);

	merge(&mut to_tree, from);
	Some(to_tree)
}


pub fn merge(target: &mut Json, source: &Json) -> Option<Json> {

	println!("Merge {} with {}", target.to_pretty_str(), source.to_pretty_str());

	match target {
		&json::Object(_) => {
			match source {
				&json::List(_) => merge_object_and_array(target, source),
				&json::Object(_) if is_merger(source) => merge_object_and_merger(target, source),
				&json::Object(_) => merge_object_and_object(target, source),
				&json::String(_) => return Some(source.clone()),
				_ => fail!("Unknown merge")
			}
		}
		&json::List(_) => {
			match source {
				&json::List(_) => merge_list_and_list(target, source),
				&json::Object(_) if is_merger(source) => merge_list_and_merger(target, source),
				&json::Object(_) => merge_list_and_object(target, source),
				&json::String(_) => return Some(source.clone()),
				_ => fail!("Unknown merge")
			}
		},
		&json::String(_) => {
			return Some(source.clone())
		}
		_ => fail!("Unknown merge")
	}

} 