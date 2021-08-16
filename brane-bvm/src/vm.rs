use std::cmp::max;

use crate::frames::CallFrame;
use crate::objects::Class;
use crate::stack::{Slot, Stack};
use crate::{
    builtins,
    bytecode::{opcodes::*, FunctionMut},
    executor::VmExecutor,
    objects::Object,
    objects::{Array, Instance},
};
use broom::{Handle, Heap};
use fnv::FnvHashMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use smallvec::SmallVec;
use specifications::common::{FunctionExt, Value};
use specifications::package::PackageIndex;
use tokio::runtime::Runtime;

#[derive(Clone, Default, Debug)]
pub struct VmOptions {
    ///
    ///
    ///
    pub clear_after_main: bool,

    ///
    ///
    ///
    pub global_return_halts: bool,
}

#[derive(Clone, Default, Debug)]
pub struct VmState {
    globals: FnvHashMap<String, Value>,
    options: VmOptions,
}

unsafe impl Send for VmState {}

impl VmState {
    fn new(
        globals: FnvHashMap<String, Value>,
        options: VmOptions,
    ) -> Self {
        Self { globals, options }
    }

    ///
    ///
    ///
    fn get_globals(
        &self,
        heap: &mut Heap<Object>,
    ) -> FnvHashMap<String, Slot> {
        let mut globals = FnvHashMap::default();

        // First process all the the classes.
        for (name, value) in &self.globals {
            if let Value::Class(_) = value {
                let slot = Slot::from_value(value.clone(), &globals, heap);
                globals.insert(name.clone(), slot);
            }
        }

        // Then the rest of the globals.
        for (name, value) in &self.globals {
            if let Value::Class(_) = value {
                continue;
            } else {
                let slot = Slot::from_value(value.clone(), &globals, heap);
                globals.insert(name.clone(), slot);
            }
        }

        globals
    }
}

///
///
///
pub struct Vm<E>
where
    E: VmExecutor + Clone + Send + Sync,
{
    executor: E,
    frames: SmallVec<[CallFrame; 64]>,
    globals: FnvHashMap<String, Slot>,
    heap: Heap<Object>,
    locations: Vec<Handle<Object>>,
    package_index: PackageIndex,
    options: VmOptions,
    stack: Stack,
}

impl<E> Default for Vm<E>
where
    E: VmExecutor + Clone + Send + Sync + Default,
{
    fn default() -> Self {
        let executor = E::default();
        let frames = SmallVec::with_capacity(64);
        let globals = FnvHashMap::<String, Slot>::with_capacity_and_hasher(256, Default::default());
        let heap = Heap::default();
        let locations = Vec::with_capacity(16);
        let package_index = PackageIndex::default();
        let options = VmOptions::default();
        let stack = Stack::default();

        Self::new(
            executor,
            frames,
            globals,
            heap,
            locations,
            package_index,
            options,
            stack,
        )
    }
}

