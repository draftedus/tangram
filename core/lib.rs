/*!
Tangram .
*/

#![allow(clippy::tabs_in_doc_comments)]

mod config;
mod grid;
mod test;
mod text;

pub mod features;
pub mod model;
pub mod predict;
pub mod stats;
pub mod train;

pub use self::{predict::predict, train::train};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
