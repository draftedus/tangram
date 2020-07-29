mod config;
mod features;
mod grid;
mod stats;
mod tangram;
mod test;
mod train;

pub mod dataframe;
pub mod gbt;
pub mod id;
pub mod linear;
pub mod metrics;
pub mod predict;
pub mod progress;
pub mod types;
pub mod util;

pub use self::predict::predict;
pub use self::tangram::*;
pub use self::train::train;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
