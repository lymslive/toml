use toml_ops::PathOperator;

fn main()
{
    let str_toml = include_str!("./sample.toml");
    let mut v: toml::Value = str_toml.parse().unwrap();

    println!("original toml content:");
    println!("{str_toml}");

    println!("read by path:");

    // The mutable struct has no Copy trait, moved by path operator,
    // and so cannot save the intermedia variable.

    // let root = v.path_mut();
    let ip = v.path_mut() / "ip" | "";
    println!("/ip = {ip}");

    // let host = root / "host";
    let ip = v.path_mut() / "host" / "ip" | "";
    println!("/host/ip = {ip}");
    let port = v.path_mut() / "host" / "port" | 0;
    println!("/host/port = {port}");

    let name = v.path_mut() / "service" / 0 / "name" | "";
    println!("/service/0/name = {name}");
    let desc = v.path_mut() / "service" / 0 / "desc" | "";
    println!("/service/0/desc = {desc}");

    let name = v.pathto_mut("service/1/name") | "";
    println!("/service/1/name = {name}");
    let desc = v.pathto_mut("service.1.desc") | "";
    println!("/service/1/desc = {desc}");

    let int = v.path_mut() / "misc" / "int" | 0;
    let float = v.path_mut() / "misc" / "float" | 0.0;
    let tf = v.path_mut() / "misc" / "bool" | false;
    println!("/misc/int = {int}");
    println!("/misc/float = {float}");
    println!("/misc/bool = {tf}");
}
