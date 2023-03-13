use super::*;

fn load_test_toml() -> Value
{
    let str_toml = include_str!("../examples/sample.toml");
    let v: Value = str_toml.parse().unwrap();
    return v;
}

#[test]
fn path_test() {
    let v = load_test_toml();
    assert_eq!(path(Some(&v), "ip").unwrap().as_str(), Some("127.0.0.1"));
    assert_eq!(v["ip"].as_str(), Some("127.0.0.1"));

    let op = TomlPtr::path(&v);
    let ip = op / "ip";
    assert_eq!(ip.valop.unwrap().as_str(), Some("127.0.0.1"));

    let ip = op / "host" / "ip";
    assert_eq!(ip.valop.unwrap().as_str(), Some("127.0.1.1"));

    let host = TomlPtr::path(&v) / "host";
    let ip = host / "ip";
    assert_eq!(ip.valop.unwrap().as_str(), Some("127.0.1.1"));
    let port = host / "port";
    assert_eq!(port.valop.unwrap().as_integer(), Some(8080));

    let proto = host / "protocol" / 1;
    assert_eq!(proto.valop.unwrap().as_str(), Some("udp"));

    let proto = v.path() / "host" / "protocol" / 2;
    assert_eq!(proto.unpath().unwrap().as_str(), Some("mmp"));

    let proto = v.path() / "host/protocol/2";
    assert_eq!(proto.unpath().unwrap().as_str(), Some("mmp"));

    let proto = v.pathto("/host/protocol/2");
    assert_eq!(proto.unpath().unwrap().as_str(), Some("mmp"));

    let server = v.path() / "service" / 0 / "name";
    assert_eq!(server.unpath().unwrap().as_str(), Some("serv_1"));

    // path() produce immutable reference, cannot write or assign
    // let server = server.unpath().unwrap();
    // *server = Value::String(String::from("serv 1"));

    let mut mv = load_test_toml();
    let ip = mv.get_mut("ip").unwrap();
    *ip = Value::String(String::from("127.0.0.2"));
    assert_eq!((mv.path() / "ip").unpath().unwrap().as_str(), Some("127.0.0.2"));
}

#[test]
fn path_none_test() {
    let v = load_test_toml();

    let root = v.path();
    assert_eq!(root.unpath().is_none(), false);
    assert_eq!(!root, false);

    let node = root / "ip";
    assert_eq!(node.unpath().is_none(), false);
    assert_eq!(!node, false);
    let node = root / "IP";
    assert_eq!(node.unpath().is_none(), true);
    assert_eq!(!node, true);

    let node = root / "host" /"protocol";
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "host" /"protocol" / 1;
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "host" /"protocol" / 3;
    assert_eq!(node.unpath().is_none(), true);

    let node = root / "service" / 0;
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "service" / 0 / "description";
    assert_eq!(node.unpath().is_none(), true);
    let node = root / "service" / 0 / "desc";
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "service" / 2;
    assert_eq!(node.unpath().is_none(), true);
}


#[test]
fn path_mut_test() {
    let mut v = load_test_toml();

    let root = v.path();
    assert_eq!(root.unpath().is_none(), false);
    assert_eq!(!root, false);

    let node = root / "ip";
    assert_eq!(node.unpath().is_none(), false);
    assert_eq!(!node, false);
    assert_eq!(!!node, true);
    assert_eq!(node.is_none(), false);

    let node = root / "IP";
    assert_eq!(node.unpath().is_none(), true);
    assert_eq!(!node, true);
    assert_eq!((*node).is_none(), true);

    let node = v.path_mut() / "ip";
    assert_eq!(node.unpath().is_none(), false);
    assert_eq!(!node, false);

    let node = v.path_mut() / "IP";
    assert_eq!(node.unpath().is_none(), true);
    assert_eq!(node.is_none(), true);
    assert_eq!(!node, true);
}

#[test]
fn path_build_test() {
    let pseg = "".build_path();
    dbg!(&pseg.paths);
    assert_eq!(pseg.paths.is_empty(), false);
    assert_eq!(pseg.paths, vec![""]);

    let pseg = "/".build_path();
    dbg!(&pseg.paths);
    assert_eq!(pseg.paths.is_empty(), false);
    assert_eq!(pseg.paths, vec!["", ""]);

    let pseg = "//".build_path();
    assert_eq!(pseg.paths, vec!["", "", ""]);

    let pseg = "/path/to/leaf".build_path();
    assert_eq!(pseg.paths, vec!["", "path", "to", "leaf"]);

    let pseg = "path/to/leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf"]);

    let pseg = "path/to//leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "", "leaf"]);

    let pseg = "path.to.leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf"]);

    let pseg = "path/to.leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf"]);

    let pseg = "path/to.leaf/".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf", ""]);

    let path = "34ab";
    let index = path.parse::<usize>();
    assert_eq!(index.is_ok(), false);

    let path = "34";
    let index = path.parse::<usize>();
    assert_eq!(index.is_ok(), true);
    assert_eq!(index, Ok(34));
}

