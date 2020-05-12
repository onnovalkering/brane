use crate::ast;
use crate::ast::{AstNode, AstTerm};
use crate::configuration::Configuration;
use crate::functions;
use crate::functions::FunctionPattern;
use crate::parser;
use crate::terms;
use specifications::common::{Argument, Value};
use specifications::instructions::Instruction;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn handle(
    input: String,
    config: &Configuration,
) -> FResult<Vec<Instruction>> {
    let mut functions = Vec::<FunctionPattern>::new();
    let mut variables = Map::<String>::new();

    let instructions = compile(input, &mut functions, &mut variables, config)?;
    Ok(instructions)
}

///
///
///
pub fn compile(
    input: String,
    functions: &mut Vec<FunctionPattern>,
    variables: &mut Map<String>,
    config: &Configuration,
) -> FResult<Vec<Instruction>> {
    let ast = parser::parse(&input)?;

    let (imports, ast) = separate_import_nodes(ast)?;
    let mut imported = functions::build_function_patterns(imports, config)?;

    functions.append(&mut imported);
    functions.sort();
    functions.dedup();

    let mut instructions = vec![];

    use AstNode::*;
    for node in ast {
        let (variable, instruction) = match node {
            Assignment { name, terms } => handle_assignment_node(name, terms, &variables, &functions)?,
            Call { terms } => handle_call_node(terms, &variables, &functions)?,
            Condition {
                predicate,
                if_exec,
                el_exec,
            } => {
                let el_exec = if let Some(el_exec) = el_exec {
                    Some(*el_exec)
                } else {
                    None
                };

                handle_condition_node(*predicate, *if_exec, el_exec)?
            }
            Parameter { name, complex } => handle_parameter_node(name, complex)?,
            Repeat { predicate, exec } => handle_repeat_node(*predicate, *exec)?,
            Terminate { terms } => handle_terminate_node(terms, &variables, &functions)?,
            _ => unreachable!(),
        };

        if let Some(variable) = variable {
            variables.insert(variable.name, variable.data_type);
        }

        instructions.push(instruction);
    }

    Ok(instructions)
}

///
///
///
fn separate_import_nodes(nodes: Vec<AstNode>) -> FResult<(Vec<AstNode>, Vec<AstNode>)> {
    let mut imports = vec![];
    let mut ast = vec![];

    for node in nodes {
        if ast::is_import(&node) {
            imports.push(node);
        } else {
            ast.push(node);
        }
    }

    Ok((imports, ast))
}

///
///
///
fn handle_assignment_node(
    name: String,
    terms: Vec<AstTerm>,
    variables: &Map<String>,
    functions: &Vec<FunctionPattern>,
) -> FResult<(Option<Argument>, Instruction)> {
    if terms.len() == 1 && ast::is_value(&terms[0]) {
        handle_assignment_value_node(name, &terms[0])
    } else {
        handle_assignment_call_node(name, terms, variables, functions)
    }
}

///
///
///
fn handle_assignment_value_node(
    name: String,
    value: &AstTerm,
) -> FResult<(Option<Argument>, Instruction)> {
    let value = if let AstTerm::Value(value) = value {
        value.clone()
    } else {
        unreachable!()
    };

    let data_type = value.get_complex();

    let instruction = Instruction::new_set_var(name.clone(), value, String::from("local"));
    let argument = Argument::new(name, data_type, None, None, None, None, None);

    Ok((Some(argument), instruction))
}

///
///
///
fn handle_assignment_call_node(
    name: String,
    terms: Vec<AstTerm>,
    variables: &Map<String>,
    functions: &Vec<FunctionPattern>,
) -> FResult<(Option<Argument>, Instruction)> {
    let (instructions, data_type) = terms::terms_to_instructions(terms, Some(name.clone()), variables, functions)?;
    let subroutine = Instruction::new_sub(instructions);

    let var = Argument::new(name, data_type, None, None, None, None, None);

    Ok((Some(var), subroutine))
}

///
///
///
fn handle_call_node(
    terms: Vec<AstTerm>,
    variables: &Map<String>,
    functions: &Vec<FunctionPattern>,
) -> FResult<(Option<Argument>, Instruction)> {
    let (instructions, _) = terms::terms_to_instructions(terms, None, variables, functions)?;
    let subroutine = Instruction::new_sub(instructions);

    Ok((None, subroutine))
}

///
///
///
fn handle_condition_node(
    _predicate: AstNode,
    _if_exec: AstNode,
    _el_exec: Option<AstNode>,
) -> FResult<(Option<Argument>, Instruction)> {
    unimplemented!();
}

///
///
///
fn handle_parameter_node(
    name: String,
    complex: String,
) -> FResult<(Option<Argument>, Instruction)> {
    let data_type = match complex.as_str() {
        "Boolean" => "boolean",
        "Integer" => "integer",
        "Decimal" => "real",
        "String" => "string",
        _ => complex.as_str(),
    };

    let instruction = Instruction::new_get_var(name.clone(), data_type.to_string());
    let argument = Argument::new(name, data_type.to_string(), None, None, None, None, None);

    Ok((Some(argument), instruction))
}

///
///
///
fn handle_repeat_node(
    _predicate: AstNode,
    _exec: AstNode,
) -> FResult<(Option<Argument>, Instruction)> {
    unimplemented!();
}

///
///
///
fn handle_terminate_node(
    terms: Option<Vec<AstTerm>>,
    variables: &Map<String>,
    functions: &Vec<FunctionPattern>,
) -> FResult<(Option<Argument>, Instruction)> {
    debug!("Terminate: {:?}", terms);

    // Always set a variable called 'terminate' in the local scope.
    let (mut instructions, data_type) = set_terminate_variable_locally(terms, variables, functions)?;

    // Return terminate in output scope.
    instructions.push(Instruction::new_set_var(
        "terminate".to_string(),
        Value::None,
        "output".to_string(),
    ));

    let subroutine = Instruction::new_sub(instructions);

    Ok((None, subroutine))
}

///
///
///
fn set_terminate_variable_locally(
    terms: Option<Vec<AstTerm>>,
    variables: &Map<String>,
    functions: &Vec<FunctionPattern>,
) -> FResult<(Vec<Instruction>, String)> {
    let terminate = "terminate".to_string();

    if let Some(terms) = terms {
        // If the term is an existing variable, set that variable as terminate variable.
        if terms.len() == 1 {
            if let AstTerm::Name(name) = &terms[0] {
                if let Some(data_type) = variables.get(name) {
                    return Ok((
                        vec![Instruction::new_set_var(
                            terminate,
                            Value::Variable(name.clone()),
                            "output".to_string(),
                        )],
                        data_type.to_string(),
                    ));
                }
            }
        }

        // Otherwise, set output from call as terminate variable.
        Ok(terms::terms_to_instructions(
            terms,
            Some(terminate),
            variables,
            functions,
        )?)
    } else {
        // Set empty variable in case of no return terms.
        Ok((vec![], String::from("unit")))
    }
}
