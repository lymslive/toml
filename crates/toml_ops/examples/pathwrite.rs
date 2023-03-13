use toml_ops::PathOperator;

fn main()
{
    let str_toml = include_str!("./sample.toml");
    let mut v: toml::Value = str_toml.parse().unwrap();

    println!("original toml content:");
    println!("{str_toml}");

    println!("modify by path:");

    let node = v.path_mut() / "ip";
    let _ = node << "127.0.0.2";

    // push key-val pair to table
    let mut node = v.path_mut() / "host";
    node = node << ("newkey1", 1) << ("newkey2", "2");

    // push scalar to leaf node, replace it
    let _ = node / "port" << 8888;

    // push single tuple to array
    node = v.path_mut() / "host" /"protocol";
    let _ = node << (8989,) << ("xyz",);

    // <<= can change node type while << cannot
    node = v.path_mut() / "misc" / "bool";
    node <<= "false";

    println!("{}", v);
}