#[test]
fn pipe_test() {
    let v = load_test_toml();

    // pipe ending slash operator to get inner scalar primitive value
    let ip = v.path() / "ip" | "";
    assert_eq!(ip, "127.0.0.1");

    let ip = v.path() / "host" / "ip" | "";
    assert_eq!(ip, "127.0.1.1");

    let ip_default = "127";
    let ip = v.path() / "host" / "ip" | ip_default;
    assert_eq!(ip, "127.0.1.1");

    // pipe convertor accept &'static str or String,
    // but ofcourse &str is more efficient
    let ip_default: String = String::from("127");
    let ip = v.path() / "host" / "ip" | ip_default;
    assert_eq!(ip, "127.0.1.1");
    // ip_default is moved by | operator, and cannot use non-static &String
    // println!("{ip_default}");

    let port = v.path() / "host" / "port" | 0;
    assert_eq!(port, 8080);

    let port_default = 80;
    let port = v.path() / "host" / "port" | port_default;
    assert_eq!(port, 8080);
    assert_eq!(port_default, 80); // simple primitive wont moved

    // can save intermedia tmp value
    let misc = v.path() / "misc";
    let value = misc / "int" | 0;
    assert_eq!(value, 1234);
    let value = misc / "float" | 0.0;
    assert_eq!(value, 3.14);
    let value = misc / "bool" | false;
    assert_eq!(value, true);

    // path ignore repeated slash or dot
    let value = v.pathto("/misc/int") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("misc/int") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("misc/int/") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("misc.int") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("/misc/./int/") | 0;
    assert_eq!(value, 1234);
}

#[test]
fn pipe_mut_test() {
    let mut v = load_test_toml();

    let ip = v.path_mut() / "ip" | "";
    assert_eq!(ip, "127.0.0.1");

    let ip = v.path_mut() / "host" / "ip" | "";
    assert_eq!(ip, "127.0.1.1");

    let ip_default = "127";
    let ip = v.path_mut() / "host" / "ip" | ip_default;
    assert_eq!(ip, "127.0.1.1");

    let ip_default: String = String::from("127");
    let ip = v.path_mut() / "host" / "ip" | ip_default;
    assert_eq!(ip, "127.0.1.1");

    let port = v.path_mut() / "host" / "port" | 0;
    assert_eq!(port, 8080);

    let port_default = 80;
    let port = v.path_mut() / "host" / "port" | port_default;
    assert_eq!(port, 8080);
    assert_eq!(port_default, 80);

    // can save intermedia tmp value
    let misc = v.path_mut() / "misc";
    let value = misc / "int" | 0;
    assert_eq!(value, 1234);

    let value = v.pathto_mut("/misc.int") | 0;
    assert_eq!(value, 1234);

    let value = v.path_mut() / "misc" / "float" | 0.0;
    assert_eq!(value, 3.14);
    let value = v.path_mut() / "misc" / "bool" | false;
    assert_eq!(value, true);
}

