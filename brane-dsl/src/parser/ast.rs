use semver::Version;
use std::collections::HashMap;

pub type Program = Block;
pub type Block = Vec<Stmt>;

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign(Ident, Expr),
    Block(Block),
    DeclareClass {
        ident: Ident,
        properties: HashMap<Ident, Ident>,
        methods: HashMap<Ident, Stmt>,
    },
    DeclareFunc {
        ident: Ident,
        params: Vec<Ident>,
        body: Block,
    },
    Expr(Expr),
    For {
        initializer: Box<Stmt>,
        condition: Expr,
        increment: Box<Stmt>,
        consequent: Block,
    },
    If {
        condition: Expr,
        consequent: Block,
        alternative: Option<Block>,
    },
    Import {
        package: Ident,
        version: Option<Version>,
    },
    LetAssign(Ident, Expr),
    On {
        location: Expr,
        block: Block,
    },
    Parallel {
        let_assign: Option<Ident>,
        blocks: Vec<Stmt>,
    },
    Property {
        ident: Ident,
        class: Ident,
    },
    Return(Option<Expr>),
    While {
        condition: Expr,
        consequent: Block,
    },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Array(Vec<Expr>),
    Binary {
        operator: BinOp,
        lhs_operand: Box<Expr>,
        rhs_operand: Box<Expr>,
    },
    Call {
        function: Ident,
        arguments: Vec<Expr>,
    },
    Ident(Ident),
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Instance {
        class: Ident,
        properties: Vec<Stmt>,
    },
    Literal(Lit),
    Pattern(Vec<Expr>),
    Unary {
        operator: UnOp,
        operand: Box<Expr>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Ident(pub String);

#[derive(Clone, Debug)]
pub enum Lit {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Unit,
}

impl Lit {
    pub fn data_type(&self) -> String {
        match self {
            Lit::Boolean(_) => String::from("boolean"),
            Lit::Integer(_) => String::from("integer"),
            Lit::Real(_) => String::from("real"),
            Lit::String(_) => String::from("string"),
            Lit::Unit => String::from("unit"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Operator {
    Binary(BinOp),
    Unary(UnOp),
}

#[derive(Clone, Debug)]
/// Operations with a two operands.
pub enum BinOp {
    /// The `+` operator (addition)
    Add,
    /// The `-` operator (subtraction)
    Sub,
    /// The `*` operator (multiplication)
    Mul,
    /// The `/` operator (division)
    Div,
    /// The `.` operator (nesting)
    Dot,
    /// The `&&` operator (logical and)
    And,
    /// The `||` operator (logical or)
    Or,
    /// The `==` operator (equality)
    Eq,
    /// The `<` operator (less than)
    Lt,
    /// The `<=` operator (less than or equal to)
    Le,
    /// The `!=` operator (not equal to)
    Ne,
    /// The `>=` operator (greater than or equal to)
    Ge,
    /// The `>` operator (greater than)
    Gt,
}

impl BinOp {
    ///
    ///
    ///
    pub fn binding_power(&self) -> (u8, u8) {
        match &self {
            BinOp::And | BinOp::Or => (1, 2),   // Conditional
            BinOp::Eq | BinOp::Ne => (3, 4),    // Equality
            BinOp::Lt | BinOp::Gt => (5, 6),    // Comparison
            BinOp::Le | BinOp::Ge => (5, 6),    // Comparison
            BinOp::Add | BinOp::Sub => (7, 8),  // Terms
            BinOp::Mul | BinOp::Div => (9, 10), // Factors
            BinOp::Dot => (13, 14),             // Nesting
        }
    }
}

#[derive(Clone, Debug)]
/// Operations with one operand.
pub enum UnOp {
    /// The '[' operator (index)
    Idx,
    /// The `!` operator (logical inversion)
    Not,
    /// The `-` operator (negation)
    Neg,
    /// The '(' operator (prioritize)
    Prio,
}

impl UnOp {
    ///
    ///
    ///
    pub fn binding_power(&self) -> (u8, u8) {
        match &self {
            UnOp::Not => (0, 11),
            UnOp::Neg => (0, 11),
            UnOp::Idx => (11, 0),
            UnOp::Prio => (0, 0), // Handled seperatly by pratt parser.
        }
    }
}
