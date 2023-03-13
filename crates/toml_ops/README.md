# Toml Path and Pointer

## Overview

Implement Toml pointer following the json path syntax, with type
`Option<&toml::Value>`, which dependent on crate `toml`.  Overload `/` as path
operator to point into a node in toml tree, as well as some other meaningfull
operator overload.  Such as pipe operator `|` to get primitive value from
scalar leaf node, push operator `<<` to overwrite scalar node or push new item
to array or table, and push assign operator `<<=` to re-assign to toml node
unconditionally.  While `/` or operator `<<` may invalidate the pointer, we
can use `!` operator or `is_none()` method to test such failed case.

### Expample
```rust
use toml_ops::PathOperator;
let tv = r#"
[host]
ip="127.0.0.1"
port=8080
proto=["tcp", "udp"]
"#;
let mut v: toml::Value = tv.parse().unwrap(); 

let port = v.path() / "host" / "port" | 0;
assert_eq!(port, 8080);

let node = v.path_mut() / "host" / "port" << 8989;
let port = node | 0;
assert_eq!(port, 8989);

let proto = v.path() / "host" / "proto" / 0 | "";
assert_eq!(proto, "tcp");

let host = v.path_mut() / "host";
let host = host << ("newkey", "newval") << ("morekey", 1234);
assert_eq!(host.is_none(), false);
assert_eq!(!host, false);

let mut proto = v.path_mut() / "host" / "proto";
proto = proto << ("json", ) << ["protobuf"];
assert_eq!(proto.as_ref().unwrap().as_array().unwrap().len(), 4);

proto <<= "default";
assert_eq!(proto.as_ref().unwrap().is_str(), true);
let proto = v.path() / "host" / "proto" | "";
assert_eq!(proto, "default");

let invalid = v.path() / "host" / "no-key";
assert_eq!(!invalid, true);
assert_eq!(invalid.is_none(), true);
```

## Path Syntax

Some like file path in unix file system, separate each component by slash `/`,
espacially for array index also use `/` not `[]`. So you should not use
numeric key in table to avoid confuse.

Please refer detail or standard json path syntax, as toml is roughly equivalent to
json in their data model.

## Toml Operater Overload Guide

### Operator Trigger

We cannot overload operator form `toml::Value` directly outside the `toml`
crate. So I define a trait named `PathOperator` in this `toml_ops` crate, and
implent it for `toml::Value`. Then you can call the following methods from a
`toml::Value` to create a toml pointer:

* `path()`: point to self toml value.
* `pathto(subpath: &str)`: point to some sub node of self toml value.
* `path_mut()`: mutable version of `path()`.
* `pathto_mut(subpath: &str)`: mutable version of `pathto()`.

The pointer is just a struct wrapper of `Option<&toml::Value>` or
`Option<&mut toml::Value>` for mutable version. But usually no need to care
about this, only use overloaded operators after it.

The mutable version pointer can not implement the `Copy` trait, pay attention
that many operator afterward would consume (move) it.

### Path Operator `/`

It is the core operator for toml path pointer, and it is straightforward to
overload `/` (Div) as path operator purpose.

For example, the following statements is roughtly equivalent:

```rsut
let node = toml_value.path() / "path" / "to" / "node";
let node = toml_value.pathto("path/to/node");
let node = toml_value.path() / "path/to/node";
let node = toml_value.path() / "path.to.node";
```

Note that the later two forms with one string for long path is slightly
inefficient in performance as it involes to split and parse the path syntax,
and it may confuse when there is numeric key in table node(there is subtle bug
only in mutable version now).

It's better to point to array item like this:

```rust
let node = toml_value.path() / "sub-array" / 0 / "sub-key";
// not so good to use:
// let node = toml_value.path() / "sub-array/0/sub-key";
```

While it is impossible to override operator `.`, the dot `.` can also used as
path seperator in the long single path form. But not mix use slash and dot to
confuse youself. For example, `path/../to/node` is not point to parrent as in
file system, it just same as `path/to/node` or `path.to.node`, because path
operator ignore continuous separators.

### Pipe Operator `|`

User pipe operator `|` to get the primitive scalar value from toml node. You
can view `|` as the vertical form of `/`, and read it as "or default", means
that if the node is invalid or mistype, return the default value in the right
hand of `|`.

