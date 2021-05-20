use broom::Heap;
use bytes::{BufMut, Bytes, BytesMut};
use crate::{objects::Object, stack::Slot, values::Value};

#[derive(Clone, Debug)]
pub struct Chunk {
    pub code: BytesMut,
    pub constants: Vec<Value>,
}

impl Chunk {
    ///
    ///
    ///
    pub fn new() -> Self {
        Chunk {
            code: BytesMut::new(),
            constants: Vec::new(),
        }
    }

    ///
    ///
    ///
    pub fn freeze(self, heap: &mut Heap<Object>) -> FrozenChunk {
        let constants = self.constants.into_iter().map(|c| {
            match c {
                Value::Boolean(b) => {
                    match b {
                        true => Slot::True,
                        false => Slot::False,
                    }
                }
                Value::Integer(i) => Slot::Integer(i),
                Value::Real(r) => Slot::Real(r),
                Value::Function(f) => {
                    let function = Object::Function(f.freeze(heap));
                    let handle = heap.insert(function).into_handle();

                    Slot::Object(handle)
                },
                Value::String(s) => {
                    let string = Object::String(s);
                    let handle = heap.insert(string).into_handle();

                    Slot::Object(handle)
                },
                a => {
                    dbg!(&a);
                    todo!();
                }
            }
        })
        .collect();

        FrozenChunk {
            code: self.code.freeze(),
            constants,
        }
    }

    ///
    ///
    ///
    pub fn write<B: Into<u8>>(
        &mut self,
        byte: B,
    ) {
        self.code.put_u8(byte.into());
    }

    ///
    ///
    ///
    pub fn write_pair<B1: Into<u8>, B2: Into<u8>>(
        &mut self,
        byte1: B1,
        byte2: B2,
    ) {
        self.code.put_u8(byte1.into());
        self.code.put_u8(byte2.into());
    }

    ///
    ///
    ///
    pub fn write_bytes(
        &mut self,
        bytes: &[u8],
    ) {
        self.code.extend(bytes);
    }

    ///
    ///
    ///
    pub fn add_constant(
        &mut self,
        value: Value,
    ) -> u8 {
        self.constants.push(value);

        (self.constants.len() as u8) - 1
    }
}

#[derive(Debug, Clone)]
pub struct FrozenChunk {
    pub code: Bytes,
    pub constants: Vec<Slot>,
}
