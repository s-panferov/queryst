## What is Queryst?

**This is a fork of the original, with serde and serde_json updated to 0.9**

A query string parsing library for Rust inspired by https://github.com/hapijs/qs. A part of REST-like API micro-framework [Rustless].

```toml
# Cargo.toml

[dependencies]
queryst = "1"
```

[API docs](http://rustless.org/queryst/doc/queryst)

[Rustless]: https://github.com/rustless/rustless

## Usage

Use **queryst** library to parse query-string to corresponding [json values].

```rust
use queryst::parse;

// will contain result as Json value
let object = parse("foo[0][a]=a&foo[0][b]=b&foo[1][a]=aa&foo[1][b]=bb");
```

[json values]: https://github.com/serde-rs/json

## Description

**queryst** allows you to create nested objects within your query strings,
by surrounding the name of sub-keys with square brackets `[]`.
or example, the string `'foo[bar]=baz'` converts to this JSON:

```json
{
  "foo": {
    "bar": "baz"
  }
}
```

URI encoded strings work too:

```js
parse('a%5Bb%5D=c');
// { "a": { "b": "c" } }
```

You can also nest your objects, like `'foo[bar][baz]=foobarbaz'`:

```javascript
{
  "foo": {
    "bar": {
      "baz": "foobarbaz"
    }
  }
}
```

### Parsing Arrays

**queryst** can also parse arrays using a similar `[]` notation:

```javascript
parse('a[]=b&a[]=c');
// { "a": ["b", "c"] }
```

You may specify an index as well:

```javascript
parse('a[0]=c&a[1]=b');
// { "a": ["c", "b"] }
```

Note that the only difference between an index in an array and a key in an object is that the value between the brackets must be a number to create an array.

**queryst** **does't** allow to specify sparse indexes on arrays and will convert target array to object:

```javascript
parse('a[1]=b&a[15]=c');
// { "a": {"1": "b", "15": "c"} }
```

Also if you mix notations, **queryst** will merge the two items into an object:

```javascript
parse('a[0]=b&a[b]=c');
// { "a": { "0": "b", "b": "c" } }
```

You can also create arrays of objects:

```javascript
parse('a[][b]=c');
// { "a": [{ "b": "c" }] }
```
