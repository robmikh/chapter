use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/yarn_spinner.proto"], &["src/"])?;

    if cfg!(test) {
        for entry in std::fs::read_dir("test_files").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if let Some(extension) = extension.to_str() {
                    if extension == "yarn" {
                        let yarn_path = path.to_str().unwrap();
                        println!("cargo:rerun-if-changed={}", yarn_path);
                        let status = std::process::Command::new("ysc.exe")
                                .args([
                                    "compile",
                                    "--output-directory",
                                    "test_files",
                                    yarn_path,
                                ])
                                .status()
                                .unwrap();
                            assert!(status.success());
                    }
                }
            }
        }
    }

    Ok(())
}
