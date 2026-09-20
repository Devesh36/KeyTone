use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../../packs");
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let packs_root = manifest_dir.join("../../packs");
    let generated = generate_bundled_packs(&packs_root).expect("generate bundled pack table");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("out dir"));
    fs::write(out_dir.join("bundled_packs.rs"), generated).expect("write bundled pack table");
}

fn generate_bundled_packs(root: &Path) -> io::Result<String> {
    let mut directories = fs::read_dir(root)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter(|entry| entry.path().join(".keytone-bundled").is_file())
        .collect::<Vec<_>>();
    directories.sort_by_key(fs::DirEntry::file_name);

    let mut output = String::from("static BUNDLED_PACKS: &[BundledPack] = &[\n");
    for directory in directories {
        let id = directory.file_name().to_string_lossy().into_owned();
        let path = directory.path();
        let version = fs::read_to_string(path.join(".keytone-bundled"))?;
        let mut files = Vec::new();
        collect_files(&path, &path, &mut files)?;
        files.sort();

        output.push_str("    BundledPack {\n");
        output.push_str(&format!("        id: {id:?},\n"));
        output.push_str(&format!("        version: {:?},\n", version.trim()));
        output.push_str("        files: &[\n");
        for file in files {
            let relative = file
                .strip_prefix(&path)
                .expect("collected file belongs to pack")
                .to_string_lossy()
                .replace('\\', "/");
            if relative == ".keytone-bundled" {
                continue;
            }
            let absolute = file.canonicalize()?;
            output.push_str("            BundledPackFile { ");
            output.push_str(&format!("relative_path: {relative:?}, "));
            output.push_str(&format!(
                "bytes: include_bytes!({:?}) ",
                absolute.to_string_lossy()
            ));
            output.push_str("},\n");
        }
        output.push_str("        ],\n");
        output.push_str("    },\n");
    }
    output.push_str("];\n");
    Ok(output)
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else if file_type.is_file() {
            let path = entry.path();
            if path.strip_prefix(root).is_ok() {
                files.push(path);
            }
        }
    }
    Ok(())
}