impl<E> Vm<E>
where
    E: VmExecutor + Clone + Send + Sync,
{
    ///
    ///
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        executor: E,
        frames: SmallVec<[CallFrame; 64]>,
        globals: FnvHashMap<String, Slot>,
        heap: Heap<Object>,
        locations: Vec<Handle<Object>>,
        package_index: PackageIndex,
        options: VmOptions,
        stack: Stack,
    ) -> Self {
        let mut globals = globals;
        let mut heap = heap;

        builtins::register(&mut globals, &mut heap);

        Self {
            executor,
            frames,
            globals,
            heap,
            locations,
            package_index,
            options,
            stack,
        }
    }

    ///
    ///
    ///
    pub fn new_with(
        executor: E,
        package_index: Option<PackageIndex>,
        options: Option<VmOptions>,
    ) -> Self {
        // Override options, if provided.
        let mut state = VmState::default();
        if let Some(options) = options {
            state.options = options;
        }

        Self::new_with_state(executor, package_index, state)
    }

    ///
    ///
    ///
    pub fn new_with_state(
        executor: E,
        package_index: Option<PackageIndex>,
        state: VmState,
    ) -> Self {
        let package_index = package_index.unwrap_or_default();
        let mut heap = Heap::default();

        Self::new(
            executor,
            Default::default(),
            state.get_globals(&mut heap),
            heap,
            Default::default(),
            package_index,
            state.options,
            Stack::default(),
        )
    }

    ///
    ///
    ///
    pub fn capture_state(&self) -> VmState {
        let mut globals = FnvHashMap::default();
        for (name, slot) in &self.globals {
            let value = (*slot).into_value(&self.heap);
            globals.insert(name.clone(), value);
        }

        VmState::new(globals, self.options.clone())
    }

    ///
    ///
    ///
    pub async fn main(
        &mut self,
        function: FunctionMut,
    ) {
        if !self.frames.is_empty() || !self.stack.is_empty() {
            panic!("VM not in a state to accept main function.");
        }

        let function = Object::Function(function.freeze(&mut self.heap));
        let handle = self.heap.insert(function).into_handle();

        self.stack.push_object(handle);
        self.call(0).await;
        self.run().await;

        // For REPLs
        if self.options.clear_after_main {
            self.frames.pop();
            self.stack.pop();
        }
    }

    ///
    ///
    ///
    pub async fn anonymous(
        &mut self,
        function: FunctionMut,
    ) -> Value {
        if function.arity != 0 {
            panic!("Not a nullary function.");
        }

        self.options.global_return_halts = true;

        let function = Object::Function(function.freeze(&mut self.heap));
        let handle = self.heap.insert(function).into_handle();

        self.stack.push_object(handle);
        self.call(0).await;
        self.run().await;

        if self.stack.len() == 1 {
            self.stack.pop().into_value(&self.heap)
        } else {
            Value::Unit
        }
    }

    ///
    ///
    ///
    async fn call(
        &mut self,
        arity: u8,
    ) {
        let frame_last = self.stack.len();
        let frame_first = frame_last - (arity + 1) as usize;

        let function = self.stack.get(frame_first).as_object().expect("");
        if let Some(Object::Function(_f)) = self.heap.get(function) {
            self.executor
                .debug(_f.chunk.disassemble().unwrap().to_string())
                .await
                .unwrap();

            // Position 0 is the main function, never allow it as root for a nested call frame.
            let frame = CallFrame::new(function, max(frame_first, 1));
            self.frames.push(frame);

            return;
        }

        panic!("illegal");
    }

    ///
    ///
    ///
    async fn run(&mut self) -> Option<Slot> {
        while let Some(instruction) = self.next() {
            match *instruction {
                OP_ADD => self.op_add(),
                OP_AND => self.op_and(),
                OP_ARRAY => self.op_array(),
                OP_CALL => self.op_call().await,
                OP_CLASS => self.op_class(),
                OP_CONSTANT => self.op_constant(),
                OP_DEFINE_GLOBAL => self.op_define_global(),
                OP_DIVIDE => self.op_divide(),
                OP_DOT => self.op_dot(),
                OP_EQUAL => self.op_equal(),
                OP_FALSE => self.op_false(),
                OP_GET_GLOBAL => self.op_get_global(),
                OP_GET_LOCAL => self.op_get_local(),
                OP_GET_METHOD => self.op_get_method(),
                OP_GET_PROPERTY => self.op_get_property(),
                OP_GREATER => self.op_greater(),
                OP_IMPORT => self.op_import(),
                OP_INDEX => self.op_index(),
                OP_JUMP => self.op_jump(),
                OP_JUMP_BACK => self.op_jump_back(),
                OP_JUMP_IF_FALSE => self.op_jump_if_false(),
                OP_LESS => self.op_less(),
                OP_LOC => self.op_loc(),
                OP_LOC_POP => self.op_loc_pop(),
                OP_LOC_PUSH => self.op_loc_push(),
                OP_MULTIPLY => self.op_multiply(),
                OP_NEGATE => self.op_negate(),
                OP_NEW => self.op_new(),
                OP_NOT => self.op_not(),
                OP_OR => self.op_or(),
                OP_PARALLEL => self.op_parallel(),
                OP_POP => self.op_pop(),
                OP_POP_N => self.op_pop_n(),
                OP_RETURN => {
                    self.op_return();

                    if self.options.global_return_halts && self.frames.is_empty() {
                        break;
                    }
                }
                OP_SET_GLOBAL => self.op_set_global(false),
                OP_SET_LOCAL => self.op_set_local(),
                OP_SUBSTRACT => self.op_substract(),
                OP_TRUE => self.op_true(),
                OP_UNIT => self.op_unit(),
                x => {
                    println!("Unkown opcode: {}", x);
                    unreachable!();
                }
            }

            // INVESTIGATE: this appears to cause a deadlock (?).
            // debug!("Sending stack to client.");
            // self.executor.debug(format!("{}", self.stack)).await.unwrap();
            // debug!("Sent stack to client.");
        }

        debug!("No more instructions to process within this call frame.");

        None
    }

    ///
    ///
    ///
    #[inline]
    fn next(&mut self) -> Option<&u8> {
        self.frame().read_u8()
    }

    ///
    ///
    ///
    fn arguments(
        &mut self,
        arity: u8,
    ) -> Vec<Value> {
        let mut arguments: Vec<Value> = (0..arity).map(|_| self.stack.pop().into_value(&self.heap)).collect();
        arguments.reverse();

        arguments
    }

    ///
    ///
    ///
    #[inline]
    fn frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("")
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_add(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        match (lhs, rhs) {
            (Slot::Integer(lhs), Slot::Integer(rhs)) => self.stack.push_integer(lhs + rhs),
            (Slot::Integer(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs as f64 + rhs),
            (Slot::Real(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs + rhs),
            (Slot::Real(lhs), Slot::Integer(rhs)) => self.stack.push_real(lhs + rhs as f64),
            (Slot::Object(lsh), Slot::Object(rhs)) => {
                let lhs = self.heap.get(lsh).unwrap().as_string();
                let rhs = self.heap.get(rhs).unwrap().as_string();

                match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => {
                        let mut new = lhs.clone();
                        new.push_str(rhs);

                        let object = self.heap.insert(Object::String(new));
                        let object = object.into_handle();

                        self.stack.push_object(object);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_and(&mut self) {
        let rhs = self.stack.pop_boolean();
        let lhs = self.stack.pop_boolean();

        self.stack.push_boolean(lhs && rhs);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_array(&mut self) {
        let n = *self.frame().read_u8().expect("");
        let elements: Vec<Slot> = (0..n).map(|_| self.stack.pop()).rev().collect();

        let array = Object::Array(Array::new(elements));
        let handle = self.heap.insert(array).into_handle();

        self.stack.push(Slot::Object(handle));
    }

    ///
    ///
    ///
    #[inline]
    pub async fn op_call(&mut self) {
        let arity = *self.frame().read_u8().expect("");
        let frame_last = self.stack.len();
        let frame_first = frame_last - (arity + 1) as usize;

        let function = self.stack.get(frame_first);
        let location = self
            .locations
            .last()
            .map(|l| self.heap.get(l).unwrap())
            .map(|l| l.as_string().cloned().unwrap());

        let value = match function {
            Slot::BuiltIn(code) => {
                let function = *code;
                let arguments = self.arguments(arity);

                builtins::call(function, arguments, &self.executor, location).await
            }
            Slot::Object(handle) => match self.heap.get(handle).expect("") {
                Object::Function(_) => {
                    // Execution is handled through call frames.
                    self.call(arity).await;
                    return;
                }
                Object::FunctionExt(f) => {
                    let function = f.clone();
                    let arguments = self.arguments(arity);

                    let arguments = itertools::zip(&function.parameters, arguments)
                        .map(|(p, a)| (p.name.clone(), a))
                        .collect();

                    let function_name = function.name.clone();
                    match self.executor.call(function, arguments, location).await {
                        Ok(value) => {
                            debug!("Value from function '{}' (external): \n{:#?}", function_name, value);
                            value
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            panic!("External function failed");
                        }
                    }
                }
                object => {
                    dbg!(&object);
                    dbg!(&self.stack);
                    panic!("Not a callable object");
                }
            },
            _ => panic!("Not a callable object"),
        };

        // Remove (built-in or external) function from the stack.
        self.stack.pop();

        // Store return value on the stack.
        let slot = Slot::from_value(value, &self.globals, &mut self.heap);
        self.stack.push(slot);

        debug!("Completed call to op_call.");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_class(&mut self) {
        let class = *self.frame().read_constant().expect("");
        self.stack.push(class);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_constant(&mut self) {
        let constant = *self.frame().read_constant().expect("Failed to read constant");

        self.stack.push(constant);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_define_global(&mut self) {
        self.op_set_global(true);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_divide(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        match (lhs, rhs) {
            (Slot::Integer(lhs), Slot::Integer(rhs)) => self.stack.push_integer(lhs / rhs),
            (Slot::Integer(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs as f64 / rhs),
            (Slot::Real(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs / rhs),
            (Slot::Real(lhs), Slot::Integer(rhs)) => self.stack.push_real(lhs / rhs as f64),
            _ => unreachable!(),
        };
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_dot(&mut self) {
        let instance = self.stack.pop().as_object().expect("expecting object.");
        let property = self
            .frame()
            .read_constant()
            .expect("expecting constant.")
            .as_object()
            .expect("expecting object.");

        if let Some(Object::Instance(instance)) = self.heap.get(instance) {
            if let Some(Object::String(property)) = self.heap.get(property) {
                let value = *instance.properties.get(property).expect("expecting property.");
                self.stack.push(value);

                return;
            }
        }

        panic!("invalid");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_equal(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        self.stack.push_boolean(lhs == rhs);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_false(&mut self) {
        self.stack.push(Slot::False);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_get_global(&mut self) {
        let identifier = *self.frame().read_constant().expect("Failed to read constant.");

        if let Slot::Object(handle) = identifier {
            if let Some(Object::String(identifier)) = self.heap.get(handle) {
                let value = *self
                    .globals
                    .get(identifier)
                    .unwrap_or_else(|| panic!("Failed to retreive global: {}", identifier));
                self.stack.push(value);

                return;
            }
        }

        panic!("Illegal identifier");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_get_local(&mut self) {
        let index = *self.frame().read_u8().expect("Failed to read byte.");
        let index = self.frame().stack_offset + index as usize;

        self.stack.copy_push(index);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_get_method(&mut self) {
        let instance_slot = self.stack.pop();
        let instance = instance_slot.as_object().expect("expecting object.");
        let method = self
            .frame()
            .read_constant()
            .expect("expecting constant.")
            .as_object()
            .expect("expecting object.");

        if let Some(Object::Instance(instance)) = self.heap.get(instance) {
            if let Some(Object::String(method)) = self.heap.get(method) {
                if let Some(Object::Class(class)) = self.heap.get(instance.class) {
                    let method = if class.name == *"Service" {
                        match method.as_str() {
                            // Quickfix :(
                            "waitUntilStarted" => Slot::BuiltIn(0x02),
                            "waitUntilDone" => Slot::BuiltIn(0x03),
                            _ => panic!("expecting method."),
                        }
                    } else {
                        *class.methods.get(method).expect("expecting method.")
                    };

                    self.stack.push(method);
                    self.stack.push(instance_slot);

                    return;
                }
            }
        }

        panic!("invalid");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_get_property(&mut self) {
        let instance = self.stack.pop().as_object().expect("expecting object.");
        let property = self
            .frame()
            .read_constant()
            .expect("expecting constant.")
            .as_object()
            .expect("expecting object.");

        if let Some(Object::Instance(instance)) = self.heap.get(instance) {
            if let Some(Object::String(property)) = self.heap.get(property) {
                let value = *instance.properties.get(property).expect("expecting property.");
                self.stack.push(value);

                return;
            }
        }

        panic!("invalid");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_greater(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        self.stack.push_boolean(lhs > rhs);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_import(&mut self) {
        let p_name = self.frame().read_constant().expect("").as_object().expect("");

        if let Some(Object::String(p_name)) = self.heap.get(p_name) {
            let p_name = p_name.clone();
            let package = self.package_index.get(&p_name, None).expect("");

            // TODO: update upstream so we don't need this anymore.
            let kind = match package.kind.as_str() {
                "ecu" => String::from("code"),
                "oas" => String::from("oas"),
                _ => unreachable!(),
            };

            if let Some(functions) = &package.functions {
                for (f_name, function) in functions {
                    let function = FunctionExt {
                        name: f_name.clone(),
                        detached: package.detached,
                        package: p_name.clone(),
                        kind: kind.clone(),
                        version: package.version.clone(),
                        parameters: function.parameters.clone(),
                    };

                    let handle = self.heap.insert(Object::FunctionExt(function)).into_handle();
                    let object = Slot::Object(handle);

                    self.globals.insert(f_name.clone(), object);
                }
            }

            if let Some(types) = &package.types {
                for t_name in types.keys() {
                    let class = Class {
                        name: t_name.clone(),
                        methods: Default::default(),
                    };

                    let handle = self.heap.insert(Object::Class(class)).into_handle();
                    let object = Slot::Object(handle);

                    self.globals.insert(t_name.clone(), object);
                }
            }
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_index(&mut self) {
        let index = self.stack.pop_integer();
        let array = self.stack.pop_object();

        if let Some(Object::Array(array)) = self.heap.get(array) {
            if let Some(element) = array.elements.get(index as usize) {
                self.stack.push(*element);
                return;
            }
        }

        panic!("invalid index.");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_jump(&mut self) {
        let offset = self.frame().read_u16();
        self.frame().ip += offset as usize;
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_jump_back(&mut self) {
        let offset = self.frame().read_u16();
        self.frame().ip -= offset as usize;
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_jump_if_false(&mut self) {
        let truthy = self.stack.peek_boolean();
        if !truthy {
            self.op_jump();
        } else {
            self.frame().ip += 2;
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_less(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        self.stack.push_boolean(lhs < rhs);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_loc(&mut self) {
        let location = self.locations.pop().map(Slot::Object).unwrap_or(Slot::Unit);

        self.stack.push(location);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_loc_pop(&mut self) {
        self.locations.pop();
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_loc_push(&mut self) {
        let location = self.stack.pop_object();
        self.locations.push(location);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_multiply(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        match (lhs, rhs) {
            (Slot::Integer(lhs), Slot::Integer(rhs)) => self.stack.push_integer(lhs * rhs),
            (Slot::Integer(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs as f64 * rhs),
            (Slot::Real(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs * rhs),
            (Slot::Real(lhs), Slot::Integer(rhs)) => self.stack.push_real(lhs * rhs as f64),
            _ => unreachable!(),
        };
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_negate(&mut self) {
        let value = self.stack.pop();

        let value = match value {
            Slot::Integer(i) => Slot::Integer(-i),
            Slot::Real(r) => Slot::Real(-r),
            _ => panic!("expecting a integer or real value."),
        };

        self.stack.push(value);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_new(&mut self) {
        let properties_n = *self.frame().read_u8().expect("");
        let class = self.stack.pop().as_object().expect("expecting object");

        let mut properties = FnvHashMap::default();
        (0..properties_n).for_each(|_| {
            let ident = self.stack.pop().as_object().expect("expecting object");
            let value = self.stack.pop();

            if let Some(Object::String(ident)) = self.heap.get(ident) {
                properties.insert(ident.clone(), value);
            } else {
                panic!("Invalid property identifier.");
            }
        });

        if let Some(Object::Class(_)) = self.heap.get(class) {
            let instance = Instance::new(class, properties);
            let instance = self.heap.insert(Object::Instance(instance)).into_handle();

            self.stack.push_object(instance);
            return;
        }

        panic!("Invalid");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_not(&mut self) {
        let value = self.stack.pop_boolean();
        self.stack.push_boolean(!value);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_or(&mut self) {
        let rhs = self.stack.pop_boolean();
        let lhs = self.stack.pop_boolean();

        self.stack.push_boolean(lhs || rhs);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_parallel(&mut self) {
        let branches_n = *self.frame().read_u8().expect("");
        let mut branches: Vec<FunctionMut> = Vec::new();

        // TODO: combine op_parallel with op_array.
        for _ in 0..branches_n {
            let handle = self.stack.pop_object();
            let function = self.heap.get(handle).expect("").as_function().expect("").clone();

            let function = function.unfreeze(&self.heap);
            branches.push(function);
        }

        let results = if branches.is_empty() {
            Array::new(vec![])
        } else {
            let executor = self.executor.clone();
            let package_index = self.package_index.clone();
            let state = self.capture_state();

            let results = branches
                .into_par_iter()
                .map(|f| {
                    let mut vm = Vm::<E>::new_with_state(executor.clone(), Some(package_index.clone()), state.clone());

                    // TEMP: needed because the VM is not completely `send`.
                    let rt = Runtime::new().unwrap();
                    rt.block_on(vm.anonymous(f))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|v| Slot::from_value(v, &self.globals, &mut self.heap))
                .collect();

            Array::new(results)
        };

        let array = Object::Array(results);
        let array = self.heap.insert(array).into_handle();

        self.stack.push_object(array);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_pop(&mut self) {
        self.stack.pop();
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_pop_n(&mut self) {
        let x = *self.frame().read_u8().expect("");

        let index = self.stack.len() - x as usize;
        self.stack.clear_from(index);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_return(&mut self) {
        if self.frames.len() == 1 && !self.options.global_return_halts {
            panic!("Cannot return outside a function.");
        }

        if let Some(frame) = self.frames.pop() {
            let return_value = self.stack.try_pop();
            self.stack.clear_from(frame.stack_offset);
            self.stack.try_push(return_value);
        }
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_set_global(
        &mut self,
        create_if_not_exists: bool,
    ) {
        let identifier = *self.frame().read_constant().expect("Failed to read constant.");

        let value = self.stack.pop();

        if let Slot::Object(handle) = identifier {
            if let Some(Object::String(identifier)) = self.heap.get(handle) {
                if create_if_not_exists || self.globals.contains_key(identifier) {
                    self.globals.insert(identifier.clone(), value);
                } else {
                    panic!("Variable '{}' not found.", identifier);
                }

                return;
            }
        }

        panic!("Illegal identifier");
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_set_local(&mut self) {
        let index = *self.frame().read_u8().expect("Failed to read byte.");
        let index = self.frame().stack_offset + index as usize;

        self.stack.copy_pop(index);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_substract(&mut self) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();

        match (lhs, rhs) {
            (Slot::Integer(lhs), Slot::Integer(rhs)) => self.stack.push_integer(lhs - rhs),
            (Slot::Integer(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs as f64 - rhs),
            (Slot::Real(lhs), Slot::Real(rhs)) => self.stack.push_real(lhs - rhs),
            (Slot::Real(lhs), Slot::Integer(rhs)) => self.stack.push_real(lhs - rhs as f64),
            _ => unreachable!(),
        };
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_true(&mut self) {
        self.stack.push(Slot::True);
    }

    ///
    ///
    ///
    #[inline]
    pub fn op_unit(&mut self) {
        self.stack.push(Slot::Unit);
    }
}
