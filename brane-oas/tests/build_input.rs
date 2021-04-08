mod common;

use anyhow::Result;

///
///
///
#[test]
fn param_count_matches_function_param_count() -> Result<()> {
    let (function, _) = common::build_oas_function_param("/param-count/{1}")?;
    assert_eq!(function.parameters.len(), 1);

    let (function, _) = common::build_oas_function_param("/param-count/{1}/{2}")?;
    assert_eq!(function.parameters.len(), 2);

    let (function, _) = common::build_oas_function_param("/param-count/{1}/{2}/{3}")?;
    assert_eq!(function.parameters.len(), 3);

    // For >=4 parameters, the parameters are grouped into a input type.
    let (function, types) = common::build_oas_function_param("/param-count/{1}/{2}/{3}/{4}")?;
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(types.len(), 1);
    let input_type = types.values().next().unwrap();
    assert_eq!(input_type.properties.len(), 4);

    Ok(())
}

///
///
///
#[test]
fn param_locations_are_irrelevant() -> Result<()> {
    let (function, types) = common::build_oas_function_param("/param-locations/{1}")?;
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(types.len(), 1);
    let input_type = types.values().next().unwrap();
    assert_eq!(input_type.properties.len(), 4);

    Ok(())
}

///
///
///
#[test]
fn param_required_is_preserved() -> Result<()> {
    let (function, _) = common::build_oas_function_param("/param-required")?;
    assert_eq!(function.parameters.len(), 2);

    assert!(function
        .parameters
        .iter()
        .filter(|p| p.name == String::from("1"))
        .any(|p| p.optional.unwrap() == false));

    assert!(function
        .parameters
        .iter()
        .filter(|p| p.name == String::from("2"))
        .any(|p| p.optional.unwrap() == true));

    // For >=4 parameters, the parameters are grouped into a input type.
    let (function, types) = common::build_oas_function_param("/param-required-count-4")?;
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(types.len(), 1);
    let input_type = types.values().next().unwrap();
    assert_eq!(input_type.properties.len(), 4);

    assert!(input_type
        .properties
        .iter()
        .filter(|p| p.name == String::from("1"))
        .any(|p| p.optional.unwrap() == false));

    assert!(input_type
        .properties
        .iter()
        .filter(|p| p.name == String::from("2"))
        .any(|p| p.optional.unwrap() == true));

    Ok(())
}
