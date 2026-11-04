//! Embeds every file under ../dist into the component as a sorted table of
//! (path, bytes) and an ETag that changes with any file.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{env, fs};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("dist/ exists: run `make web` first") {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn main() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
    let dist = dist
        .canonicalize()
        .expect("dist/ exists: run `make web` first");
    println!("cargo:rerun-if-changed={}", dist.display());

    let mut paths = Vec::new();
    walk(&dist, &mut paths);
    // Sorted by URL, the key the server binary-searches on.
    let mut files: Vec<(String, PathBuf)> = paths
        .into_iter()
        .map(|p| {
            let url = p
                .strip_prefix(&dist)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            (url, p)
        })
        .collect();
    files.sort();

    let mut hasher = DefaultHasher::new();
    let mut table = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");
    for (url, path) in &files {
        println!("cargo:rerun-if-changed={}", path.display());
        url.hash(&mut hasher);
        fs::read(path).unwrap().hash(&mut hasher);
        table += &format!(
            "    (\"/{url}\", include_bytes!({:?})),\n",
            path.display().to_string()
        );
    }
    table += "];\n";
    table += &format!(
        "pub const ETAG: &str = \"\\\"{:016x}\\\"\";\n",
        hasher.finish()
    );

    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("files.rs");
    fs::write(out, table).unwrap();
}
