/*!
Tangram is cool.
*/

#![allow(clippy::tabs_in_doc_comments)]

mod config;
mod grid;
mod tangram;
mod test;
mod train;

pub mod dataframe;
pub mod features;
pub mod id;
pub mod linear;
pub mod metrics;
pub mod model;
pub mod predict;
pub mod progress;
pub mod stats;
pub mod tree;
pub mod util;

pub use self::{predict::predict, tangram::*, train::train};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
