use crate::functions::{self, FunctionPattern};
use crate::indexes::PackageIndex;
use crate::parser::{self, AstNode, AstPredicate, AstRelation};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use semver::Version;
use specifications::common::{Property, Type, Value, Variable};
use specifications::instructions::*;

type Map<T> = std::collections::HashMap<String, T>;

pub struct CompilerOptions {
    pub return_call: bool,
}

impl CompilerOptions {
    ///
    ///
    ///
    pub fn default() -> Self {
        CompilerOptions { return_call: false }
    }

    ///
    ///
    ///
    pub fn repl() -> Self {
        CompilerOptions { return_call: true }
    }
}

pub struct CompilerState {
    pub imports: Vec<FunctionPattern>,
    pub variables: Map<String>,
    pub types: Map<Type>,
}

pub struct Compiler {
    pub options: CompilerOptions,
    pub package_index: PackageIndex,
    pub state: CompilerState,
}

impl Compiler {
    ///
    ///
    ///
    pub fn new(
        options: CompilerOptions,
        package_index: PackageIndex,
    ) -> Result<Self> {
        let state = CompilerState {
            imports: vec![],
            variables: Map::<String>::new(),
            types: Map::<Type>::new(),
        };

        Ok(Compiler {
            options,
            package_index,
            state,
        })
    }

    ///
    ///
    ///
    pub fn quick_compile(
        package_index: PackageIndex,
        input: &str,
    ) -> Result<Vec<Instruction>> {
        let mut compiler = Compiler::new(CompilerOptions::default(), package_index)?;
        compiler.compile(input)
    }

    ///
    ///
    ///
    pub fn compile(
        &mut self,
        input: &str,
    ) -> Result<Vec<Instruction>> {
        let ast = parser::parse(input)?;
        let mut instructions = vec![];

        for node in ast {
            let (variable, instruction) = match node {
                AstNode::Assignment { name, expr } => self.handle_assignment_node(name, *expr)?,
                AstNode::Call { terms } => {
                    if self.options.return_call {
                        self.handle_assignment_call_node(String::from("terminate"), terms)?
                    } else {
                        self.handle_call_node(terms)?
                    }
                }
                AstNode::Condition {
                    predicate,
                    if_exec,
                    el_exec,
                } => self.handle_condition_node(predicate, *if_exec, el_exec.map(|e| *e))?,
                AstNode::Import { module, version } => self.handle_import_node(module, version)?,
                AstNode::Parameter { name, complex } => self.handle_parameter_node(name, complex)?,
                AstNode::Terminate { terms } => self.handle_terminate_node(terms)?,
                AstNode::WaitUntil { predicate } => self.handle_wait_until_node(predicate)?,
                AstNode::WhileLoop { predicate, exec } => self.handle_while_loop_node(predicate, *exec)?,
                AstNode::Literal { .. } | AstNode::Word { .. } => {
                    debug!("Encountered standalone literal or word.");
                    (None, None)
                }
            };

            // Variable bookkeeping
            if let Some(variable) = variable {
                self.state.variables.insert(variable.name, variable.data_type);
            }
            if let Some(instruction) = instruction {
                instructions.push(instruction);
            }
        }

        Ok(instructions)
    }

    pub fn inject(
        &mut self,
        variables: Map<String>,
    ) -> () {
        self.state.variables.extend(variables);
    }

    ///
    ///
    ///
    fn handle_assignment_node(
        &mut self,
        name: String,
        expr: AstNode,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        debug!("Handling assignment node: {:?}", expr);

        match expr {
            AstNode::Call { terms } => self.handle_assignment_call_node(name, terms),
            AstNode::Literal { value } => self.handle_assignment_literal_node(name, value),
            _ => unreachable!()
        }
    }

    ///
    ///
    ///
    fn handle_assignment_call_node(
        &mut self,
        name: String,
        terms: Vec<AstNode>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let (instructions, data_type) = terms_to_instructions(terms, Some(name.clone()), &self.state)?;
        let subroutine = SubInstruction::new(instructions, Default::default());

        let variable = Variable::new(name, data_type, None, None);

        Ok((Some(variable), Some(subroutine)))
    }

