pub type Program = Block;
pub type Block = Vec<Stmt>;

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign(Ident, Expr),
    Block(Block),
    DeclareClass {
        ident: Ident,
    },
    DeclareFunc {
        ident: Ident,
        params: Vec<Ident>,
        body: Block,
    },
    Expr(Expr),
    If {
        condition: Expr,
        consequent: Block,
        alternative: Option<Block>,
    },
    Import(Ident),
    For {
        initializer: Box<Stmt>,
        condition: Expr,
        increment: Box<Stmt>,
        consequent: Block,
    },
    LetAssign(Ident, Expr),
    Return(Option<Expr>),
    While {
        condition: Expr,
        consequent: Block,
    },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Array(Vec<Expr>),
    Call {
        function: Ident,
        arguments: Vec<Expr>,
    },
    Binary {
        operator: BinOp,
        lhs_operand: Box<Expr>,
        rhs_operand: Box<Expr>,
    },
    Ident(Ident),
    Unit,
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Literal(Lit),
    Object(Vec<(Lit, Expr)>),
    Unary {
        operator: UnOp,
        operand: Box<Expr>,
    },
}

#[derive(Clone, Debug)]
pub enum Operator {
    Unary(UnOp),
    Binary(BinOp),
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
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ident(pub String);

#[derive(Clone, Debug)]
pub enum Lit {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
}
