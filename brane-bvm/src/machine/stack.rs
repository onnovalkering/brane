use std::usize;

use broom::Handle;

use crate::objects::Object;

const STACK_MAX: usize = 256;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Slot {
    ConstMinusOne,
    ConstMinusTwo,
    ConstOne,
    ConstTwo,
    ConstZero,
    False,
    Number(i64),
    Real(f64),
    True,
    Unit,
    Object(Handle<Object>),
}

#[derive(Debug)]
pub struct Stack {
    inner: Vec<Slot>,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new(STACK_MAX)
    }
}

impl Stack {
    pub fn new(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn get(
        &self,
        index: usize,
    ) -> &Slot {
        if let Some(slot) = self.inner.get(index) {
            slot
        } else {
            panic!("Expected value");
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn copy_push(
        &mut self,
        index: usize,
    ) {
        self.push_unit();

        let length = self.inner.len();
        self.inner.copy_within(index..index + 1, length - 1);
    }

    ///
    ///
    ///
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    ///
    ///
    ///
    #[inline]
    pub fn pop(&mut self) -> Slot {
        self.inner.pop().unwrap()
    }

    ///
    ///
    ///
    #[inline]
    pub fn pop_boolean(&mut self) -> bool {
        if let Some(slot) = self.inner.pop() {
            match slot {
                Slot::False => false,
                Slot::True => true,
                _ => panic!("Expecting a boolean."),
            }
        } else {
            panic!("Empty stack.");
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn pop_number(&mut self) -> i64 {
        if let Some(slot) = self.inner.pop() {
            match slot {
                // TODO: benchmark if this really makes sense.
                Slot::ConstMinusTwo => -2,
                Slot::ConstMinusOne => -1,
                Slot::ConstZero => 0,
                Slot::ConstOne => 1,
                Slot::ConstTwo => 2,
                Slot::Number(n) => n,
                _ => panic!("Expecting a number."),
            }
        } else {
            panic!("Empty stack.");
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn pop_object(&mut self) -> Handle<Object> {
        if let Some(slot) = self.inner.pop() {
            match slot {
                Slot::Object(h) => h,
                _ => panic!("Expecting a object."),
            }
        } else {
            panic!("Empty stack.");
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn pop_real(&mut self) -> f64 {
        if let Some(slot) = self.inner.pop() {
            match slot {
                Slot::Real(r) => r,
                _ => panic!("Expecting a real."),
            }
        } else {
            panic!("Empty stack.");
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn pop_unit(&mut self) -> () {
        if let Some(slot) = self.inner.pop() {
            match slot {
                Slot::Unit => (),
                _ => panic!("Expecting unit."),
            }
        } else {
            panic!("Empty stack.");
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn push(
        &mut self,
        slot: Slot,
    ) {
        self.inner.push(slot);
    }

    ///
    ///
    ///
    #[inline]
    pub fn push_boolean(
        &mut self,
        boolean: bool,
    ) {
        let boolean = match boolean {
            false => Slot::False,
            true => Slot::True,
        };

        self.inner.push(boolean);
    }

    ///
    ///
    ///
    #[inline]
    pub fn push_number(
        &mut self,
        number: i64,
    ) {
        let number = match number {
            -2 => Slot::ConstMinusTwo,
            -1 => Slot::ConstMinusOne,
            0 => Slot::ConstZero,
            1 => Slot::ConstOne,
            2 => Slot::ConstTwo,
            n => Slot::Number(n),
        };

        self.inner.push(number);
    }

    ///
    ///
    ///
    #[inline]
    pub fn push_object(
        &mut self,
        object: Handle<Object>,
    ) {
        self.inner.push(Slot::Object(object));
    }

    ///
    ///
    ///
    #[inline]
    pub fn push_real(
        &mut self,
        real: f64,
    ) {
        self.inner.push(Slot::Real(real));
    }

    ///
    ///
    ///
    #[inline]
    pub fn push_unit(&mut self) {
        self.inner.push(Slot::Unit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_push() {
        let mut stack = Stack::default();
        stack.push(Slot::Number(1));
        stack.push(Slot::Number(2));

        stack.copy_push(0);

        assert_eq!(stack.pop_number(), 1);
        assert_eq!(stack.pop_number(), 2);
        assert_eq!(stack.pop_number(), 1);
    }
}
