pub mod game;

// WebAssembly interface - only included when compiling for WASM
#[cfg(target_arch = "wasm32")]
pub mod wasm_interface;

pub use game::*;

// Re-export WASM interface when targeting WASM
#[cfg(target_arch = "wasm32")]
pub use wasm_interface::*;