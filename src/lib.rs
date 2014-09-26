#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate collections;

use regex::Regex;
use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json};
use serialize::json::ToJson;

use merge::merge;
use mutable_json::MutableJson;

static parent_regexp: Regex = regex!(r"^([^][]+)");
static child_regexp: Regex = regex!(r"(\[[^][]*\])");

mod merge;
mod mutable_json;

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

	if keys.len() > 0 {
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



fn parse(params: &str) -> Json {
	let tree: TreeMap<String,Json> = TreeMap::new();
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