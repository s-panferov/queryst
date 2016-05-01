use regex::Regex;
use std::collections::BTreeMap;
use serde_json::{Value};
use url::percent_encoding::percent_decode;

use merge::merge;
use helpers::{create_array, push_item_to_array};

lazy_static! {
    static ref PARENT_REGEX: Regex = Regex::new(r"^([^][]+)").unwrap();
    static ref CHILD_REGEX: Regex = Regex::new(r"(\[[^][]*\])").unwrap();
}

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub enum ParseErrorKind {
    DecodingError,
    Other
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String
}

pub type ParseResult<T> = Result<T,ParseError>;

pub fn decode_component(source: &str) -> Result<String,String> {
    let result = percent_decode(source.as_bytes()).decode_utf8_lossy().to_string();
    return Ok(result);
}

fn parse_pairs(body: &str) -> Vec<(&str, Option<&str>)> {

    let mut pairs = vec![];

    for part in body.split("&") {
        let separator = part.find("]=")
                            .and_then(|pos| Some(pos+1))
                            .or_else(|| part.find("="));

        match separator {
            None => pairs.push((part, None)),
            Some(pos) => {
                let key = &part[..pos];
                let val = &part[(pos + 1)..];
                pairs.push((key, Some(val)));
            }
        }
    }

    return pairs
}

fn parse_key(key: &str) -> ParseResult<Vec<String>> {
    let mut keys: Vec<String> = vec![];

    match PARENT_REGEX.captures(key) {
        Some(captures) => {
            match decode_component(captures.at(1).unwrap()) {
                Ok(decoded_key) => keys.push(decoded_key),
                Err(err_msg) => return Err(ParseError{ kind: ParseErrorKind::DecodingError, message: err_msg })
            }
        }
        None => ()
    };

    for captures in CHILD_REGEX.captures_iter(key) {
        match decode_component(captures.at(1).unwrap()) {
            Ok(decoded_key) => keys.push(decoded_key),
            Err(err_msg) => return Err(ParseError{ kind: ParseErrorKind::DecodingError, message: err_msg })
        }
    }

    Ok(keys)
}

fn cleanup_key(key: &str) -> &str {
    if key.starts_with("[") && key.ends_with("]") {
        &key[1..(key.len()-1)]
    } else {
        key
    }
}

fn create_idx_merger(idx: u64, obj: Value) -> Value {
    let mut tree: BTreeMap<String,Value> = BTreeMap::new();
    tree.insert("__idx".to_string(), Value::U64(idx));
    tree.insert("__object".to_string(), obj);
    return Value::Object(tree)
}

fn create_object_with_key(key: String, obj: Value) -> Value {
    let mut tree: BTreeMap<String,Value> = BTreeMap::new();
    tree.insert(key, obj);
    return Value::Object(tree)
}

fn apply_object(keys: &[String], val: Value) -> Value {

    if keys.len() > 0 {
        let key = keys.get(0).unwrap();
        if key == "[]" {
            let mut new_array = create_array();
            let item = apply_object(&keys[1..], val);
            push_item_to_array(&mut new_array, item);
            return new_array;
        } else {
            let key = cleanup_key(key);
            let array_index = key.parse();

            match array_index {
                Ok(idx) => {
                    let result = apply_object(&keys[1..], val);
                    let item = create_idx_merger(idx, result);
                    return item;
                },
                Err(_) => {
                    return create_object_with_key(key.to_string(), apply_object(&keys[1..], val));
                }
            }
        }

    } else {
        return val;
    }
}

pub fn parse(params: &str) -> ParseResult<Value> {
    let tree: BTreeMap<String,Value> = BTreeMap::new();
    let mut obj = Value::Object(tree);
    let pairs = parse_pairs(params);
    for &(key, value) in pairs.iter() {
        let parse_key_res = try!(parse_key(key));
        let key_chain = &parse_key_res[0..];
        let decoded_value = match decode_component(value.unwrap_or("")) {
            Ok(val) => val,
            Err(err) => return Err(ParseError{ kind: ParseErrorKind::DecodingError, message: err })
        };
        let partial = apply_object(key_chain, Value::String(decoded_value));
        merge(&mut obj, &partial);
    }

    Ok(obj)
}

#[cfg(test)]
mod tests {
    use parse;
    use serde_json::{Value, to_string};

    fn eq_str(value: Value, string: &str) {
        assert_eq!(&to_string(&value).unwrap(), string)
    }

    #[test]
    fn it_parses_simple_string() {
        eq_str(parse("0=foo").unwrap(), r#"{"0":"foo"}"#);
        eq_str(parse("a[<=>]==23").unwrap(), r#"{"a":{"<=>":"=23"}}"#);
        eq_str(parse(" foo = bar = baz ").unwrap(), r#"{" foo ":" bar = baz "}"#);
    }

    #[test]
    fn it_parses_nested_string() {
        eq_str(parse("a[b][c][d][e][f][g][h]=i").unwrap(),
            r#"{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":"i"}}}}}}}}"#);
    }

    #[test]
    fn it_parses_simple_array() {
        eq_str(parse("a=b&a=c&a=d&a=e").unwrap(),
            r#"{"a":["b","c","d","e"]}"#);
    }

    #[test]
    fn it_parses_explicit_array() {
        eq_str(parse("a[]=b&a[]=c&a[]=d").unwrap(),
            r#"{"a":["b","c","d"]}"#);
    }

    #[test]
    fn it_parses_nested_array() {
        eq_str(parse("a[b][]=c&a[b][]=d").unwrap(),
            r#"{"a":{"b":["c","d"]}}"#);
    }

    #[test]
    fn it_allows_to_specify_array_indexes() {
        eq_str(parse("a[0][]=c&a[1][]=d").unwrap(),
            r#"{"a":[["c"],["d"]]}"#);
    }

    #[test]
    fn it_transforms_arrays_to_object() {
        eq_str(parse("foo[0]=bar&foo[bad]=baz").unwrap(),
            r#"{"foo":{"0":"bar","bad":"baz"}}"#);

        eq_str(parse("foo[0][a]=a&foo[0][b]=b&foo[1][a]=aa&foo[1][b]=bb").unwrap(),
            r#"{"foo":[{"a":"a","b":"b"},{"a":"aa","b":"bb"}]}"#);
    }

    #[test]
    fn it_doesnt_produce_empty_keys() {
        eq_str(parse("_r=1&").unwrap(),
            r#"{"_r":"1"}"#);
    }

    #[test]
    fn it_supports_encoded_strings() {
        eq_str(parse("a[b%20c]=c%20d").unwrap(),
            r#"{"a":{"b c":"c d"}}"#);
    }
}

