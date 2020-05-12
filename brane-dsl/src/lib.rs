#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod compiler;
pub mod configuration;
pub mod functions;
pub mod parser;
pub mod repl;
pub mod setup;
pub mod terms;
