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

            // Find circuit C files in the directory
            // rust-witness generates several types of C files:
            // - preimage_poseidon.c (main circuit exports and entry points)
            // - s0000000001.c, s0000000002.c, ... (individual function chunks from w2c2 -f 1)
            // - globals.c (witness_c_init, witness_c_resolver, witness_c_cleanup)
            // - handlers.c (module-specific runtime handlers like preimage_poseidon_runtime__exceptionHandler)
            let mut c_files: Vec<PathBuf> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&circuit_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Include:
                        // 1. *_poseidon*.c - main circuit files (preimage_poseidon, universal_poseidon, etc.)
                        // 2. s0*.c - w2c2 function chunk files (s0000000001.c, etc.)
                        // 3. globals.c - witness lifecycle functions
                        // 4. handlers.c - runtime exception/error handlers
                        let is_circuit_file = file_name.ends_with(".c") &&
                            (file_name.starts_with("preimage_poseidon") ||
                             file_name.starts_with("universal_poseidon"));
                        let is_chunk_file = file_name.starts_with("s0") && file_name.ends_with(".c");
                        let is_runtime_file = file_name == "globals.c" || file_name == "handlers.c";
                        if is_circuit_file || is_chunk_file || is_runtime_file {
                            c_files.push(path);
                        }
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
