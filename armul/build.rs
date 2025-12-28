use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

fn main() {
    println!("cargo::rerun-if-changed=test/");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tests.rs");
    let mut file = BufWriter::new(File::create(dest_path).unwrap());

    for entry in glob::glob("test/**/*.s").unwrap() {
        let entry = entry.unwrap();
        let test_name = entry.strip_prefix("test").unwrap();
        let parent = test_name.parent().unwrap();
        for folder in parent.iter() {
            writeln!(file, "mod {} {{", folder.to_string_lossy()).unwrap();
        }
        writeln!(file, "#[test]").unwrap();
        writeln!(
            file,
            "fn {}() -> Result<(), crate::test::TestError> {{",
            entry.file_stem().unwrap().to_string_lossy()
        )
        .unwrap();
        writeln!(file, "let src = std::fs::read_to_string({entry:?}).map_err(|x| crate::test::TestError::FileError(x.to_string()))?;").unwrap();
        writeln!(file, "crate::test::test(&src)").unwrap();
        writeln!(file, "}}").unwrap();
        for _ in parent.iter() {
            writeln!(file, "}}").unwrap();
        }
        writeln!(file).unwrap();
    }

    file.flush().unwrap();
}