```rust
let str_val: &str = toml_value().path() / "path" / "to" / "node" | "";
let int_val: i64 = toml_value().path() / "path" / "to" / "node" | 0;
let float_val: f64 = toml_value().path() / "path" / "to" / "node" | 0.0;
let bool_val: bool = toml_value().path() / "path" / "to" / "node" | false;
```

The type notation such as `: i64` is not necessary as it can derived from the
right hand of `|`.

And it is obvious that the `|` would finalize the `/` operator chain as it
return a value of primitve type.

### Put and Push Operator `<<`

It is used in the mutable pointer version, modify the node it refers to.

Put operator `<<` apply to leaf node. Because a toml leaf node can only hold a
scalar value, put operator will overwrite the value it hold, provided that the
value type is match with the origin one.

```rust
let node = toml_vaule.path_mut() / "path" / "to" / "int-node" << 314;
let node = toml_vaule.path_mut() / "path" / "to" / "float-node" << 3.14;
let _ = toml_vaule.path_mut() / "path" / "to" / "string-node" << "PI";
let _ = toml_vaule.path_mut() / "path" / "to" / "bool-node" << true;
```

Put operator consume the left pointer and return a new pointer, and may result
in `None` poiner, if the node is non-existed or mistype. Use `let _` to
depreess warning if not want to use the returned pointer.

Push operator which also overload `<<`, would push new item to array node or
table node.

```rust
let node = toml_vaule.path_mut() / "table" << ("int", 314) << ("float", 3.14);
let _ = toml_vaule.path_mut() / "array" << (314,) << [3.14];
```

As `<<` return also a poiner, the push operator can be chained to append more
items to array or table, provided the origin refered array or table is valid.
You can push any value type the can convert to `toml::Value`.

### Push Assign Operator `<<=`

Because in rust cannot oversion assgin operator `=`, so choose `<<=` instead,
which would has some consistent meanning as previous put or push operator
`<<`. It would re-assign to node unconditionally to any value that can convert
to `toml::Value`.

```rust
let node = toml_vaule.path_mut() / "path" / "to" / "bool-node" << true;
node <<= "true";
```

Note that `<<=` receive reference to self and won't return new pointer, and
should not to chain it as `<<`. Beside that, the following tow statements have
the same effect for leaf node, if `<<` successes:

```rust
let node = node << val; // may fail when mistype
node <<= val; // always sucesse expect node already invalid pointer.
```

### Valid Operator `!`

Overlaod not operator `!` which can be used to test if the pointer is invalid.
In the fowllong cases would return invalid pointer:

* path operator `/` when point to non-existed node.
* put operator `<<` when try to put value of mistype.

Deref operator `*` is also overloaded, support to use pointer implicitly as
`Option<&toml::Value>`. And so `is_none()` method will behave the same as
operator `!`.

```rust
let node = toml_value.path() / "path" / "to" / "nil";
if (!node) { return; }
if (node.is_none()) { return; }
```

However, in mutable verion pointer, the operator `!` would consume the pointer
and so better use `is_none()` method.

### Save Intermediate Pointer

You can save intermediate pointer as you want, for some reasons:

```rust
let root = toml_value.path();
let table = root / "path" / "to" / "table";
let val1 = table / "key1" | 0;
let val2 = table / "key2" | "";
```

While the mutable pointer may have some limitation, follow the compiler
prompt to fix any question.

## Path Operator vs Nest Struct 

When deal with simple toml, it is fairly convenient to deserialize to some
nested struct, and then can use chained dot `.` operator to access the depper
fields.

``rust
let node = toml_value.path.to.node;
``

Where in above, suppose `path` and `to` and `node` are all filed of some
struct in deffirent level, that can map to the toml keys.

However, this easy way can also be broken easily in some practice case:

* when some keys(fields) are not determined in compile time but changed in
  rutime, or just change it's type in runtime.
* when you want to handle optional field in strict way, and then wrap many
  fields in `Option<>`.

Then in these case, it is impossible to keep the simplest `path.to.node` style
any more, and become more tedious than `"path"/"to"/"node"` way.

While in any cases, it can be used path operator in the consistent way, it may
be more straightforward to write business code according to the sample data
structure, and the json path syntax is some what language neutral.

The pay off for this flexibility is typo in coding and may lost IDE completion
tips.

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
    http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
    http://opensource.org/licenses/MIT)

at your option.