    ///
    ///
    ///
    fn handle_assignment_literal_node(
        &mut self,
        name: String,
        value: Value,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let data_type = value.data_type().to_string();
        let data_type = if data_type == "??[]" {
            if let Value::Array { entries, .. } = &value {
                if let Some(Value::Pointer { variable, .. }) = entries.first() {
                    let element_type = self.state.variables.get(variable).expect(&format!("Not a valid variable pointer from array: {}.", variable));
                    format!("{}[]", element_type)
                } else {
                    unreachable!();
                }
            } else {
                unreachable!();
            }
        } else {
            data_type.clone()
        };

        let value = if let Value::Struct { data_type, properties } = value {
            if let Some(c_type) = self.state.types.get(&data_type) {
                let mut resolved_properties = Map::<Value>::new();

                for property in &c_type.properties {
                    ensure!(
                        properties.get(&property.name).is_some(),
                        "Missing '{}' in {} object.",
                        property.name,
                        data_type
                    );

                    let actual_property = properties.get(&property.name).unwrap();
                    let actual_data_type = if let Value::Pointer { variable, data_type, .. } = actual_property {
                        if data_type == "??" {
                            if let Some(ref_data_typed) = self.state.variables.get(variable) {
                                ref_data_typed.clone()
                            } else {
                                bail!("Cannot find variable '{}'", variable);
                            }
                        } else {
                            data_type.clone()
                        }
                    } else {
                        actual_property.data_type().to_string()
                    };

                    ensure!(
                        actual_data_type == property.data_type,
                        "Mismatch in datatype '{}' should be {} but is {}.",
                        property.name,
                        property.data_type,
                        actual_data_type
                    );

                    let resolved_value = if let Value::Pointer { variable, secret, .. } = actual_property {
                        Value::Pointer { variable: variable.clone(), secret: secret.clone(), data_type: actual_data_type }
                    } else {
                        actual_property.clone()
                    };

                    resolved_properties.insert(property.name.clone(), resolved_value);
                }

                ensure!(
                    properties.len() == c_type.properties.len(),
                    "Mismatch in number of actual and expected properties."
                );

                Value::Struct { data_type, properties: resolved_properties }
            } else {
                bail!(
                    "Cannot find type information for {}. If it is custom type, please bring it into scope.",
                    data_type
                );
            }
        } else {
            value
        };

        let variable = Variable::new(name, data_type.to_string(), None, Some(value.clone()));
        let instruction = VarInstruction::new(Default::default(), vec![variable.clone()], Default::default());

        Ok((Some(variable), Some(instruction)))
    }    

    ///
    ///
    ///
    fn handle_call_node(
        &mut self,
        terms: Vec<AstNode>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let (instructions, _) = terms_to_instructions(terms, None, &self.state)?;
        let subroutine = SubInstruction::new(instructions, Default::default());

        Ok((None, Some(subroutine)))
    }

    ///
    ///
    ///
    fn handle_condition_node(
        &mut self,
        predicate: AstPredicate,
        if_exec: AstNode,
        el_exec: Option<AstNode>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        debug!("{:?}", predicate);

        let (poll, condition) = match predicate {
            AstPredicate::Call { terms } => {
                let (variable, poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms })?;
                let condition = Condition::eq(variable.unwrap().as_pointer(), Value::Boolean(true));

                (poll.unwrap(), condition)
            }
            AstPredicate::Comparison {
                lhs_terms,
                relation,
                rhs_terms,
            } => {
                let (lhs_var, lhs_poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms: lhs_terms })?;
                let (rhs_var, rhs_poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms: rhs_terms })?;
                let poll = SubInstruction::new(vec![lhs_poll.unwrap(), rhs_poll.unwrap()], Default::default());

