use crate::functions::{self, FunctionPattern};
use crate::indexes::PackageIndex;
use crate::parser::{self, AstNode, AstTerm};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use semver::Version;
use specifications::common::{Value, Variable, Type, Property};
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

        use AstNode::*;
        for node in ast {
            let (variable, instruction) = match node {
                Assignment { name, terms } => self.handle_assignment_node(name, terms)?,
                Call { terms } => {
                    if self.options.return_call {
                        self.handle_assignment_call_node(String::from("terminate"), terms)?
                    } else {
                        self.handle_call_node(terms)?
                    }
                }
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
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        debug!("Handling assignment node: {:?}", terms);

        if terms.len() == 1 {
            match &terms[0] {
                AstTerm::Name(variable) => {
                    if self.state.variables.contains_key(variable) {
                        return self.handle_assignment_value_node(name, &terms[0]);
                    }
                }
                AstTerm::Value(_) => {
                    return self.handle_assignment_value_node(name, &terms[0]);
                }
                _ => unreachable!(),
            }
        }

        self.handle_assignment_call_node(name, terms)
    }

    ///
    ///
    ///
    fn handle_assignment_value_node(
        &mut self,
        name: String,
        value: &AstTerm,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let (value, data_type) = match value {
            AstTerm::Value(value) => (Some(value.clone()), value.data_type().to_string()),
            AstTerm::Name(variable) => {
                let data_type = self.state.variables.get(variable).unwrap();
                let value = Value::Pointer {
                    variable: variable.clone(),
                    data_type: data_type.clone(),
                };

                (Some(value.clone()), data_type.clone())
            }
            _ => unreachable!(),
        };

        let variable = Variable::new(name, data_type, None, value);
        let instruction = VarInstruction::new(Default::default(), vec![variable.clone()], Default::default());

        Ok((Some(variable), Some(instruction)))
    }

    ///
    ///
    ///
    fn handle_assignment_call_node(
        &mut self,
        name: String,
        terms: Vec<AstTerm>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let (instructions, data_type) = terms_to_instructions(terms, Some(name.clone()), &self.state)?;
        let subroutine = SubInstruction::new(instructions, Default::default());

        let variable = Variable::new(name, data_type, None, None);

        Ok((Some(variable), Some(subroutine)))
    }

    ///
    ///
    ///
    fn handle_call_node(
        &mut self,
        terms: Vec<AstTerm>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let (instructions, _) = terms_to_instructions(terms, None, &self.state)?;
        let subroutine = SubInstruction::new(instructions, Default::default());

        Ok((None, Some(subroutine)))
    }

    ///
    ///
    ///
    fn handle_import_node(
        &mut self,
        module: String,
        version: Option<Version>,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        let package_info = self.package_index.get(&module, version.as_ref());

        if let Some(package_info) = package_info {
            let package_patterns = functions::get_module_patterns(package_info)?;
            self.state.imports.extend(package_patterns);

            if let Some(types) = &package_info.types {
                for (n, t) in types.iter() {
                    self.state.types.insert(n.clone(), t.clone());
                }
            }
        } else {
            unreachable!();
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
    fn handle_repeat_node(
        &mut self,
        _predicate: AstNode,
        _exec: AstNode,
    ) -> Result<(Option<Variable>, Option<Instruction>)> {
        unimplemented!();
    }

    ///
    ///
    ///
    fn handle_terminate_node(
        &mut self,
        terms: Option<Vec<AstTerm>>,
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
        terms: Option<Vec<AstTerm>>,
    ) -> Result<(Vec<Instruction>, String)> {
        let terminate = "terminate".to_string();

        if let Some(terms) = terms {
            // If the term is an existing variable, set that variable as terminate variable.
            if terms.len() == 1 {
                if let AstTerm::Name(name) = &terms[0] {
                    if let Some(data_type) = self.state.variables.get(name) {
                        let value = Value::Pointer {
                            variable: name.clone(),
                            data_type: data_type.clone(),
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
    terms: Vec<AstTerm>,
    result_var: Option<String>,
    state: &CompilerState,
) -> Result<(Vec<Instruction>, String)> {
    let variables = state.variables.clone();
    let functions = state.imports.clone();

    debug!("Variables: {:?}", &variables);

    let mut term_pattern = build_terms_pattern(terms, &variables, &state.types)?;

    // This regex will match consecutive value placeholders (e.g. <integer>), if
    // the start and end of the regex capture group corresponds to the start and
    // end of the target pattern, then all unknown terms are eliminated.
    let placeholders_regex = Regex::new(r"(?:(?:<(?:[\.\w]+):(?:[a-zA-Z]+(?:\[\])*)>))+").unwrap();

    // Same as above, but just match one, so we can iterate over placeholders.
    let placeholder_regex = Regex::new(r"<(?:([\.\w]+)):([a-zA-Z]+(?:\[\])*)>").unwrap();

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
                let mut parameters = function.parameters.iter();
                let mut consumed_temp: Option<String> = None;

                // Map each placeholder to function parameters
                for ph in placeholder_regex.captures_iter(coverage) {
                    if let Some(group) = ph.get(1) {
                        let arg = group.as_str().to_string();
                        if temp_vars.contains(&arg) {
                            consumed_temp = Some(arg.clone());
                        }

                        if let Some(parameter) = parameters.next() {
                            debug!("Input: {:?} <- {:?}", &parameter.name, &arg);
                            input.insert(
                                parameter.name.clone(),
                                Value::Pointer {
                                    variable: arg,
                                    data_type: parameter.data_type.clone(),
                                },
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

        ensure!(one_plus_matches, "Failed to match");
    }

    // Modify assignment of last ACT instruction, if specified.
    if let Some(result_var) = result_var {
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
    terms: Vec<AstTerm>,
    variables: &Map<String>,
    types: &Map<Type>,
) -> Result<String> {
    let mut term_pattern_segments = vec![];
    for term in &terms {
        match term {
            AstTerm::Name(name) => {
                if name.contains(".") {
                    let segments: Vec<_> = name.split(".").collect();
                    if let Some(arch_type) = variables.get(segments[0]) {
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
                } else {
                    if let Some(data_type) = variables.get(name) {
                        let segment = format!("<{}:{}>", name, data_type);
                        term_pattern_segments.push(segment);

                        continue;
                    }
                }

                // If the above attempt(s) faile, just add the name.
                term_pattern_segments.push(name.to_string());
            }
            AstTerm::Symbol(symbol) => {
                term_pattern_segments.push(symbol.to_string());
            }
            AstTerm::Value(value) => {
                let segment = format!("<{}>", value.data_type());
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
