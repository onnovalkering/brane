use std::{collections::HashMap, usize};
use anyhow::Result;
use specifications::package::PackageIndex;
use crate::{CallFrame, bytecode::Function, values::{Array, Instance, Value}};

///
///
///
#[inline]
pub fn op_constant(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(constant) = frame.chunk.code.get(ip) {
        if let Some(value) = frame.chunk.constants.get(*constant as usize) {
            stack.push(value.clone());

            return Ok(ip + 1);
        }
    }

    bail!("unreachable");
}


///
///
///
#[inline]
pub fn op_get_local(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(index) = frame.chunk.code.get(ip) {
        let index = frame.slot_offset + *index as usize;
        let local = stack.get(index).unwrap().clone();
        stack.push(local);

        return Ok(ip + 1);
    }

    bail!("unreachable");
}

///
///
///
#[inline]
pub fn op_set_local(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(index) = frame.chunk.code.get(ip) {
        let value = stack.pop().unwrap();
        stack[frame.slot_offset + *index as usize] = value;

        return Ok(ip + 1);
    }

    bail!("unreachable");
}

///
///
///
#[inline]
pub fn op_define_global(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
    state: &mut HashMap<String, Value>,
) -> Result<usize> {
    if let Some(ident) = frame.chunk.code.get(ip) {
        if let Some(ident) = frame.chunk.constants.get(*ident as usize) {
            let value = stack.pop().unwrap();

            if let Value::String(ident) = ident {
                state.insert(ident.clone(), value);

                return Ok(ip + 1);
            }
        }
    }

    bail!("unreachable");
}

///
///
///
#[inline]
pub fn op_get_global(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
    state: &mut HashMap<String, Value>,
) -> Result<usize> {
    if let Some(ident) = frame.chunk.code.get(ip) {
        if let Some(ident) = frame.chunk.constants.get(*ident as usize) {
            if let Value::String(ident) = ident {
                if let Some(value) = state.get(ident) {
                    stack.push(value.clone());

                    return Ok(ip + 1);
                } else {
                    bail!("Tried to access undefined variable: {:?}", ident);
                }
            }
        }
    }

    bail!("unreachable");
}


///
///
///
#[inline]
pub fn op_set_global(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
    state: &mut HashMap<String, Value>,
) -> Result<usize> {
    if let Some(ident) = frame.chunk.code.get(ip) {
        if let Some(ident) = frame.chunk.constants.get(*ident as usize) {
            let value = stack.pop().unwrap();

            if let Value::String(ident) = ident {
                state.insert(ident.clone(), value);

                return Ok(ip + 1);
            }
        } else {
            bail!("Tried to assign to undefined variable: {:?}", ident);
        }
    }

    bail!("unreachable");
}

///
///
///
#[inline]
pub fn op_class(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(class) = frame.chunk.code.get(ip) {
        if let Some(value) = frame.chunk.constants.get(*class as usize) {
            stack.push(value.clone());

            return Ok(ip + 1);
        }
    }

    bail!("unreachable");
}

