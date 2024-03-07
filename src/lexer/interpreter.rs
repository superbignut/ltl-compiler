use super::{
    environment::Environment,
    expr::{Expr, ExprLiteral},
    parser::{self, Parser},
    stmt::Stmt,
    token::{Token, TokenType},
};

pub struct Interpreter {
    environment: Environment, // struct to save variavle and create scope.
}

impl Interpreter {
    // brief: Create a Interpreter, with setting previous Env None.
    // input:
    // output:
    pub fn new() -> Self {
        Self {
            environment: Environment::new(None),
        }
    }

    // brief: Pub function to evaluate Vec<Stmt> by Match all kinds of Stmt.
    // input:
    // output:
    pub fn interpreter(&mut self, statements: &Vec<Stmt>) -> Result<(), String> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), String> {
        match statement {
            // If just an expression.
            Stmt::Expression(v) => {
                let _ = self.evaluate(v)?; // Evaluate Expression.
            }
            // If a print statement.
            Stmt::Print(v) => {
                println!("{}", (self.evaluate(v)?).two_string()); // Print Expression.
            }
            // If a Var defination.
            Stmt::Let { name, initializer } => {
                let value;
                if *initializer
                    != (Expr::Literal {
                        value: ExprLiteral::Nil,
                    })
                {
                    value = self.evaluate(initializer)?;
                    self.environment.define(name.lexeme.clone(), value); // Define variable in the temp Environment.
                }
            }
            // If a Block.
            Stmt::Block { statements } => {
                self.environment = Environment::new(Some(Box::new(self.environment.clone()))); // Save temp environment.and Restore later.
                self.interpreter(statements)?; // Scope recursively;
                self.environment = *self.environment.enclosing.clone().unwrap();
            }
            // If an If.
            Stmt::If {
                condition,
                thenBranch,
                elseBranch,
            } => {
                let if_condition = self.evaluate(condition)?;
                if self.is_truthy(if_condition) == ExprLiteral::True {
                    // then branch.
                    self.execute(thenBranch)?;
                } else if let Some(v) = elseBranch {
                    // If there is an else branch.
                    self.execute(v)?;
                } else {
                    // No else branch, just continue.
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    // brief: Evaluate an Expression.
    // input:
    // output:
    pub fn evaluate(&mut self, expr: &Expr) -> Result<ExprLiteral, String> {
        self.match_expr(expr)
    }

    // brief: Match all kinds of Expression recursively.
    // input:
    // output:
    fn match_expr(&mut self, expr: &Expr) -> Result<ExprLiteral, String> {
        match expr {
            // 1 Literal
            Expr::Literal { value } => Ok(value.clone()),

            // 2 Grouping
            Expr::Grouping { expression } => self.evaluate(expression), // recursively.

            // 3 Unary
            Expr::Unary { operator, right } => {
                if operator.token_type == TokenType::Minus {
                    if let ExprLiteral::NumberLiteral(v) = self.evaluate(right)? {
                        return Ok(ExprLiteral::NumberLiteral(-v));
                    }
                    return Err(format!(
                        "Error occur when interpreter number at line {} at {}.",
                        operator.line_number, operator.lexeme
                    ));
                } else if operator.token_type == TokenType::Bang {
                    let evaluated = self.evaluate(right)?;
                    return Ok(self.is_truthy(evaluated));
                }
                Err(format!(
                    "Error occur when interpreter at line {} at {} for no matching unary operator.",
                    operator.line_number, operator.lexeme
                ))
            }

            // 4 Variable
            Expr::Variable { name } => Ok(self.environment.get(name)?), // Get variable.

            // 5 Assign
            Expr::Assign { name, value } => {
                let new_value = self.evaluate(value)?; // recursively.
                self.environment.assign(name, new_value.clone())?; // define variable.
                Ok(new_value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                if operator.token_type == TokenType::Or {
                    if self.is_truthy(left.clone()) == ExprLiteral::True {
                        Ok(left) // A OR B : A == true return A
                    } else {
                        Ok(self.evaluate(right)?) // A OR B : A == false return B
                    }
                } else if self.is_truthy(left.clone()) == ExprLiteral::False {
                    Ok(left) // A AND B : A == false return A
                } else {
                    Ok(self.evaluate(right)?) // A AND B : A == true return B
                }
            }

            // 5 Binary
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_operand = self.evaluate(left)?; // recursively.
                let right_operand = self.evaluate(right)?; // recursively.

                match operator.token_type {
                    TokenType::Minus => {
                        if let (true, l_number, r_number) =
                            self.check_number_operands(&left_operand, &right_operand)
                        {
                            return Ok(ExprLiteral::NumberLiteral(l_number - r_number));
                        }
                        Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                    },
                    TokenType::Slash => {
                        if let (true, l_number, r_number) =
                            self.check_number_operands(&left_operand, &right_operand)
                        {
                            return Ok(ExprLiteral::NumberLiteral(l_number / r_number));
                        }
                        Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                    },
                    TokenType::Star => {
                        if let (true, l_number, r_number) =
                            self.check_number_operands(&left_operand, &right_operand)
                        {
                            return Ok(ExprLiteral::NumberLiteral(l_number * r_number));
                        }
                        Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                    },
                    TokenType::Plus => match (left_operand, right_operand) {
                        (
                            ExprLiteral::NumberLiteral(l_number),
                            ExprLiteral::NumberLiteral(r_number),
                        ) => Ok(ExprLiteral::NumberLiteral(l_number + r_number)),

                        (
                            ExprLiteral::StringLiteral(l_string),
                            ExprLiteral::StringLiteral(r_string),
                        ) => Ok(ExprLiteral::StringLiteral(format!(
                            "{}{}",
                            l_string, r_string
                        ))),

                        _ => {
                            Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                        }
                    },
                    TokenType::Greater => {
                        if let (true, l_number, r_number) =
                            self.check_number_operands(&left_operand, &right_operand)
                        {
                            if l_number > r_number {
                                return Ok(ExprLiteral::True);
                            } else {
                                return Ok(ExprLiteral::False);
                            }
                        }
                         Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                    },
                    TokenType::GreaterEqual => {
                        if let (true, l_number, r_number) =
                            self.check_number_operands(&left_operand, &right_operand)
                        {
                            if l_number >= r_number {
                                return Ok(ExprLiteral::True);
                            } else {
                                return Ok(ExprLiteral::False);
                            }
                        }
                         Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                    },
                    TokenType::Less => {
                        if let (true, l_number, r_number) =
                            self.check_number_operands(&left_operand, &right_operand)
                        {
                            if l_number < r_number {
                                return Ok(ExprLiteral::True);
                            } else {
                                return Ok(ExprLiteral::False);
                            }
                        }
                         Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                    },
                    TokenType::LessEqual => {
                        if let (true, l_number, r_number) =
                            self.check_number_operands(&left_operand, &right_operand)
                        {
                            if l_number <= r_number {
                                return Ok(ExprLiteral::True);
                            } else {
                                return Ok(ExprLiteral::False);
                            }
                        }
                         Err(format!(
                            "Error occur when interpreter at line {} at {} for some wrong operand.",
                            operator.line_number, operator.lexeme
                        ))
                    },
                    TokenType::EqualEqual => {
                        if left_operand.is_equal(&right_operand) {
                            Ok(ExprLiteral::True)
                        } else {
                            Ok(ExprLiteral::False)
                        }
                    },
                    TokenType::BangEqual => {
                        if !left_operand.is_equal(&right_operand) {
                            Ok(ExprLiteral::True)
                        } else {
                            Ok(ExprLiteral::False)
                        }
                    },
                    _ => {
                         Err(format!(
                            "Error occur when interpreter at line {} at {} for no matchine Binary operator.",
                            operator.line_number, operator.lexeme
                        ))
                    }
                }
            }
        }
    }

    // brief: operand is f64 ?
    // input:
    // output:
    fn check_number_operands(
        &self,
        l_operand: &ExprLiteral,
        r_operand: &ExprLiteral,
    ) -> (bool, f64, f64) {
        if let (true, v1) = self.check_number_operand(l_operand) {
            if let (true, v2) = self.check_number_operand(r_operand) {
                return (true, v1, v2);
            }
        }
        (false, 0.0, 0.0)
    }

    // brief: operand is f64 ?
    // input:
    // output:
    fn check_number_operand(&self, operand: &ExprLiteral) -> (bool, f64) {
        if let ExprLiteral::NumberLiteral(v) = operand {
            return (true, *v);
        }
        (false, 0.0)
    }

    // brief: All is true but nil and false.
    // input:
    // output:
    fn is_truthy(&self, expr: ExprLiteral) -> ExprLiteral {
        match expr {
            ExprLiteral::False | ExprLiteral::Nil => ExprLiteral::False,
            _ => ExprLiteral::True,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Interpreter;
    use super::Parser;
    use crate::Scanner;

    #[test]
    fn test_inter_one() {
        let sources = "1.0 * 3.0 * 2.0 + 2.0 * 4.1 = 14.0".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        let pas = Parser::new(tok).parse().unwrap();

        // match Interpreter::new().evaluate(&pas) {
        //     Ok(v) => {
        //         println!("[    PASS!     ] ---> {}", v.two_string());
        //     }
        //     Err(v) => {
        //         println!("[    Error!    ] ---> {}", v);
        //     }
        // }
        //dbg!(pas);
    }

    #[test]
    fn test_inter_two() {
        let sources =
            "1.0 * 3.0 * ( 2.0 + 14.0 ) * 4.0 / 8.0 ; \n print \" Successfully!! \"; ".to_string();

        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        let pas = Parser::new(tok).parse().unwrap();

        match Interpreter::new().interpreter(&pas) {
            Ok(()) => {
                println!("[    PASS!     ] ---> Compile Successfully.");
            }
            Err(v) => {
                println!("[    Error!    ] ---> {}", v);
            }
        }
        //        dbg!(pas);
    }
    #[test]
    fn test_inter_three() {
        let sources = "let a = 10.0; let b = 2.0; print a + b + 12.0; ".to_string();

        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        let pas = Parser::new(tok).parse().unwrap();

        match Interpreter::new().interpreter(&pas) {
            Ok(()) => {
                println!("[    PASS!     ] ---> Compile Successfully.");
            }
            Err(v) => {
                println!("[    Error!    ] ---> {}", v);
            }
        }
        //        dbg!(pas);
    }
    #[test]
    fn test_inter_four() {
        let sources = "let a = 10.0; let b = 2.0; print a + b + 12.0 >= 25.0 == true; ".to_string();

        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        let pas = Parser::new(tok).parse().unwrap();

        match Interpreter::new().interpreter(&pas) {
            Ok(()) => {
                println!("[    PASS!     ] ---> Compile Successfully.");
            }
            Err(v) => {
                println!("[    Error!    ] ---> {}", v);
            }
        }
        //        dbg!(pas);
    }

    #[test]
    fn test_inter_five() {
        let sources = "let a = 10.0; print a = 20.0; a = a + 20.0; print a ; ".to_string();

        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        let pas = Parser::new(tok).parse().unwrap();

        match Interpreter::new().interpreter(&pas) {
            Ok(()) => {
                println!("[    PASS!     ] ---> Compile Successfully.");
            }
            Err(v) => {
                println!("[    Error!    ] ---> {}", v);
            }
        }
        //        dbg!(pas);
    }
}

// cargo test unique-keyword -- --nocapture
