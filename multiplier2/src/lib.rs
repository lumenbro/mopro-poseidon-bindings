#[macro_use]
mod stubs;

mod error;
pub use error::MoproError;

// Initializes the shared UniFFI scaffolding and defines the `MoproError` enum.
#[cfg(not(target_arch = "wasm32"))]
mopro_ffi::app!();
// Skip wasm_setup!() to avoid extern crate alias conflict
// Instead, we import wasm_bindgen directly when needed
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use mopro_ffi::prelude::wasm_bindgen;

/// You can also customize the bindings by #[uniffi::export]
/// Reference: https://mozilla.github.io/uniffi-rs/latest/proc_macro/index.html
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn mopro_hello_world() -> String {
    "Hello, World!".to_string()
}

#[cfg_attr(
    all(feature = "wasm", target_arch = "wasm32"),
    wasm_bindgen(js_name = "moproWasmHelloWorld")
)]
pub fn mopro_wasm_hello_world() -> String {
    "Hello, World!".to_string()
}

#[cfg(test)]
mod uniffi_tests {
    #[test]
    fn test_mopro_hello_world() {
        assert_eq!(super::mopro_hello_world(), "Hello, World!");
    }
}


// CIRCOM_TEMPLATE
rust_witness::witness!(preimageposeidon);
rust_witness::witness!(multiplier2bls);
rust_witness::witness!(multiplier2);

mopro_ffi::set_circom_circuits! {
("/home/brandonian/LumenBroMobile/mopro-bindings/lumenbro_zk/./test-vectors/circom/preimage_poseidon_final.zkey", mopro_ffi::witness::WitnessFn::RustWitness(preimageposeidon_witness)),
("/home/brandonian/LumenBroMobile/mopro-bindings/lumenbro_zk/./test-vectors/circom/multiplier2_bls_final.zkey", mopro_ffi::witness::WitnessFn::RustWitness(multiplier2bls_witness)),
("/home/brandonian/LumenBroMobile/mopro-bindings/lumenbro_zk/./test-vectors/circom/multiplier2_final.zkey", mopro_ffi::witness::WitnessFn::RustWitness(multiplier2_witness)),
}

// HALO2_TEMPLATE
halo2_stub!();

// NOIR_TEMPLATE
noir_stub!();
