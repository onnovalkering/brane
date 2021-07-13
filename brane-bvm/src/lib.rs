#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod builtins;
pub mod bytecode;
pub mod executor;
mod frames;
pub mod objects;
mod stack;
pub mod values;
pub mod vm;

pub use objects::Function;
