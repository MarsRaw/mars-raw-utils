use std::env;
use std::path::Path;

// Checks if file exists.
pub fn file_exists(chk_path: &str) -> bool {
    Path::new(&chk_path).exists()
}

pub fn is_file(chk_path: &str) -> bool {
    Path::new(&chk_path).is_file()
}

pub fn is_dir(chk_path: &str) -> bool {
    Path::new(&chk_path).is_dir()
}

pub fn basename(chk_path: &str) -> String {
    String::from(Path::new(&chk_path).file_name().unwrap().to_str().unwrap())
}

pub fn file_writable(chk_path: &str) -> bool {
    let path = Path::new(&chk_path);
    !path.metadata().unwrap().permissions().readonly()
}

pub fn get_parent(chk_path: &str) -> String {
    let path = Path::new(&chk_path);
    let parent = path.parent().unwrap();
    String::from(parent.to_str().unwrap())
}

pub fn parent_exists(chk_path: &str) -> bool {
    let parent = get_parent(chk_path);
    if !parent.is_empty() {
        file_exists(parent.as_str())
    } else {
        true // Cannot assume true on $CWD (or that it is $CWD)...
    }
}

pub fn parent_writable(chk_path: &str) -> bool {
    let parent = get_parent(chk_path);
    if !parent.is_empty() {
        file_writable(parent.as_str())
    } else {
        true // Cannot assume true on $CWD (or that is is $CWD)...
    }
}

pub fn parent_exists_and_writable(chk_path: &str) -> bool {
    parent_exists(chk_path) && parent_writable(chk_path)
}

pub fn cwd() -> String {
    match env::current_dir().unwrap().as_os_str().to_str() {
        Some(d) => String::from(d),
        None => String::from("./"),
    }
}
