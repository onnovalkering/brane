use crate::functions::{self, FunctionPattern};
use crate::indexes::PackageIndex;
use crate::parser::{self, AstNode, AstTerm};
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use semver::Version;
use specifications::common::{Argument, Value};
use specifications::instructions::Instruction;

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

pub struct CompilerOptions {}

impl CompilerOptions {
    pub fn none() -> Self {
        CompilerOptions {}
    }
}

pub struct CompilerState {
    pub imports: Vec<FunctionPattern>,
    pub variables: Map<String>,
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
    ) -> FResult<Self> {
        let state = CompilerState {
            imports: vec![],
            variables: Map::<String>::new(),
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
        input: &String,
    ) -> FResult<Vec<Instruction>> {
        let mut compiler = Compiler::new(CompilerOptions::none(), package_index)?;
        compiler.compile(input)
    }

    ///
    ///
    ///
    pub fn compile(
        &mut self,
        input: &String,
    ) -> FResult<Vec<Instruction>> {
        let ast = parser::parse(input)?;
        let mut instructions = vec![];

        use AstNode::*;
        for node in ast {
            let (variable, instruction) = match node {
                Assignment { name, terms } => self.handle_assignment_node(name, terms)?,
                Call { terms } => self.handle_call_node(terms)?,
                Parameter { name, complex } => self.handle_parameter_node(name, complex)?,
                Repeat { predicate, exec } => self.handle_repeat_node(*predicate, *exec)?,
                Terminate { terms } => self.handle_terminate_node(terms)?,
                Import { module, version } => self.handle_import_node(module, version)?,
                _ => unreachable!(),
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

    ///
    ///
    ///
    fn handle_assignment_node(
        &mut self,
        name: String,
        terms: Vec<AstTerm>,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        if terms.len() == 1 && terms[0].is_value() {
            self.handle_assignment_value_node(name, &terms[0])
        } else {
            self.handle_assignment_call_node(name, terms)
        }
    }

    ///
    ///
    ///
    fn handle_assignment_value_node(
        &mut self,
        name: String,
        value: &AstTerm,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        let value = if let AstTerm::Value(value) = value {
            value.clone()
        } else {
            unreachable!()
        };

        let data_type = value.get_complex().to_string();

        let instruction = Instruction::new_set_var(name.clone(), value, String::from("local"));
        let argument = Argument::new(name, data_type, None, None, None, None, None);

        Ok((Some(argument), Some(instruction)))
    }

    ///
    ///
    ///
    fn handle_assignment_call_node(
        &mut self,
        name: String,
        terms: Vec<AstTerm>,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        let (instructions, data_type) = terms_to_instructions(terms, Some(name.clone()), &self.state)?;
        let subroutine = Instruction::new_sub(instructions);

        let var = Argument::new(name, data_type, None, None, None, None, None);

        Ok((Some(var), Some(subroutine)))
    }

    ///
    ///
    ///
    fn handle_call_node(
        &mut self,
        terms: Vec<AstTerm>,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        let (instructions, _) = terms_to_instructions(terms, None, &self.state)?;
        let subroutine = Instruction::new_sub(instructions);

        Ok((None, Some(subroutine)))
    }

    ///
    ///
    ///
    fn handle_import_node(
        &mut self,
        module: String,
        version: Option<Version>,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        let package_info = self.package_index.get(&module, version.as_ref());
        if let Some(package_info) = package_info {
            let package_patterns = functions::get_module_patterns(package_info)?;
            self.state.imports.extend(package_patterns);
        } else {
        }

        Ok((None, None))
    }

    ///
    ///
    ///
    fn handle_parameter_node(
        &mut self,
        name: String,
        complex: String,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        let data_type = match complex.as_str() {
            "Boolean" => "boolean",
            "Integer" => "integer",
            "Decimal" => "real",
            "String" => "string",
            _ => complex.as_str(),
        };

        let instruction = Instruction::new_get_var(name.clone(), data_type.to_string());
        let argument = Argument::new(name, data_type.to_string(), None, None, None, None, None);

        Ok((Some(argument), Some(instruction)))
    }

    ///
    ///
    ///
    fn handle_repeat_node(
        &mut self,
        _predicate: AstNode,
        _exec: AstNode,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        unimplemented!();
    }

    ///
    ///
    ///
    fn handle_terminate_node(
        &mut self,
        terms: Option<Vec<AstTerm>>,
    ) -> FResult<(Option<Argument>, Option<Instruction>)> {
        debug!("Terminate: {:?}", terms);

        // Always set a variable called 'terminate' in the local scope.
        let (mut instructions, _) = self.set_terminate_variable_locally(terms)?;

        // Return terminate in output scope.
        instructions.push(Instruction::new_set_var(
            "terminate".to_string(),
            Value::None,
            "output".to_string(),
        ));

        let subroutine = Instruction::new_sub(instructions);

        Ok((None, Some(subroutine)))
    }

    ///
    ///
    ///
    fn set_terminate_variable_locally(
        &mut self,
        terms: Option<Vec<AstTerm>>,
    ) -> FResult<(Vec<Instruction>, String)> {
        let terminate = "terminate".to_string();

        if let Some(terms) = terms {
            // If the term is an existing variable, set that variable as terminate variable.
            if terms.len() == 1 {
                if let AstTerm::Name(name) = &terms[0] {
                    if let Some(data_type) = self.state.variables.get(name) {
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
            Ok(terms_to_instructions(terms, Some(terminate), &mut self.state)?)
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
    terms: Vec<AstTerm>,
    result_var: Option<String>,
    state: &CompilerState,
) -> FResult<(Vec<Instruction>, String)> {
    let variables = state.variables.clone();
    let functions = state.imports.clone();

    debug!("Variables: {:?}", &variables);

    let mut term_pattern = build_terms_pattern(terms, &variables)?;

    // This regex will match consecutive value placeholders (e.g. <integer>), if
    // the start and end of the regex capture group corresponds to the start and
    // end of the target pattern, then all unknown terms are eliminated.
    let placeholders_regex = Regex::new(r"(?:(?:<(?:\w+):(?:[a-zA-Z]+(?:\[\])*)>))+").unwrap();

    // Same as above, but just match one, so we can iterate over placeholders.
    let placeholder_regex = Regex::new(r"<(?:(\w+)):([a-zA-Z]+(?:\[\])*)>").unwrap();

    // We use temporary variables to hold values between function calls. If we
    // consume a temporary variable, we can directly reuse it again.
    let mut temp_vars: Vec<String> = vec![];

    let mut instructions: Vec<Instruction> = vec![];
    let mut return_data_type = String::from("unit");

    loop {
        let term_pattern_clone = term_pattern.clone();
        debug!("Pattern: {:?}", &term_pattern_clone);

        if let Some(coverage) = placeholders_regex.find(&term_pattern_clone) {
            if coverage.start() == 0 && coverage.end() == term_pattern_clone.len() {
                debug!("Done: no more unkowns to eliminate.");
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

                let mut input = Map::<Value>::new();
                let mut arguments = function.arguments.iter();
                let mut consumed_temp: Option<String> = None;

                // Map each placeholder to function parameters
                for ph in placeholder_regex.captures_iter(coverage) {
                    if let Some(group) = ph.get(1) {
                        let arg = group.as_str().to_string();
                        if temp_vars.contains(&arg) {
                            consumed_temp = Some(arg.clone());
                        }

                        if let Some(argument) = arguments.next() {
                            debug!("Input: {:?} <- {:?}", &argument.name, &arg);

                            input.insert(
                                argument.name.clone(),
                                Value::Variable(arg),
                            );
                        } else {
                            unreachable!();
                        }
                    }
                }

                // Decide if we can reuse consumed temp variable, or create a new one.
                let temp_var = if let Some(temp_var) = consumed_temp {
                    temp_var
                } else {
                    let new_temp_var = create_temp_var();
                    temp_vars.push(new_temp_var.clone());

                    new_temp_var
                };

                // Replace part of the term pattern with new temp variable placeholder.
                let segment = format!("<{}:{}>", temp_var, &function.return_type);
                term_pattern.replace_range(range, segment.as_str());

                // Construct ACT instruction
                let instruction = Instruction::new_act(
                    function.name.clone(),
                    input,
                    function.meta.clone(),
                    Some(temp_var),
                    Some(function.return_type.to_string()),
                );
                instructions.push(instruction);

                // Rewind loop
                one_plus_matches = true;
                break;
            }
        }

        ensure!(one_plus_matches, "Failed to match");
    }

    // Modify assignment of last ACT instruction, if specified.
    if let Some(result_var) = result_var {
        ensure!(instructions.len() > 0, "Created no ACT instructions.");

        match instructions.remove(instructions.len() - 1) {
            Instruction::Act {
                assignment: _,
                input,
                meta,
                name,
                r#type: _,
                data_type,
            } => {
                let updated = Instruction::new_act(name, input, meta, Some(result_var), data_type.clone());

                if let Some(data_type) = data_type {
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
    terms: Vec<AstTerm>,
    variables: &Map<String>,
) -> FResult<String> {
    let mut term_pattern_segments = vec![];
    for term in &terms {
        match term {
            AstTerm::Name(name) => {
                if let Some(complex) = variables.get(name) {
                    let segment = format!("<{}:{}>", name, complex);
                    term_pattern_segments.push(segment);
                } else {
                    term_pattern_segments.push(name.to_string());
                }
            }
            AstTerm::Symbol(symbol) => {
                term_pattern_segments.push(symbol.to_string());
            }
            AstTerm::Value(value) => {
                let segment = format!("<{}>", value.get_complex());
                term_pattern_segments.push(segment);
            }
        }
    }

    let term_pattern = term_pattern_segments.join(" ");
    Ok(term_pattern)
}

///
///
///
fn create_temp_var() -> String {
    let random_name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .collect::<String>()
        .to_lowercase();

    format!("_{}", random_name)
}