#[test]
fn push_test() {
    let mut v = load_test_toml();

    let ip = v.path() / "ip" | "";
    assert_eq!(ip, "127.0.0.1");

    let ip_node = v.path_mut() / "ip" << "127.0.0.2";
    let ip = ip_node | "";
    assert_eq!(ip, "127.0.0.2");
    let ip = v.path() / "ip" | "";
    assert_eq!(ip, "127.0.0.2");

    // push mistype value has no effect.
    let ip_node = v.path_mut() / "ip";
    let ip_node = ip_node << 127;
    assert_eq!(ip_node.is_none(), true);
    let ip = v.path() / "ip" | "";
    assert_eq!(ip, "127.0.0.2");

    // push scalar to leat node with supported type.
    let node = v.path_mut() / "misc" / "int" << 4242;
    let val = node | 0;
    assert_eq!(val, 4242);

    let node = v.path_mut() / "misc" / "float";
    let _ = node << 31.4;
    let val = v.path() / "misc" / "float" | 0.0;
    assert_eq!(val, 31.4);

    let node = v.path_mut() / "misc" / "float";
    let node = node << 3142;
    assert_eq!(node.is_none(), true);
    let val = node | 0.0;
    assert_eq!(val, 0.0);
    let val = v.path() / "misc" / "float" | 0.0;
    assert_eq!(val, 31.4);

    let node = v.path_mut() / "misc" / "bool";
    let _ = node << false;
    let val = v.path() / "misc" / "bool" | true;
    assert_eq!(val, false);

    // push a item to toml array
    let node = v.path_mut() / "host" / "protocol";
    let node = node << ("abc", ) << ["edf"];
    // enable print by: cargo test -- --nocapture
    // dbg!(node.unpath());
    let val = node / 3 | "";
    assert_eq!(val, "abc");
    let val = v.path() / "host" / "protocol" / 4 | "";
    assert_eq!(val, "edf");

    // push slice to toml array
    let node = v.path_mut() / "host" / "protocol";
    let _ = node << &["xyz"][..] << &["ABC", "DEF"][..];
    let node = v.path() / "host" / "protocol";
    println!("{}", node.unpath().unwrap());
    assert_eq!(node.unpath().unwrap().as_array().unwrap().len(), 8);

    // push key-val pair to toml table
    let node = v.path_mut() / "host";
    let _ = node << ("newkey1", 1) << ("newkey2", "2");
    let val = v.path() / "host" / "newkey1" | 0;
    assert_eq!(val, 1);
    let val = v.path() / "host" / "newkey2" | "";
    assert_eq!(val, "2");

    // use i32, must cast to i64, because toml integer save i64 
    let input: i32 = 32;
    let output_default: i32 = 2;
    let node = v.path_mut() / "misc" / "int";
    let _ = node << input as i64;
    let val = v.path() / "misc" / "int" | output_default as i64;
    assert_eq!(val, input as i64);
}

#[test]
fn assign_test() {
    let mut v = load_test_toml();

    let ip = v.path() / "ip" | "";
    assert_eq!(ip, "127.0.0.1");

    let mut ip_node = v.path_mut() / "ip";
    ip_node <<= "127.0.0.2";
    let ip = v.path() / "ip" | "";
    assert_eq!(ip, "127.0.0.2");

    // can not assign to expression.
    // (v.path_mut() / "host" / "ip") <<= "127.0.1.2";

    let ip = v.path() / "host" / "ip" | "";
    assert_eq!(ip, "127.0.1.1");
    let mut ip_node = v.path_mut() / "host" / "ip";
    ip_node <<= 127.0; // type mismatch
    let ip = v.path() / "host" / "ip" | "";
    assert_eq!(ip, "");
    let ip = v.path() / "host" / "ip" | 0.0;
    assert_eq!(ip, 127.0);
    let mut ip_node = v.path_mut() / "host" / "ip";
    ip_node <<= "127.0.1.2";
    let ip = v.path() / "host" / "ip" | "";
    assert_eq!(ip, "127.0.1.2");

    // cannot create non-existed node with <<=
    let mut port_node = v.path_mut() / "port";
    assert_eq!(port_node.unpath().is_none(), true);
    port_node <<= 80;
    assert_eq!(port_node.unpath().is_none(), true);

    // <<= can assign any type that support toml::from() method.
    let vecint = vec![1, 2, 3, 4];
    let mut node = v.path_mut() / "misc" / "int";
    node <<= vecint;
    let int = v.path() / "misc" / "int" / 0 | 0;
    assert_eq!(int, 1);
    let int = v.path() / "misc" / "int" / 1 | 0;
    assert_eq!(int, 2);

    let mut node = v.path_mut() / "misc" / "int";
    node.assign(1234);
    let int = v.path() / "misc" / "int" | 0;
    assert_eq!(int, 1234);
}

#[test]
fn path_if_test() {
    let mut v = load_test_toml();

    let node = v.path() / "ip";
    if node.is_some() {
        let ip = node | "";
        assert_eq!(ip, "127.0.0.1");
    }
    if !!node {
        let ip = node | "";
        assert_eq!(ip, "127.0.0.1");
    }

    let node = v.path() / "IP";
    if node.is_none() {
        let ip = node | "";
        assert_eq!(ip, "");
    }
    if !node {
        let ip = node | "";
        assert_eq!(ip, "");
    }

    let node = v.path_mut() / "ip";
    // in mut version operator !node would move self
    if node.is_some() {
        let ip = node | "";
        assert_eq!(ip, "127.0.0.1");
    }

    let mut node = v.path_mut() / "ip";
    if !node.is_none() {
        node = node << "127.0.0.2";
        let ip = node | "";
        assert_eq!(ip, "127.0.0.2");
    }

    let mut node = v.path_mut() / "host" / "port";
    if !node.is_none() {
        node = node << "127.0.0.2";
        assert_eq!(node.is_none(), true);
        let val = node | 0;
        assert_eq!(val, 0);
    }
}

