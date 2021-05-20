use broom::prelude::*;

use crate::chunk::FrozenChunk;

#[derive(Debug)]
pub enum Object {
    Array(Vec<Handle<Self>>),
    Class(Class),
    Function(Function),
    Instance(Instance),
    String(String),
}

impl Object {
    #[inline]
    pub fn as_function(&self) -> Option<&Function> {
        if let Object::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }
}

// Tell the garbage collector how to explore a graph of this object
impl Trace<Self> for Object {
    fn trace(
        &self,
        tracer: &mut Tracer<Self>,
    ) {
        match self {
            Object::Array(a) => a.trace(tracer),
            Object::Class(c) => c.trace(tracer),
            Object::Function(f) => f.trace(tracer),
            Object::Instance(i) => i.trace(tracer),
            Object::String(_) => {}
        }
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
}

impl Trace<Object> for Class {
    fn trace(
        &self,
        _tracer: &mut Tracer<Object>,
    ) {
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: u8,
    pub chunk: FrozenChunk,
    pub name: String,
}

impl Trace<Object> for Function {
    fn trace(
        &self,
        _tracer: &mut Tracer<Object>,
    ) {
    }
}

#[derive(Debug)]
pub struct Instance {}

impl Trace<Object> for Instance {
    fn trace(
        &self,
        _tracer: &mut Tracer<Object>,
    ) {
    }
}
