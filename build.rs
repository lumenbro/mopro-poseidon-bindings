use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();

    // Check if we're cross-compiling for iOS
    let is_ios = target.contains("apple-ios");

    // Check if we have pre-generated circuit files (from native build)
    let pregenerated_dir = env::var("CIRCUIT_C_FILES_DIR").ok();

    if is_ios {
        if let Some(circuit_dir) = pregenerated_dir {
            // For iOS cross-compilation, use pre-generated C files
            // and compile them with cc-rs for iOS
            println!("cargo:warning=Using pre-generated circuit files from {}", circuit_dir);

            let circuit_path = PathBuf::from(&circuit_dir);

            // Find and compile the circuit C file
            let c_file = circuit_path.join("preimage_poseidon.c");
            if c_file.exists() {
                cc::Build::new()
                    .file(&c_file)
                    .include(&circuit_path)
                    .opt_level(3)
                    .compile("circuit");

                println!("cargo:rerun-if-changed={}", c_file.display());
            } else {
                panic!("Pre-generated circuit file not found: {}", c_file.display());
            }
        } else {
            // No pre-generated files, try normal transpilation
            // This will fail for iOS due to w2c2 cross-compilation issues
            println!("cargo:warning=No CIRCUIT_C_FILES_DIR set, attempting normal transpilation (may fail for iOS)");
            rust_witness::transpile::transpile_wasm("./test-vectors/circom".to_string());
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
