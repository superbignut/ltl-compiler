use super::{expr::Expr, token::Token};

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Let {
        name: Token,
        initializer: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
    If {
        condition: Expr,
        thenBranch: Box<Stmt>,
        elseBranch: Option<Box<Stmt>>,
    },
}
