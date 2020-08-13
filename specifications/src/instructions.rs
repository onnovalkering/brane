use crate::common::{Value, Variable};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

type Map<T> = std::collections::HashMap<String, T>;

#[skip_serializing_none]
#[serde(tag = "variant", rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Instruction {
    Act(ActInstruction),
    Mov(MovInstruction),
    Sub(SubInstruction),
    Var(VarInstruction),
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActInstruction {
    pub assignment: Option<String>,
    pub data_type: Option<String>,
    pub input: Map<Value>,
    pub meta: Map<String>,
    pub name: String,
}

impl ActInstruction {
    ///
    ///
    ///
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        input: Map<Value>,
        assignment: Option<String>,
        data_type: Option<String>,
        meta: Map<String>,
    ) -> Instruction {
        let act = ActInstruction {
            meta,
            assignment,
            name,
            input,
            data_type,
        };

        Instruction::Act(act)
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MovInstruction {
    pub branches: Vec<Move>,
    pub conditions: Vec<Condition>,
    pub meta: Map<String>,
}

impl MovInstruction {
    ///
    ///
    ///
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        conditions: Vec<Condition>,
        branches: Vec<Move>,
        meta: Map<String>,
    ) -> Instruction {
        let mov = MovInstruction {
            branches,
            conditions,
            meta,
        };

        Instruction::Mov(mov)
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Condition {
    pub left: Value,
    pub operator: Operator,
    pub right: Value,
}

impl Condition {
    ///
    ///
    ///
    pub fn eq(
        left: Value,
        right: Value,
    ) -> Condition {
        Condition {
            left,
            operator: Operator::Equals,
            right,
        }
    }

    ///
    ///
    ///
    pub fn ne(
        left: Value,
        right: Value,
    ) -> Condition {
        Condition {
            left,
            operator: Operator::NotEquals,
            right,
        }
    }

    ///
    ///
    ///
    pub fn gt(
        left: Value,
        right: Value,
    ) -> Condition {
        Condition {
            left,
            operator: Operator::Greater,
            right,
        }
    }

    ///
    ///
    ///
    pub fn lt(
        left: Value,
        right: Value,
    ) -> Condition {
        Condition {
            left,
            operator: Operator::Less,
            right,
        }
    }

    ///
    ///
    ///
    pub fn ge(
        left: Value,
        right: Value,
    ) -> Condition {
        Condition {
            left,
            operator: Operator::GreaterOrEqual,
            right,
        }
    }

    ///
    ///
    ///
    pub fn le(
        left: Value,
        right: Value,
    ) -> Condition {
        Condition {
            left,
            operator: Operator::LessOrEqual,
            right,
        }
    }
}

#[repr(u8)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Move {
    Backward = 1,
    Forward = 2,
    Skip = 3,
}

#[repr(u8)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Operator {
    Equals = 1,
    NotEquals = 2,
    Greater = 3,
    Less = 4,
    GreaterOrEqual = 5,
    LessOrEqual = 6,
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubInstruction {
    pub instructions: Vec<Instruction>,
    pub meta: Map<String>,
}

impl SubInstruction {
    ///
    ///
    ///
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        instructions: Vec<Instruction>,
        meta: Map<String>,
    ) -> Instruction {
        let sub = SubInstruction { instructions, meta };

        Instruction::Sub(sub)
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VarInstruction {
    pub get: Vec<Variable>,
    pub meta: Map<String>,
    pub set: Vec<Variable>,
}

impl VarInstruction {
    ///
    ///
    ///
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        get: Vec<Variable>,
        set: Vec<Variable>,
        meta: Map<String>,
    ) -> Instruction {
        let var = VarInstruction { get, meta, set };

        Instruction::Var(var)
    }
}
