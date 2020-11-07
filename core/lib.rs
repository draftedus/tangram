/*!
*/

mod config;
mod features;
mod grid;
pub mod model;
pub mod predict;
mod stats;
mod test;
mod train;

pub use self::{
	predict::predict,
	train::{train, Progress},
};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
