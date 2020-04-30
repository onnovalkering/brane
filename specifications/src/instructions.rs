use crate::common::{Argument, FunctionNotation, Type, Value};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use serde_yaml;
use std::fs;
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;
type FResult<T> = Result<T, failure::Error>;

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Instructions {
    pub functions: Vec<Function>,
    pub meta: InstructionsMeta,
    pub types: Option<Map<Type>>,
}

impl Instructions {
    pub fn from_path(path: PathBuf) -> FResult<Instructions> {
        let contents = fs::read_to_string(path)?;

        Instructions::from_string(contents)
    }

    pub fn from_string(contents: String) -> FResult<Instructions> {
        let result = serde_yaml::from_str(&contents)?;

        Ok(result)
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstructionsMeta {
    pub description: Option<String>,
    pub name: String,
    pub version: String,
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    pub description: Option<String>,
    pub instructions: Vec<Instruction>,
    pub name: String,
    pub notation: Option<FunctionNotation>,
}

#[skip_serializing_none]
#[serde(untagged, rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Instruction {
    Act {
        meta: Map<String>,
        r#type: String,
        assignment: Option<String>,
        name: String,
        input: Map<Argument>,
        data_type: Option<String>,
    },
    Mov {
        meta: Map<String>,
        r#type: String,
        conditions: Vec<Condition>,
        branches: Vec<Move>,
    },
    Var {
        meta: Map<String>,
        r#type: String,
        get: Vec<Variable>,
        set: Vec<Variable>,
    },
    Sub {
        meta: Map<String>,
        r#type: String,
        instructions: Vec<Instruction>,
    },
}

impl Instruction {
    pub fn new_get_var(name: String, data_type: String) -> Instruction {
        let variable = Variable {
            name,
            value: Value::None,
            data_type,
            description: None,
            scope: "input".to_string(),
        };

        let get = vec![variable];
        let set = vec![];

        Instruction::new_var(get, set)
    }

    pub fn new_set_var(name: String, value: Value, scope: String) -> Instruction {
        let data_type = value.get_complex();

        let variable = Variable {
            name,
            value,
            data_type,
            description: None,
            scope,
        };

        let get = vec![];
        let set = vec![variable];

        Instruction::new_var(get, set)
    }    

    pub fn new_var(get: Vec<Variable>, set: Vec<Variable>) -> Instruction {
        Instruction::Var {
            get,
            set,
            meta: Map::<String>::new(),
            r#type: "VAR".to_string(),
        }
    }

    pub fn new_act(
        name: String,
        input: Map<Argument>,
        meta: Map<String>,
        assignment: Option<String>,
        data_type: Option<String>,
    ) -> Instruction {
        Instruction::Act {
            name,
            input,
            assignment,
            meta,
            r#type: "ACT".to_string(),
            data_type,
        }
    }

    pub fn new_sub(instructions: Vec<Instruction>) -> Instruction {
        Instruction::Sub {
            r#type: "SUB".to_string(),
            meta: Map::<String>::new(),
            instructions,
        }
    }
}

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    #[serde(rename = "type")]
    pub data_type: String,
    pub description: Option<String>,
    pub name: String,
    pub scope: String,
    pub value: Value,
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Condition {
    left: String,
    operator: Operator,
    right: String,
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
    Higher = 3,
    Less = 4,
    HigherOrEqual = 5,
    LowerOrEqual = 6,
}
