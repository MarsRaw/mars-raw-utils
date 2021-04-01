
use std::path::Path;

// Checks if file exists.
pub fn file_exists(chk_path:&str) -> bool {
    Path::new(&chk_path).exists()
}

pub fn file_writable(chk_path:&str) -> bool {
    let path = Path::new(&chk_path);
    !path.metadata().unwrap().permissions().readonly()
}

pub fn get_parent(chk_path:&str) -> String {
    let path = Path::new(&chk_path);
    let parent = path.parent().unwrap();
    String::from(parent.to_str().unwrap())
}

pub fn parent_exists(chk_path:&str) -> bool {
    let parent = get_parent(chk_path);
    file_exists(parent.as_str())
}

pub fn parent_writable(chk_path:&str) -> bool {
    let parent = get_parent(chk_path);
    file_writable(parent.as_str())
}

pub fn parent_exists_and_writable(chk_path:&str) -> bool {
    parent_exists(chk_path) && parent_writable(chk_path)
}