                let condition = match relation {
                    AstRelation::Equals => Condition::eq(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::NotEquals => {
                        Condition::ne(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                    AstRelation::Greater => Condition::gt(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::Less => Condition::lt(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::GreaterOrEqual => {
                        Condition::ge(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                    AstRelation::LessOrEqual => {
                        Condition::le(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                };

                (poll, condition)
            }
        };

        let if_check = MovInstruction::new(
            vec![condition.clone()],
            vec![Move::Forward, Move::Skip],
            Default::default(),
        );

        let (_, if_exec) = match if_exec {
            AstNode::Assignment { name, expr } => self.handle_assignment_node(name, *expr)?,
            AstNode::Call { terms } => self.handle_call_node(terms)?,
            AstNode::Terminate { terms } => self.handle_terminate_node(terms)?,
            _ => unreachable!(),
        };

        let instruction = if let Some(el_exec) = el_exec {
            let (_, el_exec) = match el_exec {
                AstNode::Assignment { name, expr } => self.handle_assignment_node(name, *expr)?,
                AstNode::Call { terms } => self.handle_call_node(terms)?,
                AstNode::Terminate { terms } => self.handle_terminate_node(terms)?,
                _ => unreachable!(),
            };

            let el_check = MovInstruction::new(
                vec![condition.clone()],
                vec![Move::Skip, Move::Forward],
                Default::default(),
            );
            SubInstruction::new(
                vec![poll, if_check, if_exec.unwrap(), el_check, el_exec.unwrap()],
                Default::default(),
            )
        } else {
            SubInstruction::new(vec![poll, if_check, if_exec.unwrap()], Default::default())
        };

        Ok((None, Some(instruction)))
    }

    ///
    ///
    ///
    fn handle_import_node(
        &mut self,
        package: String,
        version: Option<Version>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let package_info = self.package_index.get(&package, version.as_ref());

        if let Some(package_info) = package_info {
            let package_patterns = functions::get_module_patterns(package_info)?;
            self.state.imports.extend(package_patterns);
            self.state
                .imports
                .sort_by(|a, b| a.parameters.len().partial_cmp(&b.parameters.len()).unwrap());
            self.state.imports.reverse();

            if let Some(types) = &package_info.types {
                for (n, t) in types.iter() {
                    self.state.types.insert(n.clone(), t.clone());
                }
            }
        } else {
            return Err(anyhow!("No package found with name: {}", package));
        }

        Ok((None, None))
    }

    ///
    ///
    ///
    fn handle_parameter_node(
        &mut self,
        name: String,
        data_type: String,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let data_type = match data_type.as_str() {
            "Boolean" => "boolean",
            "Integer" => "integer",
            "Decimal" => "real",
            "String" => "string",
            _ => data_type.as_str(),
        };

        let variable = Variable::new(name, String::from(data_type), None, None);
        let instruction = VarInstruction::new(vec![variable.clone()], Default::default(), Default::default());

        Ok((Some(variable), Some(instruction)))
    }

    ///
    ///
    ///
    fn handle_wait_until_node(
        &mut self,
        predicate: AstPredicate,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let (poll, condition) = match predicate {
            AstPredicate::Call { terms } => {
                let (variable, poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms })?;
                let condition = Condition::eq(variable.unwrap().as_pointer(), Value::Boolean(true));

                (poll.unwrap(), condition)
            }
            AstPredicate::Comparison {
                lhs_terms,
                relation,
                rhs_terms,
            } => {
                let (lhs_var, lhs_poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms: lhs_terms })?;
                let (rhs_var, rhs_poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms: rhs_terms })?;
                let poll = SubInstruction::new(vec![lhs_poll.unwrap(), rhs_poll.unwrap()], Default::default());

                let condition = match relation {
                    AstRelation::Equals => Condition::eq(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::NotEquals => {
                        Condition::ne(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                    AstRelation::Greater => Condition::gt(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::Less => Condition::lt(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::GreaterOrEqual => {
                        Condition::ge(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                    AstRelation::LessOrEqual => {
                        Condition::le(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                };

                (poll, condition)
            }
        };

        let mut check_meta = Map::<String>::new();
        check_meta.insert(String::from("sleep_after_false"), String::from("10s"));

        let check = MovInstruction::new(vec![condition], vec![Move::Forward, Move::Backward], check_meta);
        let instruction = SubInstruction::new(vec![poll, check], Default::default());

        Ok((None, Some(instruction)))
    }

    ///
    ///
    ///
    fn handle_while_loop_node(
        &mut self,
        predicate: AstPredicate,
        exec: AstNode,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let (poll, condition) = match predicate {
            AstPredicate::Call { terms } => {
                let (variable, poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms })?;
                let condition = Condition::eq(variable.unwrap().as_pointer(), Value::Boolean(true));

                (poll.unwrap(), condition)
            }
            AstPredicate::Comparison {
                lhs_terms,
                relation,
                rhs_terms,
            } => {
                let (lhs_var, lhs_poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms: lhs_terms })?;
                let (rhs_var, rhs_poll) = self.handle_assignment_node(create_temp_var(false), AstNode::Call { terms: rhs_terms })?;
                let poll = SubInstruction::new(vec![lhs_poll.unwrap(), rhs_poll.unwrap()], Default::default());

                let condition = match relation {
                    AstRelation::Equals => Condition::eq(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::NotEquals => {
                        Condition::ne(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                    AstRelation::Greater => Condition::gt(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::Less => Condition::lt(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer()),
                    AstRelation::GreaterOrEqual => {
                        Condition::ge(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                    AstRelation::LessOrEqual => {
                        Condition::le(lhs_var.unwrap().as_pointer(), rhs_var.unwrap().as_pointer())
                    }
                };

                (poll, condition)
            }
        };

        let (_, exec) = match exec {
            AstNode::Assignment { name, expr } => self.handle_assignment_node(name, *expr)?,
            AstNode::Call { terms } => self.handle_call_node(terms)?,
            AstNode::Terminate { terms } => self.handle_terminate_node(terms)?,
            _ => unreachable!(),
        };

        let check_before = MovInstruction::new(
            vec![condition.clone()],
            vec![Move::Forward, Move::Skip],
            Default::default(),
        );
        let check_after = MovInstruction::new(
            vec![condition.clone()],
            vec![Move::Backward, Move::Forward],
            Default::default(),
        );

        let exec_and_poll = SubInstruction::new(vec![exec.unwrap(), poll.clone()], Default::default());
        let instruction = SubInstruction::new(vec![poll, check_before, exec_and_poll, check_after], Default::default());

        Ok((None, Some(instruction)))
    }

    ///
    ///
    ///
    fn handle_terminate_node(
        &mut self,
        terms: Option<Vec<AstNode>>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        debug!("Terminate: {:?}", terms);

        // Always set a variable called 'terminate' in the local scope.
        let (instructions, _) = self.set_terminate_variable_locally(terms)?;
        let subroutine = SubInstruction::new(instructions, Default::default());

        Ok((None, Some(subroutine)))
    }

    ///
    ///
    ///
    fn set_terminate_variable_locally(
        &mut self,
        terms: Option<Vec<AstNode>>,
    ) -> Result<(Vec<Instruction>, String)> {
        let terminate = "terminate".to_string();

        if let Some(terms) = terms {
            // If the term is an existing variable, set that variable as terminate variable.
            if terms.len() == 1 {
                if let AstNode::Word { text: name } = &terms[0] {
                    if let Some(data_type) = self.state.variables.get(name) {
                        let value = Value::Pointer {
                            variable: name.clone(),
                            data_type: data_type.clone(),
                            secret: false,
                        };
                        let variable =
                            Variable::new(terminate, data_type.clone(), Some(String::from("output")), Some(value));
                        let instruction = VarInstruction::new(Default::default(), vec![variable], Default::default());

                        return Ok((vec![instruction], data_type.to_string()));
                    }
                }
            }

            // Otherwise, set output from call as terminate variable.
            Ok(terms_to_instructions(terms, Some(terminate), &self.state)?)
        } else {
            // Set empty variable in case of no return terms.
            Ok((vec![], String::from("unit")))
        }
    }
}

///
///
///
pub fn terms_to_instructions(
    terms: Vec<AstNode>,
    result_var: Option<String>,
    state: &CompilerState,
) -> Result<(Vec<Instruction>, String)> {
    let mut variables = state.variables.clone();
    let functions = state.imports.clone();
    let original_terms = terms.clone();

    debug!("Terms: {:?}", terms);

    let mut instructions: Vec<Instruction> = vec![];
    let mut return_data_type = String::from("unit");

    // Check if is variable assignment
    if terms.len() == 1 && result_var.is_some() {
        if let Some(AstNode::Word { text: name }) = terms.first() {
            if let Some(data_type) = variables.get(name) {
                // TODO: create set
                let pointer = Value::Pointer { data_type: data_type.clone(), variable: name.clone(), secret: false };
                let set = vec![Variable::new(result_var.unwrap(), data_type.clone(), None, Some(pointer))];
                let instruction = VarInstruction::new(vec!(), set, Default::default());
                instructions.push(instruction);
                
                return Ok((instructions, data_type.clone()))
            }
        }
    }

    // Replace literals in terms with names
    let mut literals = Map::<Value>::new();
    let terms: Vec<AstNode> = terms
        .iter()
        .map(|t| match t {
            AstNode::Literal { value } => {
                let temp_var = create_temp_var(true);
                literals.insert(temp_var.clone(), value.clone());
                variables.insert(temp_var.clone(), value.data_type().to_string());

                AstNode::Word { text: temp_var }
            }
            _ => t.clone()
        })
        .collect();

    debug!("Terms: {:?}", terms);
    debug!("Variables: {:?}", variables);
    debug!("Literals: {:?}", literals);

    // This regex will match consecutive value placeholders (e.g. <integer>), if
    // the start and end of the regex capture group corresponds to the start and
    // end of the target pattern, then all unknown terms are eliminated.
    let placeholders_regex = Regex::new(r"(?:(?:<(?:[\.\w]+):(?:[a-zA-Z]+(?:\[\])*)>))+").unwrap();

    // Same as above, but just match one, so we can iterate over placeholders.
    let placeholder_regex = Regex::new(r"<(?:([\.\w]+)):([a-zA-Z]+(?:\[\])*)>").unwrap();

    // We use temporary variables to hold values between function calls. If we
    // consume a temporary variable, we can directly reuse it again.
    let mut temp_vars: Vec<String> = vec![];

    let mut term_pattern = build_terms_pattern(&terms, &variables, &state.types)?;
    println!("{:?}", term_pattern);

    loop {
        let term_pattern_clone = term_pattern.clone();
        debug!("Pattern: {:?}", &term_pattern_clone);

        if let Some(coverage) = placeholders_regex.find(&term_pattern_clone) {
            if coverage.start() == 0 && coverage.end() == term_pattern_clone.len() {
                debug!("Done: no more unkowns to eliminate.");
                println!("DONE");
                break;
            }
        }

        let mut one_plus_matches = false;
        for function in &functions {
            debug!("Check: {:?}", &function.pattern);

            // Look for function pattern (needle) in remaining terms (haystack).
            let needle = Regex::new(&function.pattern).unwrap();

            if let Some(find_location) = needle.find(&term_pattern_clone) {
                let range = find_location.start()..find_location.end();
                let coverage = term_pattern_clone.get(range.clone()).unwrap();

                debug!("Parameters: {:?}", function.parameters);

                let mut input = Map::<Value>::new();
                let mut parameters = function.parameters.iter();
                let mut consumed_temp: Option<String> = None;

                // Map each placeholder to function parameters
                for ph in placeholder_regex.captures_iter(coverage) {
                    debug!("Placeholder: {:?}", ph);

                    if let Some(group) = ph.get(1) {
                        let arg = group.as_str().to_string();
                        if temp_vars.contains(&arg) {
                            consumed_temp = Some(arg.clone());
                        }

                        if let Some(parameter) = parameters.next() {
                            let value = if let Some(value) = literals.get(&arg) {
                                value.clone()
                            } else {
                                Value::Pointer {
                                    variable: arg,
                                    data_type: parameter.data_type.clone(),
                                    secret: false,
                                }
                            };

                            debug!("Input: {:?} <- {:?}", &parameter.name, value);

                            input.insert(parameter.name.clone(), value);
                        } else {
                            unreachable!();
                        }
                    }
                }

                // Add implicit arguments to input (secrets)
                function.parameters.iter().filter(|p| p.secret.is_some()).for_each(|p| {
                    let pointer = Value::Pointer {
                        variable: p.secret.as_ref().unwrap().clone(),
                        data_type: p.data_type.clone(),
                        secret: true,
                    };

                    input.insert(p.name.clone(), pointer);
                });

                debug!("Input: {:?}", &input);

                // Decide if we can reuse consumed temp variable, or create a new one.
                let temp_var = if let Some(temp_var) = consumed_temp {
                    temp_var
                } else {
                    let new_temp_var = create_temp_var(false);
                    temp_vars.push(new_temp_var.clone());

                    new_temp_var
                };

                // Replace part of the term pattern with new temp variable placeholder.
                let segment = format!("<{}:{}>", temp_var, &function.return_type);
                term_pattern.replace_range(range, segment.as_str());

                // Construct ACT instruction
                let instruction = ActInstruction::new(
                    function.name.clone(),
                    input,
                    Some(temp_var),
                    Some(function.return_type.to_string()),
                    function.meta.clone(),
                );
                instructions.push(instruction);

                // Rewind loop
                one_plus_matches = true;
                break;
            }
        }

        if !one_plus_matches {
            let mut sb = String::new();
            sb.push_str("Failed to match terms of statement:\n\n   ");
            for term in &original_terms {
                sb.push_str(&format!("{} ", term));
            }
            sb.push_str("\n");

            bail!(sb);
        }
    }

    // Modify assignment of last ACT instruction, if specified.
    if let Some(result_var) = result_var {
        println!("{:#?}", terms);

        ensure!(!instructions.is_empty(), "Created no ACT instructions.");

        match instructions.remove(instructions.len() - 1) {
            Instruction::Act(act) => {
                let updated =
                    ActInstruction::new(act.name, act.input, Some(result_var), act.data_type.clone(), act.meta);

                if let Some(data_type) = act.data_type {
                    return_data_type = data_type
                }

                instructions.push(updated);
            }
            _ => unreachable!(),
        }
    }

    Ok((instructions, return_data_type))
}

///
///
///
fn build_terms_pattern(
    terms: &Vec<AstNode>,
    variables: &Map<String>,
    types: &Map<Type>,
) -> Result<String> {
    let mut term_pattern_segments = vec![];
    for term in terms {
        match term {
            AstNode::Word { text: name } => {
                if name.contains('.') {
                    let segments: Vec<_> = name.split('.').collect();
                    if let Some(arch_type) = variables.get(segments[0]) {
                        if arch_type.ends_with("[]") && segments[1] == "length" {
                            let segment = format!("<{}:{}>", name, String::from("integer"));
                            term_pattern_segments.push(segment);
                            continue;
                        }

                        debug!("Resolving {} within type {}", name, arch_type);
                        if let Some(arch_type) = types.get(arch_type) {
                            // TODO: use hashmap in Type struct
                            let mut properties = Map::<Property>::new();
                            for p in &arch_type.properties {
                                properties.insert(p.name.clone(), p.clone());
                            }

                            if let Some(p) = properties.get(segments[1]) {
                                let segment = format!("<{}:{}>", name, p.data_type);
                                term_pattern_segments.push(segment);

                                continue;
                            }
                        }
                    }
                } else if let Some(data_type) = variables.get(name) {
                    let segment = format!("<{}:{}>", name, data_type);
                    term_pattern_segments.push(segment);

                    continue;
                }

                // If the above attempt(s) faile, just add the name.
                term_pattern_segments.push(name.to_string());
            }
            AstNode::Literal { value } => {
                let segment = format!("<{}>", value.data_type());
                term_pattern_segments.push(segment);
            },
            _ => unreachable!()
        }
    }

    let term_pattern = term_pattern_segments.join(" ");
    Ok(term_pattern)
}

///
///
///
fn create_temp_var(literal: bool) -> String {
    let random_name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .collect::<String>()
        .to_lowercase();

    if literal {
        random_name
    } else {
        format!("_{}", random_name)
    }
}
