rust-query
=======

A querystring parsing library for Rust inspired by https://github.com/hapijs/qs. A part of REST-like API micro-framework [Raisin].

[Raisin]: https://github.com/s-panferov/raisin

## Usage

Use **rust-query** library to parse query-string to corresponding [json] values.

```rust
use query::parse;

// will contain result as Json value
let object = parse("foo[0][a]=a&foo[0][b]=b&foo[1][a]=aa&foo[1][b]=bb");
```

You can work with raw Json value or decode it to Rust strucure by using serialize::json::decode function (see examples in [json]).

[json]: http://doc.rust-lang.org/serialize/json/index.html

## Description

**rust-query** allows you to create nested objects within your query strings, by surrounding the name of sub-keys with square brackets `[]`. For example, the string `'foo[bar]=baz'` converts to this JSON:

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

**rust-query** can also parse arrays using a similar `[]` notation:

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

**rust-query** **does't** allow to specify sparse indexes on arrays and will convert target array to object:

```javascript
parse('a[1]=b&a[15]=c');
// { "a": {"1":"b", "15":"c"} }
```

Also if you mix notations, **rust-query** will merge the two items into an object:

```javascript
parse('a[0]=b&a[b]=c');
// { "a": { "0": "b", "b": "c" } }
```

You can also create arrays of objects:

```javascript
parse('a[][b]=c');
// { "a": [{ "b": "c" }] }
```
