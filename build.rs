use std::{fs, path::Path, env};

fn main() {
    println!("cargo::rerun-if-env-changed=PRESERVE_MEMORY_X");

    if env::var_os("PRESERVE_MEMORY_X").is_some() {
        println!("cargo::warning=not touching `memory.x`");
        println!("cargo::rerun-if-changed=memory.x");
        return
    }

    let target = Path::new("memory.x");
    let source = Path::new("app.x");

    // Copy the file
    fs::copy(&source, &target).unwrap();

    println!("cargo::warning=using `app.x`");
    println!("cargo::rerun-if-changed=app.x");
}
