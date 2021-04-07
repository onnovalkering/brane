mod common;

use anyhow::Result;

#[test]
fn output_none_unit() -> Result<()> {
    let (function, _) = common::build_oas_function_output("/schema-empty-object")?;
    assert_eq!(function.return_type, String::from("unit"));

    Ok(())
}

#[test]
fn output_singlestring_string() -> Result<()> {
    let (function, types) = common::build_oas_function_output("/schema-string-property")?;
    assert_eq!(function.return_type, String::from("string"));
    assert_eq!(types.len(), 0);

    Ok(())
}

#[test]
fn output_singlenumber_real() -> Result<()> {
    let (function, types) = common::build_oas_function_output("/schema-number-property")?;
    assert_eq!(function.return_type, String::from("real"));
    assert_eq!(types.len(), 0);

    Ok(())
}

#[test]
fn output_singleinteger_integer() -> Result<()> {
    let (function, types) = common::build_oas_function_output("/schema-integer-property")?;
    assert_eq!(function.return_type, String::from("integer"));
    assert_eq!(types.len(), 0);

    Ok(())
}

#[test]
fn output_singleboolean_boolean() -> Result<()> {
    let (function, types) = common::build_oas_function_output("/schema-boolean-property")?;
    assert_eq!(function.return_type, String::from("boolean"));
    assert_eq!(types.len(), 0);

    Ok(())
}

#[test]
fn output_twostrings_object() -> Result<()> {
    let (function, types) = common::build_oas_function_output("/schema-two-properties")?;
    assert_eq!(function.return_type, String::from("TwoPropertiesOutput"));
    assert_eq!(types.len(), 1);
    let input_type = types.values().next().unwrap();
    assert_eq!(input_type.properties.len(), 2);

    Ok(())
}

#[test]
fn output_nestedobjects_err() -> Result<()> {
    let result = common::build_oas_function_output("/schema-nested-objects");
    assert!(result.is_err());

    Ok(())
}
