/*!
Tangram .
*/

#![allow(clippy::tabs_in_doc_comments)]

mod config;
pub mod features;
mod grid;
pub mod model;
pub mod predict;
pub mod stats;
mod test;
mod text;
pub mod train;

pub use self::{predict::predict, train::train};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
