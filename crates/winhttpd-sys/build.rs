use std::env;
use std::path::PathBuf;

fn main() {
    // Only compile on Windows - this is a Windows-only crate
    if cfg!(target_os = "windows") {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let winhttpd_c = manifest_dir.join("winhttpd_lib.c");

        println!("cargo:rerun-if-changed={}", winhttpd_c.display());
        println!("cargo:rerun-if-changed={}", manifest_dir.join("winhttpd_lib.h").display());
        println!("cargo:rerun-if-changed={}", manifest_dir.join("dirent.h").display());

        // Compile winhttpd C source for MSVC
        cc::Build::new()
            .file(&winhttpd_c)
            .warnings(false)  // Suppress C code warnings
            .opt_level(2)
            .compile("winhttpd");

        println!("cargo:rustc-link-lib=static=winhttpd");

        // Link Windows socket library (required by winhttpd)
        println!("cargo:rustc-link-lib=ws2_32");
    }
}
