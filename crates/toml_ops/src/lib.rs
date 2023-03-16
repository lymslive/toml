//! Implement Toml pointer following the json path syntax, with type `Option<&toml::Value>`.
//! Overload `/` as path operator to point into a node in toml tree, as well as some other
//! meaningfull operator overload.
//! Such as pipe operator `|` to get primitive value from scalar leaf node,
//! push operator `<<` to overwrite scalar node or push new item to array or table,
//! and push assign operator `<<=` to re-assign to toml node unconditionally.
//! While `/` or operator `<<` may invalidate the pointer, we can use `!` operator
//! or `is_none()` method to test such failed case.
//! 
//! # Expample
//! ```rust
//! use toml_ops::PathOperator;
//! let tv = r#"
//! [host]
//! ip="127.0.0.1"
//! port=8080
//! proto=["tcp", "udp"]
//! "#;
//! let mut v: toml::Value = tv.parse().unwrap(); 
//!
//! let port = v.path() / "host" / "port" | 0;
//! assert_eq!(port, 8080);
//!
//! let node = v.path_mut() / "host" / "port" << 8989;
//! let port = node | 0;
//! assert_eq!(port, 8989);
//!
//! let proto = v.path() / "host" / "proto" / 0 | "";
//! assert_eq!(proto, "tcp");
//!
//! let host = v.path_mut() / "host";
//! let host = host << ("newkey", "newval") << ("morekey", 1234);
//! assert_eq!(host.is_none(), false);
//! assert_eq!(!host, false);
//!
//! let mut proto = v.path_mut() / "host" / "proto";
//! proto = proto << ("json", ) << ["protobuf"];
//! assert_eq!(proto.as_ref().unwrap().as_array().unwrap().len(), 4);
//!
//! proto <<= "default";
//! assert_eq!(proto.as_ref().unwrap().is_str(), true);
//! let proto = v.path() / "host" / "proto" | "";
//! assert_eq!(proto, "default");
//!
//! let invalid = v.path() / "host" / "no-key";
//! assert_eq!(!invalid, true);
//! assert_eq!(invalid.is_none(), true);
//! ```
//!

mod operator;
pub use operator::PathOperator;
pub use operator::TomlPtr;
pub use operator::TomlPtrMut;

