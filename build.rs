use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    println!("cargo:rerun-if-changed=assets/video_editor.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/app.manifest");

    if target_os == "windows" {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let res_obj = out_dir.join("video_editor_res.o");
        let rc_file = PathBuf::from("assets/video_editor.rc");

        let windres_bin = if let Ok(path) = env::var("WINDRES") {
            path
        } else if std::path::Path::new("/home/lazycat/.local/opt/llvm-mingw/bin/x86_64-w64-mingw32-windres").exists() {
            "/home/lazycat/.local/opt/llvm-mingw/bin/x86_64-w64-mingw32-windres".to_string()
        } else {
            "x86_64-w64-mingw32-windres".to_string()
        };

        println!("cargo:warning=Compiling Windows resources with {}", windres_bin);

        let status = Command::new(&windres_bin)
            .args(["-O", "coff", "-i"])
            .arg(&rc_file)
            .arg("-o")
            .arg(&res_obj)
            .status()
            .unwrap_or_else(|e| panic!("Failed to execute windres ({}): {}", windres_bin, e));

        if !status.success() {
            panic!("windres failed with exit code: {:?}", status.code());
        }

        println!("cargo:rustc-link-arg={}", res_obj.display());
        println!("cargo:warning=Successfully linked Windows resources object into binary!");
    }
}
