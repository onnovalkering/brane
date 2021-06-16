mod common;
use anyhow::Result;

#[test]
fn resp_empty_unit() -> Result<()> {
    let (function, _) = common::build_oas_function_resp("/schema-empty-object", "emptyObject")?;
    assert_eq!(function.return_type, String::from("unit"));

    Ok(())
}

#[test]
fn resp_string_string() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-string", "string")?;
    assert_eq!(function.return_type, String::from("StringOutput"));
    assert_eq!(types.len(), 1);

    Ok(())
}

#[test]
fn resp_number_real() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-number", "number")?;
    assert_eq!(function.return_type, String::from("NumberOutput"));
    assert_eq!(types.len(), 1);

    Ok(())
}

#[test]
fn resp_integer_integer() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-integer", "integer")?;
    assert_eq!(function.return_type, String::from("IntegerOutput"));
    assert_eq!(types.len(), 1);

    Ok(())
}

#[test]
fn resp_boolean_boolean() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-boolean", "boolean")?;
    assert_eq!(function.return_type, String::from("BooleanOutput"));
    assert_eq!(types.len(), 1);

    Ok(())
}

#[test]
fn resp_object_object() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-object", "object")?;
    assert_eq!(function.return_type, String::from("ObjectOutput"));
    assert_eq!(types.len(), 1);
    let input_type = types.values().next().unwrap();
    assert_eq!(input_type.properties.len(), 2);

    Ok(())
}

#[test]
fn resp_nestedobjects_err() -> Result<()> {
    let result = common::build_oas_function_resp("/schema-nested-objects", "nestedObjects");
    assert!(result.is_err());

    Ok(())
}

#[test]
fn resp_stringarray_stringarray() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-string-array", "stringArray")?;
    assert_eq!(function.return_type, String::from("string[]"));
    assert_eq!(types.len(), 0);

    Ok(())
}

#[test]
fn resp_numberarray_realarray() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-number-array", "numberArray")?;
    assert_eq!(function.return_type, String::from("real[]"));
    assert_eq!(types.len(), 0);

    Ok(())
}

#[test]
fn resp_integerarray_integerarray() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-integer-array", "integerArray")?;
    assert_eq!(function.return_type, String::from("integer[]"));
    assert_eq!(types.len(), 0);

    Ok(())
}

#[test]
fn resp_booleanarray_booleanarray() -> Result<()> {
    let (function, types) = common::build_oas_function_resp("/schema-boolean-array", "booleanArray")?;
    assert_eq!(function.return_type, String::from("boolean[]"));
    assert_eq!(types.len(), 0);

    Ok(())
}
