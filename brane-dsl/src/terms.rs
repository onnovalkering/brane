use crate::ast::AstTerm;
use crate::functions::FunctionPattern;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use specifications::common::Argument;
use specifications::instructions::Instruction;

type Map<T> = std::collections::HashMap<String, T>;
type FResult<T> = Result<T, failure::Error>;

///
///
///
pub fn terms_to_instructions(
    terms: Vec<AstTerm>,
    result_var: Option<String>,
    variables: &Map<String>,
    functions: &Vec<FunctionPattern>,
) -> FResult<(Vec<Instruction>, String)> {
    debug!("Variables: {:?}", &variables);

    let mut term_pattern = build_terms_pattern(terms, variables)?;

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
        for function in functions {
            debug!("Check: {:?}", &function.pattern);

            // Look for function pattern (needle) in remaining terms (haystack).
            let needle = Regex::new(&function.pattern).unwrap();

            if let Some(find_location) = needle.find(&term_pattern_clone) {
                let range = find_location.start()..find_location.end();
                let coverage = term_pattern_clone.get(range.clone()).unwrap();

                let mut input = Map::<Argument>::new();
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
                                Argument::new(arg, "variable".to_string(), None, None, None, None, None),
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
