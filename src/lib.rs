#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate collections;

use regex::Regex;
use serialize::Decodable;
use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json, JsonObject, JsonList};
use serialize::json::ToJson;
use std::from_str::FromStr;

static parent_regexp: Regex = regex!(r"^([^][]+)");
static child_regexp: Regex = regex!(r"(\[[^][]*\])");

trait MutableJson {
	fn as_object_mut<'a>(&'a mut self) -> Option<&'a mut JsonObject>;
	fn as_list_mut<'a>(&'a mut self) -> Option<&'a mut JsonList>;
}

impl MutableJson for Json {
	
    /// If the Json value is an Object, returns the associated TreeMap.
    /// Returns None otherwise.
    fn as_object_mut<'a>(&'a mut self) -> Option<&'a mut JsonObject> {
        match self {
            &json::Object(ref mut map) => Some(&mut*map),
            _ => None
        }
    }

    fn as_list_mut<'a>(&'a mut self) -> Option<&'a mut JsonList> {
        match self {
            &json::List(ref mut list) => Some(&mut *list),
            _ => None
        }
    }

}

fn parse_pairs(body: &str) -> Vec<(&str, Option<&str>)> {

	let mut pairs = vec![];

	for part in body.split_str("&") {
		let separator = part.find_str("]=")
							.and_then(|pos| Some(pos+1))
							.or_else(|| part.find_str("="));

		match separator {
			None => pairs.push((part, None)),
			Some(pos) => {
				let key = part.slice_to(pos);
				let val = part.slice_from(pos + 1);
				pairs.push((key, Some(val)));
			}
		}
	}

	return pairs
}

fn parse_key(key: &str) -> Vec<&str> {
	let mut keys = vec![];

	match parent_regexp.captures(key) {
		Some(captures) => keys.push(captures.at(1)),
		None => ()
	};

	for captures in child_regexp.captures_iter(key) {
		keys.push(captures.at(1));
	}

	keys
}

fn cleanup_key(key: &str) -> &str {
	if key.starts_with("[") && key.ends_with("]") {
		key.slice_chars(1, key.len()-1)
	} else {
		key
	}
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

	if (has_index) {
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

fn object_from_list(obj: &Json) -> Json {
	let list = obj.as_list().unwrap();
	let mut tree: TreeMap<String,Json> = TreeMap::new();

	for (idx, item) in list.iter().enumerate() {
		tree.insert(idx.to_string(), item.clone());
	}

	tree.to_json()
}

fn create_array() -> Json {
	let vec: Vec<Json> = vec![];
	return json::List(vec);
}

fn push_item_to_array(array: &mut Json, item: Json) {
	let vec = array.as_list_mut().unwrap();
	vec.push(item);
}

fn create_idx_merger(idx: uint, obj: Json) -> Json {
	let mut tree: TreeMap<String,Json> = TreeMap::new();
	tree.insert("__idx".to_string(), idx.to_json());
	tree.insert("__object".to_string(), obj);
	return json::Object(tree)
}

fn create_object_with_key(key: String, obj: Json) -> Json {
	let mut tree: TreeMap<String,Json> = TreeMap::new();
	tree.insert(key, obj);
	return json::Object(tree)
}

fn apply_object(keys: &[&str], val: Json) -> Json {

	if (keys.len() > 0) {
		let key = *keys.get(0).unwrap();
		if key == "[]" {
			let mut new_array = create_array();
			let item = apply_object(keys.tail(), val);
			push_item_to_array(&mut new_array, item);
			return new_array;
		} else {
			let key = cleanup_key(key);
			let array_index: Option<uint> = from_str(key);

			match array_index {
				Some(idx) => {
					let result = apply_object(keys.tail(), val);
					let item = create_idx_merger(idx, result);
					return item;
				},
				None => {
					return create_object_with_key(key.to_string(), apply_object(keys.tail(), val));
				}
			}
		}

	} else {
		return val;
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

		if (has_dest_obj) {
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

	if (to_index < to_vec.len()) {
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
	} else if (to_index == to_vec.len()) {
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


fn merge(target: &mut Json, source: &Json) -> Option<Json> {

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

fn parse(params: &str) -> Json {
	let mut tree: TreeMap<String,Json> = TreeMap::new();
	let mut obj = tree.to_json();
	let pairs = parse_pairs(params);
	for &(key, value) in pairs.iter() {
		let partial = apply_object(parse_key(key).slice_from(0), value.unwrap_or("").to_string().to_json());
		println!("PARTIAL {}", partial.to_pretty_str());
		merge(&mut obj, &partial);
	}

	obj
}

#[test]
fn it_parse_pairs() {
	assert_eq!(vec![("key", Some("val")), ("val[][][]", Some("1"))], parse_pairs("key=val&val[][][]=1"))
}

#[test]
fn it_parse_keys() {
	assert_eq!(parse_key("0"), vec!["0"]);
	assert_eq!(parse_key("foo"), vec!["foo"]);
	assert_eq!(parse_key("a[]"), vec!["a", "[]"]);
	assert_eq!(parse_key("a[>=]"), vec!["a", "[>=]"]);
	assert_eq!(parse_key("a[<=>]"), vec!["a", "[<=>]"]);
	assert_eq!(parse_key("a[==]"), vec!["a", "[==]"]);
	assert_eq!(parse_key("foo"), vec!["foo"]);
	assert_eq!(parse_key(" foo "), vec![" foo "]);
    assert_eq!(parse_key("a[b]"), vec!["a", "[b]"]);
	assert_eq!(parse_key("a[b][c]"), vec!["a", "[b]", "[c]"]);
	assert_eq!(parse_key("a[b][c][d][e][f][g][h]"), vec!["a", "[b]", "[c]", "[d]", "[e]", "[f]", "[g]", "[h]"]);
	assert_eq!(parse_key("a[12b]"), vec!["a", "[12b]"]);
	assert_eq!(parse_key("he=llo"), vec!["he=llo"]);
	assert_eq!(parse_key("a[b c]"), vec!["a", "[b c]"]);
	assert_eq!(parse_key("a[2]"), vec!["a", "[2]"]);
	assert_eq!(parse_key("a[99999999]"), vec!["a", "[99999999]"]);
	assert_eq!(parse_key("{%:%}"), vec!["{%:%}"]);
	assert_eq!(parse_key("foo"), vec!["foo"]);
	assert_eq!(parse_key("_r"), vec!["_r"]);
	assert_eq!(parse_key("[foo]"), vec!["[foo]"]);
	assert_eq!(parse_key("[]"), vec!["[]"]);
}

#[test]
fn it_builds_object() {
	println!("{}", parse("key[]=1&key[0]=2").to_json().to_pretty_str())

	fail!("{}")
}