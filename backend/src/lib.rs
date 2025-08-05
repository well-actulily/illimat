pub mod game;

// WebAssembly interface - only included when compiling for WASM
#[cfg(target_arch = "wasm32")]
pub mod wasm_interface;

pub use game::*;