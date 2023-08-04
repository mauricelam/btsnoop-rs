#[test]
fn test_btsnoop_version() {
    let readme = include_str!("../README.md");
    assert!(readme.contains(format!(r#"btsnoop = "{}""#, env!("CARGO_PKG_VERSION")).as_str()));
}
