use super::{
    expr::{Expr, ExprLiteral},
    stmt::Stmt,
    token::{LiterialValue, Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    // brief:
    // input:
    // output:
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /*
    program -> declaration * EOF

    declaration -> varDecl | statement

    varDecl -> "var" Identifier ( "=" expression ) ? ";"

    statement -> exprStmt | printStmt | block

    block -> "{" declaration "}"

    exprStmt -> expression ";"

    printstmt -> "print" expression ";"

    expression -> assignment

    assignment -> Identifier "=" assignment | equality

    equality -> comparision ( ("!=" | "==") comparision  ) *

    comparision -> term ( ( ">" | ">=" | "<" | "<=") ) *

    term -> factor ( ( "-" | "+" ) factor ) *

    factor -> unary ( ( "/" | "*") unary ) *

    unary -> ( ( "!" | "-" ) unary ) | primary

    primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | Identifier
    */

    // brief:
    // input:
    // output:
    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    // brief:
    // input:
    // output:
    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Var]) {
            match self.var_declaration() {
                Ok(v) => {
                    return Ok(v);
                }
                Err(err) => {
                    self.synchronize();
                    return Err(err);
                }
            }
        }
        match self.statement() {
            Ok(v) => Ok(v),
            Err(err) => {
                self.synchronize(); // Todo: Check, the return of parse() function will be changed. not a single string, but Vec<String>
                Err(err)
            }
        }
    }

    // brief:
    // input:
    // output:
    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier)?;
        let mut initializer = Expr::Literal {
            value: ExprLiteral::Nil,
        };
        if self.match_tokens(&[TokenType::Equal]) {
            initializer = self.expression()?;
        }
        let _ = self.consume(TokenType::Semicolon)?;

        Ok(Stmt::Var { name, initializer })
    }

    // brief:
    // input:
    // output:
    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_tokens(&[TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    // brief: printstmt -> "print" expression ";"
    // input:
    // output:
    fn print_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon)?;

        Ok(Stmt::Print(expr))
    }

    // brief:
    // input:
    // output:
    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon)?;

        Ok(Stmt::Expression(expr))
    }

    // block -> "{" declaration "}"
    // brief:
    // input:
    // output:
    fn block(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];
        // is_at_end check for forgeting closing "}"
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace)?;
        Ok(Stmt::Block { statements })
    }

    // brief: expression -> assignment
    // input:
    // output:
    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    // brief: assignment -> Identifier "=" assignment | equality
    // input:
    // output:
    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;
        if self.match_tokens(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            } else {
                return Err(format!(
                    "Error occurs when assignment at line: {} at {}.",
                    equals.line_number, equals.lexeme
                ));
            }
        }
        Ok(expr)
    }

    // brief: equality -> comparision ( ("!=" | "==") comparision  ) * ;
    // 1 != 2 != 3 != 4
    // input:
    // output:
    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparision()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right_expr = self.comparision()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }

        Ok(expr)
    }

    // brief: comparision -> term ( ( ">" | ">=" | "<" | "<=") ) * ;
    // input:
    // output:
    fn comparision(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right_expr = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }
        Ok(expr)
    }

    // brief: term -> factor ( ( "-" | "+" ) factor ) * ;
    // input:
    // output:
    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right_expr = self.factor()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }
        Ok(expr)
    }

    // brief: factor -> unary ( ( "/" | "*") unary ) * ;
    // input:
    // output:
    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right_expr = self.unary()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }
        Ok(expr)
    }

    // unary -> ( ( "!" | "-" ) unary ) | primary ;
    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right_expr = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right_expr),
            });
        }
        self.primary()
    }

    // primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::False]) {
            Ok(Expr::Literal {
                value: ExprLiteral::False,
            })
        } else if self.match_tokens(&[TokenType::True]) {
            Ok(Expr::Literal {
                value: ExprLiteral::True,
            })
        } else if self.match_tokens(&[TokenType::Nil]) {
            Ok(Expr::Literal {
                value: ExprLiteral::Nil,
            })
        } else if self.match_tokens(&[TokenType::String]) {
            if let Some(LiterialValue::StringValue(v)) = self.previous().literial {
                return Ok(Expr::Literal {
                    value: ExprLiteral::StringLiteral(v),
                });
            }
            Err(format!(
                "Error occur at parsering String at line {} in {}, Maybe an error from Scanner.",
                self.peek().line_number,
                self.peek().lexeme
            ))
        } else if self.match_tokens(&[TokenType::Number]) {
            if let Some(LiterialValue::FloatValue(v)) = self.previous().literial {
                return Ok(Expr::Literal {
                    value: ExprLiteral::NumberLiteral(v),
                });
            }
            Err(format!(
                "Error occur at parsering Number at line {} in {}, Maybe an error from Scanner.",
                self.peek().line_number,
                self.peek().lexeme
            ))
        } else if self.match_tokens(&[TokenType::Identifier]) {
            Ok(Expr::Variable {
                name: self.previous(),
            })
        } else if self.match_tokens(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            let _ = self.consume(TokenType::RightParen)?; // Consume the RightParen.
            Ok(Expr::Grouping {
                expression: Box::new(expr),
            })
        } else {
            Err(format!(
                "Parsering error occurs for finding nothing to match with at line {} in {}.",
                self.peek().line_number,
                self.peek().lexeme,
            ))
        }
    }

    // brief: Tihs is the second version of primary(),clear and short superficially，
    // which increase the degree of code coupling.So i still use if to match.
    // input:
    // output:
    // fn primary2(&mut self) -> Result<Expr, String> {
    //     if self.match_tokens(&[TokenType::LeftParen]) {
    //         let expr = self.expression();
    //         self.consume();
    //         Ok(Expr::Grouping {
    //             expression: Box::new(expr),
    //         })
    //     } else {
    //         let current_token = self.peek();
    //         self.advance();
    //         Ok(Expr::Literal {
    //             value: ExprLiteral::from_token(current_token)?,
    //         })
    //     }
    // }

    // brief: Consume the current token, if tokentype matched.
    // input:
    // output:
    fn consume(&mut self, token_type: TokenType) -> Result<Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(format!(
                "Parsering error occur when consuming some token at line: {} in {}.",
                self.peek().line_number,
                self.peek().lexeme,
            ))
        }
    }

    // brief: Synchronize to give up the error code untill find a Unerror Defination..
    // input:
    // output:
    fn synchronize(&mut self) {
        self.advance(); // Consume the error Token.
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::CLass
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,

                _ => {
                    self.advance();
                }
            }
        }
    }
    // brief: Check tempToken and self.current ++ if matched really.
    // input:
    // output:
    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    // brief: Peek the next char.
    // input:
    // output:
    // Attention : Make sure self.current is available before call self.peek().
    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    // brief: return current token and self.current ++
    // input:
    // output:
    // Attention : if is_at_end() return will be the last one, and current do not increase.
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // brief: peek the previous token.
    // input:
    // output:
    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    // brief: check if self current is at end.
    // input:
    // output:
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scanner;
    #[test]
    fn parser_test_one() {
        let sources = "1.0 * 3.0 * 2.0 + 2.0 * 4.0 == 11.0".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        // let pas = Parser::new(tok).parse().unwrap().two_string();

        // dbg!(pas);
    }

    #[test]
    fn parser_test_two() {
        let sources = "1.0 >= * 3.0".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }

    #[test]
    fn parser_test_three() {
        let sources = "1.0 >= 1.0 + 2.0 == 4.0".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }

    #[test]
    fn parser_test_four() {
        let sources = "- - - - - - - - true".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }

    #[test]
    fn parser_test_five() {
        let sources = "1.0 + ".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }
    #[test]
    fn parser_test_six() {
        let sources = "1.0 + 2.0; \n 2.0 * 3.0 + 4.0;\n".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }

    #[test]
    fn parser_test_seven() {
        let sources = "var abc = 123.0; \n print abc;\n print  2.0 * abc + 4.0;\n".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }

    #[test]
    fn parser_test_eight() {
        let sources =
            "var abc = 123.0; var ltl = true; \n print abc;\n print  2.0 * abc + 4.0 > 0.0 == true;"
                .to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }

    #[test]
    fn parser_test_nine() {
        let sources = "var abc = 123.0;var bbb =10.0; abc =bbb= 10.0;".to_string();
        let mut scan = Scanner::new(sources);

        let tok = scan.scan_tokens().unwrap();

        match Parser::new(tok).parse() {
            Err(error) => {
                println!("[    Error!    ] ---> {}", error);
            }
            Ok(v) => {
                dbg!(v);
            }
        }
    }
}
// cargo test <unique keyword> --  --nocapture
