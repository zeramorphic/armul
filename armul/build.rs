use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

/// Builds the test suite.
/// A test is generated for each `.s` file in the `test` subdirectory.
fn main() {
    println!("cargo::rerun-if-changed=test/");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tests.rs");
    let mut file = BufWriter::new(File::create(dest_path).unwrap());

    traverse(&mut file, &PathBuf::from("test"));

    file.flush().unwrap();
}

fn traverse(file: &mut impl std::io::Write, path: &Path) {
    println!("traversing {path:?}");
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            writeln!(file, "mod {} {{", entry.file_name().to_string_lossy()).unwrap();
            traverse(file, &entry.path());
            writeln!(file, "}}").unwrap();
        } else if entry.path().extension().map(|x| x.to_string_lossy())
            == Some(std::borrow::Cow::Borrowed("s"))
        {
            writeln!(file, "#[test]").unwrap();
            writeln!(
                file,
                "fn {}() -> Result<(), crate::test::TestError> {{",
                entry.path().file_stem().unwrap().to_string_lossy()
            )
            .unwrap();
            writeln!(file, "let src = std::fs::read_to_string({:?}).map_err(|x| crate::test::TestError::FileError(x.to_string()))?;", entry.path()).unwrap();
            writeln!(file, "crate::test::test(&src)").unwrap();
            writeln!(file, "}}").unwrap();
            writeln!(file).unwrap();
        }
    }
}
