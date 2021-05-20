use std::{cmp::Ordering, fmt::{Display, Formatter, Result}, usize};
use broom::Handle;
use std::fmt::Write;
use crate::objects::Object;

const STACK_MAX: usize = 256;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Slot {
    BuiltIn(u8),
    ConstMinusOne,
    ConstMinusTwo,
    ConstOne,
    ConstTwo,
    ConstZero,
    False,
    Integer(i64),
    Real(f64),
    True,
    Unit,
    Object(Handle<Object>),
}

impl Display for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let display = match self {
            Slot::BuiltIn(code) => format!("builtin<{:#04x}>", code),
            Slot::ConstMinusOne => String::from("-1"),
            Slot::ConstMinusTwo => String::from("-2"),
            Slot::ConstOne => String::from("1"),
            Slot::ConstTwo => String::from("2"),
            Slot::ConstZero => String::from("0"),
            Slot::False => String::from("false"),
            Slot::Integer(i) => format!("{}", i),
            Slot::Real(r) => format!("{}", r),
            Slot::True => String::from("true"),
            Slot::Unit => String::from("unit"),
            Slot::Object(h) => {
                unsafe {
                    match h.get_unchecked() {
                        Object::Array(_) => format!("array<{}>", "?"),
                        Object::Class(c) => format!("class<{}>", c.name),
                        Object::Function(f) => format!("function<{}>", f.name),
                        Object::Instance(_) => format!("instance<{}>", "?"),
                        Object::String(s) => format!("{:?}", s),
                    }
                }
            },
            _ => todo!()
        };

        write!(f, "{}", display)
    }
}

impl PartialEq for Slot {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Slot::BuiltIn(lhs), Slot::BuiltIn(rhs)) => lhs == rhs,
            (Slot::ConstMinusOne, Slot::ConstMinusOne) => true,
            (Slot::ConstMinusTwo, Slot::ConstMinusTwo) => true,
            (Slot::ConstOne, Slot::ConstOne) => true,
            (Slot::ConstTwo, Slot::ConstTwo) => true,
            (Slot::ConstZero, Slot::ConstZero) => true,
            (Slot::False, Slot::False) => true,
            (Slot::Integer(lhs), Slot::Integer(rhs)) => lhs == rhs,
            (Slot::Real(lhs), Slot::Real(rhs)) => lhs == rhs,
            (Slot::True, Slot::True) => true,
            (Slot::Unit, Slot::Unit) => true,
            (Slot::Object(lhs), Slot::Object(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl PartialOrd for Slot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Slot::Integer(lhs), Slot::Integer(rhs)) => lhs.partial_cmp(rhs),
            (Slot::Integer(lhs), Slot::Real(rhs)) => (*lhs as f64).partial_cmp(rhs),
            (Slot::Real(lhs), Slot::Real(rhs)) => lhs.partial_cmp(rhs),
            (Slot::Real(lhs), Slot::Integer(rhs)) => lhs.partial_cmp(&(*rhs as f64)),
            _ => None
        }

    }
}

#[derive(Debug)]
pub struct Stack {
    inner: Vec<Slot>,
    use_const: bool, // TODO: benchmark this option
}

impl Default for Stack {
    fn default() -> Self {
        Self::new(STACK_MAX, false)
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut display = String::from(format!("         "));
        self.inner.iter().for_each(|v| write!(display, "[ {} ]", v).unwrap());

        write!(f, "{}", display)
    }
}

impl Stack {
    ///
    ///
    ///
    pub fn new(size: usize, use_const: bool) -> Self {
        Self {
            inner: Vec::with_capacity(size),
            use_const,
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    ///
    ///
    ///
    #[inline]
    pub fn clear_from(&mut self, index: usize) {
        self.inner.truncate(index)
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
    pub fn get_object(
        &self,
        index: usize,
    ) -> &Handle<Object> {
        if let Some(slot) = self.inner.get(index) {
            match slot {
                Slot::Object(h) => h,
                _ => panic!("Expecting a object."),
            }
        } else {
            panic!("Expecting value");
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
    pub fn peek_boolean(&mut self) -> bool {
        match self.inner.last().expect("Expecting a non-empty stack.") {
            Slot::False => false,
            Slot::True => true,
            _ => panic!("Expecting a boolean."),
        }
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
    pub fn pop_integer(&mut self) -> i64 {
        if let Some(slot) = self.inner.pop() {
            match slot {
                Slot::ConstMinusTwo => -2,
                Slot::ConstMinusOne => -1,
                Slot::ConstZero => 0,
                Slot::ConstOne => 1,
                Slot::ConstTwo => 2,
                Slot::Integer(n) => n,
                _ => panic!("Expecting a integer."),
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
    pub fn push_integer(
        &mut self,
        integer: i64,
    ) {
        let integer = if self.use_const {
            match integer {
                -2 => Slot::ConstMinusTwo,
                -1 => Slot::ConstMinusOne,
                0 => Slot::ConstZero,
                1 => Slot::ConstOne,
                2 => Slot::ConstTwo,
                n => Slot::Integer(n)
            }
        } else {
            Slot::Integer(integer)
        };

        self.inner.push(integer);
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

    ///
    ///
    ///
    #[inline]
    pub fn try_pop(&mut self) -> Option<Slot> {
        self.inner.pop()
    }

    ///
    ///
    ///
    #[inline]
    pub fn try_push(&mut self, slot: Option<Slot>) {
        if let Some(slot) = slot {
            self.inner.push(slot)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_push() {
        let mut stack = Stack::default();
        stack.push(Slot::Integer(1));
        stack.push(Slot::Integer(2));

        stack.copy_push(0);

        assert_eq!(stack.pop_integer(), 1);
        assert_eq!(stack.pop_integer(), 2);
        assert_eq!(stack.pop_integer(), 1);
    }
}
