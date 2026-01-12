use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();

    // Check if we're cross-compiling for iOS or Android
    let is_ios = target.contains("apple-ios");
    let is_android = target.contains("android");
    let is_cross_compile = is_ios || is_android;

    // Check if we have pre-generated circuit files (from native build)
    let pregenerated_dir = env::var("CIRCUIT_C_FILES_DIR").ok();

    if is_cross_compile {
        if let Some(circuit_dir) = pregenerated_dir {
            // For cross-compilation (iOS/Android), use pre-generated C files
            // and compile them with cc-rs for the target platform
            println!("cargo:warning=Using pre-generated circuit files from {} for target {}", circuit_dir, target);

            let circuit_path = PathBuf::from(&circuit_dir);

            // Find ALL C files in the circuit directory
            // w2c2 generates multiple C files for large circuits:
            // - preimage_poseidon.c (main exports)
            // - preimage_poseidon_XX.c (internal functions in chunks)
            let mut c_files: Vec<PathBuf> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&circuit_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "c") {
                        c_files.push(path);
                    }
                }
            }

            if c_files.is_empty() {
                panic!("No C files found in circuit directory: {}", circuit_path.display());
            }

            println!("cargo:warning=Found {} C files to compile", c_files.len());
            for f in &c_files {
                println!("cargo:warning=  - {}", f.display());
            }

            let mut build = cc::Build::new();
            for c_file in &c_files {
                build.file(c_file);
                println!("cargo:rerun-if-changed={}", c_file.display());
            }
            build.include(&circuit_path).opt_level(3);

            // For Android, we need _GNU_SOURCE to get proper locale_t definition
            // from the NDK headers
            if is_android {
                build.define("_GNU_SOURCE", None);
            }

            build.compile("circuit");
        } else {
            // No pre-generated files - this will likely fail for cross-compilation
            panic!("CIRCUIT_C_FILES_DIR must be set for cross-compilation to {} - run native build first to generate circuit files", target);
        }
    } else {
        // For native builds (macOS, Linux), use normal transpilation
        rust_witness::transpile::transpile_wasm("./test-vectors/circom".to_string());

        // Print OUT_DIR so the workflow can find the generated files
        if let Ok(out_dir) = env::var("OUT_DIR") {
            println!("cargo:warning=Generated circuit files in: {}", out_dir);
        }
    }
}
