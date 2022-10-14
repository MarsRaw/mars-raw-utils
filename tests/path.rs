use mars_raw_utils::path;

#[test]
fn test_file_exists() {
    assert!(path::file_exists("Cargo.toml"));
}

#[test]
fn test_file_writable() {
    assert!(path::file_writable("Cargo.toml"));
}

#[test]
fn test_parent_exists() {
    assert!(path::parent_exists("Cargo.toml"));
}

#[test]
fn test_parent_writable() {
    assert!(path::parent_writable("Cargo.toml"));
}

#[test]
fn test_parent_exists_and_writable() {
    assert!(path::parent_exists_and_writable("Cargo.toml"));
}
