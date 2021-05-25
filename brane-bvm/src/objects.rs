use crate::bytecode::FunctionMut;
use crate::{bytecode::Chunk, stack::Slot};
use broom::prelude::*;
use specifications::common::FunctionExt;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Object {
    Array(Array),
    Class(Class),
    Function(Function),
    FunctionExt(FunctionExt),
    Instance(Instance),
    String(String),
}

impl Object {
    #[inline]
    pub fn as_class(&self) -> Option<&Class> {
        if let Object::Class(class) = self {
            Some(class)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_function(&self) -> Option<&Function> {
        if let Object::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        if let Object::String(string) = self {
            Some(string)
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
            Object::FunctionExt(_f) => {},
            Object::Instance(i) => i.trace(tracer),
            Object::String(_) => {}
        }
    }
}
#[derive(Debug)]
pub struct Array {
    pub element_type: String,
    pub elements: Vec<Slot>,
}

impl Array {
    pub fn new(elements: Vec<Slot>) -> Self {
        let element_type = if elements.is_empty() {
            String::from("unit")
        } else {
            String::from("???")
        };

        Self { element_type, elements }
    }
}

impl Trace<Object> for Array {
    fn trace(
        &self,
        _tracer: &mut Tracer<Object>,
    ) {
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
    pub chunk: Chunk,
    pub name: String,
}

impl Function {
    ///
    ///
    ///
    pub fn new(
        name: String,
        arity: u8,
        chunk: Chunk,
    ) -> Self {
        Self { arity, chunk, name }
    }

    ///
    ///
    ///
    pub fn unfreeze(
        self,
        heap: &Heap<Object>,
    ) -> FunctionMut {
        FunctionMut::new(self.name, self.arity, self.chunk.unfreeze(heap))
    }
}

impl Trace<Object> for Function {
    fn trace(
        &self,
        _tracer: &mut Tracer<Object>,
    ) {
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: Handle<Object>,
    pub properties: HashMap<String, Slot>,
}

impl Instance {
    ///
    ///
    ///
    pub fn new(
        class: Handle<Object>,
        properties: HashMap<String, Slot>,
    ) -> Self {
        Self { class, properties }
    }
}

impl Trace<Object> for Instance {
    fn trace(
        &self,
        tracer: &mut Tracer<Object>,
    ) {
        self.class.trace(tracer);
    }
}
