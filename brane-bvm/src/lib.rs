#[macro_use]
extern crate anyhow;

mod builtins;
pub mod bytecode;
pub mod executor;
mod frames;
mod objects;
mod stack;
pub mod values;
pub mod vm;

pub use objects::{Function, FunctionExt};
