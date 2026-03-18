use std::env;
use std::path::PathBuf;
use once_cell::sync::Lazy;

pub static SHARED_DIR_PATH: Lazy<PathBuf> = Lazy::new(|| env::current_dir().unwrap().join("experimentation").join("shared_with_vm"));