///
///
///
#[inline]
pub fn op_add(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs + rhs).into(),
            (Value::Real(lhs), Value::Real(rhs)) => (lhs + rhs).into(),
            (Value::Real(lhs), Value::Integer(rhs)) => (lhs + rhs as f64).into(),
            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 + rhs).into(),
            (Value::String(lhs), Value::String(rhs)) => (format!("{}{}", lhs, rhs)).into(),
            _ => {
                bail!("unreachable");
            }
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_substract(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs - rhs).into(),
            (Value::Real(lhs), Value::Real(rhs)) => (lhs - rhs).into(),
            (Value::Real(lhs), Value::Integer(rhs)) => (lhs - rhs as f64).into(),
            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 - rhs).into(),
            _ => bail!("unreachable"),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_multiply(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs * rhs).into(),
            (Value::Real(lhs), Value::Real(rhs)) => (lhs * rhs).into(),
            (Value::Real(lhs), Value::Integer(rhs)) => (lhs * rhs as f64).into(),
            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 * rhs).into(),
            _ => bail!("unreachable"),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_divide(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs / rhs).into(),
            (Value::Real(lhs), Value::Real(rhs)) => (lhs / rhs).into(),
            (Value::Real(lhs), Value::Integer(rhs)) => (lhs / rhs as f64).into(),
            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 / rhs).into(),
            _ => bail!("unreachable"),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_negate(
    stack: &mut Vec<Value>,
) -> Result<()> {
    if let Some(value) = stack.pop() {
        match value {
            Value::Integer(i) => stack.push((-i).into()),
            Value::Real(r) => stack.push((-r).into()),
            _ => bail!("unreachable"),
        }
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_true(
    stack: &mut Vec<Value>,
) -> Result<()> {
    stack.push(Value::Boolean(true));

    Ok(())
}

///
///
///
#[inline]
pub fn op_false(
    stack: &mut Vec<Value>,
) -> Result<()> {
    stack.push(Value::Boolean(false));

    Ok(())
}

///
///
///
#[inline]
pub fn op_unit(
    stack: &mut Vec<Value>,
) -> Result<()> {
    stack.push(Value::Unit);

    Ok(())
}

///
///
///
#[inline]
pub fn op_not(
    stack: &mut Vec<Value>,
) -> Result<()> {
    if let Some(value) = stack.pop() {
        match value {
            Value::Boolean(b) => stack.push(Value::Boolean(!b)),
            Value::Unit => stack.push(Value::Boolean(true)),
            _ => bail!("unreachable"),
        }
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_and(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Boolean(lhs), Value::Boolean(rhs)) => (lhs & rhs).into(),
            _ => Value::Boolean(false),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_or(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs | rhs),
            _ => Value::Boolean(false),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_equal(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs == rhs).into(),
            (Value::Real(lhs), Value::Real(rhs)) => (lhs == rhs).into(),
            (Value::Boolean(lhs), Value::Boolean(rhs)) => (lhs == rhs).into(),
            (Value::Unit, Value::Unit) => true.into(),
            (Value::String(lhs), Value::String(rhs)) => (lhs == rhs).into(),
            _ => false.into(),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_greater(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs > rhs).into(),
            (Value::Real(lhs), Value::Real(rhs)) => (lhs > rhs).into(),
            (Value::Real(lhs), Value::Integer(rhs)) => (lhs > rhs as f64).into(),
            (Value::Integer(lhs), Value::Real(rhs)) => (lhs as f64 > rhs).into(),
            _ => bail!("unreachable"),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_less(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let rhs = stack.pop();
    let lhs = stack.pop();

    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let value = match (lhs, rhs) {
            (Value::Integer(lhs), Value::Integer(rhs)) => (lhs < rhs).into(),
            (Value::Real(lhs), Value::Real(rhs)) => (lhs < rhs).into(),
            (Value::Real(lhs), Value::Integer(rhs)) => (lhs < rhs as f64).into(),
            (Value::Integer(lhs), Value::Real(rhs)) => ((lhs as f64) < rhs).into(),
            _ => bail!("unreachable"),
        };

        stack.push(value);
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_pop(
    stack: &mut Vec<Value>,
) -> Result<()> {
    stack.pop();

    Ok(())
}

///
///
///
#[inline]
pub fn op_return(
    stack: &mut Vec<Value>,
    call_frames: &mut Vec<CallFrame>,
) -> Result<Value> {
    let value = stack.pop().unwrap();
    call_frames.pop();

    Ok(value)
}

///
///
///
#[inline]
pub fn op_loc_push(
    stack: &mut Vec<Value>,
    locations: &mut Vec<String>,
) -> Result<()> {
    if let Some(Value::String(location)) = stack.pop() {
        locations.push(location);
    } else {
        bail!("Location must be a string.");
    }

    Ok(())
}

///
///
///
#[inline]
pub fn op_loc_pop(
    locations: &mut Vec<String>,
) -> Result<()> {
    locations.pop();

    Ok(())
}

///
///
///
#[inline]
pub fn op_jump(
    ip: usize,
    frame: &CallFrame,
) -> Result<usize> {
    match (frame.chunk.code.get(ip), frame.chunk.code.get(ip + 1)) {
        (Some(offset1), Some(offset2)) => {
            let offset = (((*offset1 as u16) << 8) | (*offset2 as u16)) as usize;

            Ok(ip + 2usize + offset)
        },
        _ => {
            bail!("unreachable!");
        }
    }
}

///
///
///
#[inline]
pub fn op_jump_back(
    ip: usize,
    frame: &CallFrame,
) -> Result<usize> {
    match (frame.chunk.code.get(ip), frame.chunk.code.get(ip + 1)) {
        (Some(offset1), Some(offset2)) => {
            let offset = (((*offset1 as u16) << 8) | (*offset2 as u16)) as usize;

            Ok((ip + 2usize) - offset)
        },
        _ => {
            bail!("unreachable!");
        }
    }
}

///
///
///
#[inline]
pub fn op_jump_if_false(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(Value::Boolean(false)) = stack.last() {
        op_jump(ip, frame)
    } else {
        Ok(ip + 2)
    }
}

///
///
///
#[inline]
pub fn op_import(
    ip: usize,
    frame: &CallFrame,
    state: &mut HashMap<String, Value>,
    package_index: &PackageIndex,
) -> Result<usize> {
    if let Some(constant) = frame.chunk.code.get(ip) {
        if let Some(Value::String(package_name)) = frame.chunk.constants.get(*constant as usize) {
            if let Some(package) = package_index.get(package_name, None) {
                let kind = match package.kind.as_str() {
                    "ecu" => String::from("code"),
                    "oas" => String::from("oas"),
                    _ => unreachable!(),
                };

                if let Some(functions) = &package.functions {
                    for (name, function) in functions {
                        state.insert(
                            name.clone(),
                            Value::Function(Function::External {
                                package: package_name.clone(),
                                version: package.version.clone(),
                                kind: kind.clone(),
                                name: name.clone(),
                                parameters: function.parameters.clone(),
                            }),
                        );
                    }

                    return Ok(ip + 1);
                }
            }
        }
    }

    bail!("unreachable");
}

///
///
///
#[inline]
pub fn op_new(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(properties_n) = frame.chunk.code.get(ip) {
        let class = stack.pop();
        let mut properties = HashMap::new();

        (0..*properties_n).for_each(|_| {
            let ident = stack.pop().unwrap();
            let value = stack.pop().unwrap();

            if let Value::String(ident) = ident {
                properties.insert(ident, value);
            }
        });

        if let Some(Value::Class(class)) = class {
            let instance = Instance::new(class, Some(properties));
            stack.push(Value::Instance(instance));
        } else {
            bail!("Not a class.");
        }

        Ok(ip + 1)
    } else {
        bail!("unreachable");
    }
}

///
///
///
#[inline]
pub fn op_array(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(entries_n) = frame.chunk.code.get(ip) {
        let entries: Vec<Value> = (0..*entries_n).map(|_| stack.pop().unwrap()).rev().collect();

        if entries.is_empty() {
            stack.push(Value::Array(Array {
                data_type: String::from("unit[]"),
                entries,
            }));
        } else {
            let data_type = match entries.get(0).unwrap() {
                Value::String(_) => String::from("string"),
                Value::Real(_) => String::from("real"),
                Value::Integer(_) => String::from("integer"),
                Value::Boolean(_) => String::from("boolean"),
                Value::Array(array) => array.data_type.clone(),
                Value::Instance(instance) => instance.class.name.clone(),
                Value::Class(_) | Value::Function(_) => todo!(),
                Value::Unit => String::from("unit"),
            };

            let data_type = format!("{}[]", data_type);
            stack.push(Value::Array(Array { data_type, entries }));
        }

        Ok(ip + 1)
    } else {
        bail!("unreachable");
    }
}

///
///
///
#[inline]
pub fn op_dot(
    ip: usize,
    frame: &CallFrame,
    stack: &mut Vec<Value>,
) -> Result<usize> {
    if let Some(property) = frame.chunk.code.get(ip) {
        let property = if let Some(Value::String(property)) = frame.chunk.constants.get(*property as usize) {
            property.clone()
        } else {
            bail!("constant not found!");
        };

        if let Some(Value::Instance(instance)) = stack.pop() {
            if let Some(property) = instance.fields.get(&property) {
                stack.push(property.clone());
            } else {
                bail!("Property not found!");
            }
        } else {
            bail!("Not an instance!");
        }

        Ok(ip + 1)
    } else {
        bail!("unreachable");
    }
}

///
///
///
#[inline]
pub fn op_index(
    stack: &mut Vec<Value>,
) -> Result<()> {
    let index = stack.pop().expect("Empty stack while expecting `index` value.");
    let array = stack.pop().expect("Empty stack while expecting `array` value.");

    if let Value::Integer(index) = index {
        if let Value::Array(array) = array {
            let entries = array.entries;
            if let Some(entry) = entries.get(index as usize) {
                stack.push(entry.clone());

                return Ok(());
            }
        }
    }

    bail!("unreachable");
}
