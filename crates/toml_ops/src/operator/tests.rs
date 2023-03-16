// test private items
use super::*;

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

