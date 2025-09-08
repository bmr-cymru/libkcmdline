//! Build script to compile parameter database into binary
use std::env;
use std::path::Path;
//use walkdir::WalkDir;
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir);

    println!("cargo:rerun-if-changed=database/");

    // Walk database directory and compile TOML files into Rust code
    compile_database(&dest_path)
}

fn compile_database(output: &Path) -> std::io::Result<()> {
    // Implementation to read all TOML files and generate embedded data
    let parameter_names = output.join("parameter_names.rs");
    let compiled_db = output.join("compiled_db.rs");

    let mut parameter_file = File::create(parameter_names)?;
    parameter_file.write_all(b"// Getting started\n(vec![\"foo\", \"bar\", \"baz\"]).into_iter().map(|i| i.to_string()).collect()\n")?;

    let mut db_file = File::create(compiled_db)?;
    db_file.write_all(b"// Getting started\nHashMap::from([(\"foo\", \"bar\"), (\"baz\", \"quux\")])\n")?;

    Ok(())
}
