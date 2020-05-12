use specifications::common::Value;

#[derive(Debug)]
pub enum AstNode {
    Assignment {
        name: String,
        terms: Vec<AstTerm>,
    },
    Call {
        terms: Vec<AstTerm>,
    },
    Condition {
        predicate: Box<AstNode>,
        if_exec: Box<AstNode>,
        el_exec: Option<Box<AstNode>>,
    },
    Import {
        module: String,
        version: Option<String>,
    },
    Parameter {
        name: String,
        complex: String,
    },
    Repeat {
        predicate: Box<AstNode>,
        exec: Box<AstNode>,
    },
    Terminate {
        terms: Option<Vec<AstTerm>>,
    },
}

#[derive(Debug)]
pub enum AstTerm {
    Name(String),
    Symbol(String),
    Value(Value),
}

///
///
///
pub fn is_import(node: &AstNode) -> bool {
    match node {
        AstNode::Import { module: _, version: _ } => true,
        _ => false,
    }
}

///
///
///
pub fn is_value(node: &AstTerm) -> bool {
    match node {
        AstTerm::Value(_v) => true,
        _ => false,
    }
}
