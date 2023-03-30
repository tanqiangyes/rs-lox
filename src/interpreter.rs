use crate::error::LoxError;
use crate::expr::*;
use crate::object::Object;
use crate::token_type::TokenType;

pub struct Interpreter {}

impl ExprVisitor<Object> for Interpreter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Object, LoxError> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        let op = expr.operator.token_type();
        let result = match (left, right) {
            (Object::Num(left), Object::Num(right)) => match op {
                TokenType::Plus => Object::Num(left + right),
                TokenType::Minus => Object::Num(left - right),
                TokenType::Slash => Object::Num(left / right),
                TokenType::Star => Object::Num(left * right),
                TokenType::Greater => Object::Bool(left > right),
                TokenType::GreaterEqual => Object::Bool(left >= right),
                TokenType::Less => Object::Bool(left < right),
                TokenType::LessEqual => Object::Bool(left <= right),
                TokenType::Equal => Object::Bool(left == right),
                TokenType::BangEqual => Object::Bool(left != right),
                _ => {
                    return Err(LoxError::error(
                        expr.operator.line,
                        "Unreachable according to num binary expression",
                    ));
                }
            },
            (Object::Str(left), Object::Str(right)) => match op {
                TokenType::Plus => Object::Str(left + &*right),
                TokenType::Greater => Object::Bool(left > right),
                TokenType::GreaterEqual => Object::Bool(left >= right),
                TokenType::Less => Object::Bool(left < right),
                TokenType::LessEqual => Object::Bool(left <= right),
                TokenType::Equal => Object::Bool(left == right),
                TokenType::BangEqual => Object::Bool(left != right),
                _ => {
                    return Err(LoxError::error(
                        expr.operator.line,
                        "Unreachable according to string binary expression",
                    ));
                }
            },
            (Object::Nil, Object::Nil) => match op {
                TokenType::Equal => Object::Bool(true),
                TokenType::BangEqual => Object::Bool(false),
                _ => {
                    return Err(LoxError::error(
                        expr.operator.line,
                        "Unreachable according to nil binary expression",
                    ));
                }
            },
            (Object::Nil, _) => match op {
                TokenType::Equal => Object::Bool(false),
                TokenType::BangEqual => Object::Bool(true),
                _ => {
                    return Err(LoxError::error(
                        expr.operator.line,
                        "Unreachable according to nil eq other binary expression",
                    ));
                }
            },
            (Object::Bool(left), Object::Bool(right)) => match op {
                TokenType::Equal => Object::Bool(left == right),
                TokenType::BangEqual => Object::Bool(left != right),
                _ => {
                    return Err(LoxError::error(
                        expr.operator.line,
                        "Unreachable according to bool binary expression",
                    ));
                }
            },
            _ => {
                return Err(LoxError::error(
                    expr.operator.line,
                    "Both operands of the comparison expression must be of the same type",
                ))
            }
        };

        if result == Object::ArithmeticError {
            Err(LoxError::runtime_error(
                expr.operator.dup(),
                "Illegal expression",
            ))
        } else {
            Ok(result)
        }
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Object, LoxError> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Object, LoxError> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxError> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => Ok(Object::Num(-n)),
                _ => Ok(Object::Nil),
            },
            TokenType::Bang => Ok(Object::Bool(!self.is_truthy(right))),
            _ => Err(LoxError::error(
                expr.operator.line,
                "Unreachable according to Unary expression",
            )),
        }
    }
}

impl Interpreter {
    fn evaluate(&self, expr: &Box<Expr>) -> Result<Object, LoxError> {
        expr.accept(self)
    }

    fn is_truthy(&self, right: Object) -> bool {
        match right {
            Object::Nil | Object::Bool(false) => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    fn make_literal_number(n: f64) -> Box<Expr> {
        Box::new(Expr::Literal(LiteralExpr {
            value: Some(Object::Num(n)),
        }))
    }

    fn make_literal_string(n: String) -> Box<Expr> {
        Box::new(Expr::Literal(LiteralExpr {
            value: Some(Object::Str(n)),
        }))
    }

    fn make_literal_bool(n: bool) -> Box<Expr> {
        Box::new(Expr::Literal(LiteralExpr {
            value: Some(Object::Bool(n)),
        }))
    }

    fn make_literal_nil() -> Box<Expr> {
        Box::new(Expr::Literal(LiteralExpr {
            value: Some(Object::Nil),
        }))
    }

    fn make_operator(
        ttype: TokenType,
        lexeme: String,
        literal: Option<Object>,
        line: usize,
    ) -> Token {
        Token::new(ttype, lexeme, literal, line)
    }

    fn make_unary_expr(
        right: f64,
        ttype: TokenType,
        lexeme: String,
        literal: Option<Object>,
        line: usize,
    ) -> UnaryExpr {
        UnaryExpr {
            operator: make_operator(ttype, lexeme, literal, line),
            right: make_literal_number(right),
        }
    }

    fn make_unary_bool(
        right: bool,
        ttype: TokenType,
        lexeme: String,
        literal: Option<Object>,
        line: usize,
    ) -> UnaryExpr {
        UnaryExpr {
            operator: make_operator(ttype, lexeme, literal, line),
            right: make_literal_bool(right),
        }
    }

    fn make_binary_expr(
        left: f64,
        right: f64,
        ttype: TokenType,
        lexeme: String,
        literal: Option<Object>,
        line: usize,
    ) -> BinaryExpr {
        BinaryExpr {
            left: make_literal_number(left),
            operator: make_operator(ttype, lexeme, literal, line),
            right: make_literal_number(right),
        }
    }

    fn make_binary_expr_str(
        left: String,
        right: String,
        ttype: TokenType,
        lexeme: String,
        literal: Option<Object>,
        line: usize,
    ) -> BinaryExpr {
        BinaryExpr {
            left: make_literal_string(left),
            operator: make_operator(ttype, lexeme, literal, line),
            right: make_literal_string(right),
        }
    }

    #[test]
    fn test_unary_minus() {
        let terp = Interpreter {};
        let unary_expr = make_unary_expr(123.0, TokenType::Minus, "-".to_string(), None, 123);

        let result = terp.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(-123.0)));
    }

    #[test]
    fn test_unary_not() {
        let terp = Interpreter {};
        let unary_expr = make_unary_bool(false, TokenType::Bang, "!".to_string(), None, 123);

        let result = terp.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_subtraction() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr(15.0, 7.0, TokenType::Minus, "-".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(8.0)));
    }

    #[test]
    fn test_subtraction_error() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal_number(15.0),
            operator: make_operator(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal_bool(false),
        };

        let result = terp.visit_binary_expr(&binary_expr);
        println!("{:?}", result.as_ref().err());
        assert!(result.is_err());
    }

    #[test]
    fn test_slash() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr(14.0, 7.0, TokenType::Slash, "/".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(2.0)));
    }

    #[test]
    fn test_star() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr(2.0, 7.0, TokenType::Star, "*".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(14.0)));
    }

    #[test]
    fn test_add() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr(2.0, 7.0, TokenType::Plus, "+".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(9.0)));
    }

    #[test]
    fn test_str_add() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr_str(
            "jack".to_string(),
            "tan".to_string(),
            TokenType::Plus,
            "+".to_string(),
            None,
            123,
        );

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Str("jacktan".to_string())));
    }

    #[test]
    fn test_greater_than() {
        let terp = Interpreter {};
        let binary_expr =
            make_binary_expr(15.0, 10.0, TokenType::Greater, ">".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_greater_equal_than() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr(
            10.0,
            10.0,
            TokenType::GreaterEqual,
            ">=".to_string(),
            None,
            123,
        );

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_less_than() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr(10.0, 9.0, TokenType::Less, "<".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(false)));
    }

    #[test]
    fn test_less_greater_than() {
        let terp = Interpreter {};
        let binary_expr =
            make_binary_expr(9.0, 9.0, TokenType::LessEqual, "<=".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_equal() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr(9.0, 9.0, TokenType::Equal, "==".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_equal_str() {
        let terp = Interpreter {};
        let binary_expr = make_binary_expr_str(
            "9.0".to_string(),
            "9.0".to_string(),
            TokenType::Equal,
            "==".to_string(),
            None,
            123,
        );

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_bang_equal() {
        let terp = Interpreter {};
        let binary_expr =
            make_binary_expr(9.0, 9.0, TokenType::BangEqual, "!=".to_string(), None, 123);

        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(false)));
    }

    #[test]
    fn test_nil() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal_bool(false),
            operator: make_operator(TokenType::Equal, "=".to_string(), None, 123),
            right: make_literal_bool(false),
        };

        let result = terp.visit_binary_expr(&binary_expr);

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_str_equal_num() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal_string("1.0".to_string()),
            operator: make_operator(TokenType::Equal, "=".to_string(), None, 123),
            right: make_literal_number(1.0),
        };

        let result = terp.visit_binary_expr(&binary_expr);
        // println!("{:?}", result.as_ref().err());
        assert!(result.is_err());
    }

    #[test]
    fn test_nil_and_other() {
        let terp = Interpreter {};
        let binary_expr = BinaryExpr {
            left: make_literal_nil(),
            operator: make_operator(TokenType::Equal, "=".to_string(), None, 123),
            right: make_literal_bool(false),
        };

        let result = terp.visit_binary_expr(&binary_expr);

        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(false)));
    }

    #[test]
    fn test_run_comparison() {
        let terp = Interpreter {};

        let binary_expr = make_binary_expr_str(
            "9.0".to_string(),
            "9.0".to_string(),
            TokenType::Equal,
            "==".to_string(),
            None,
            123,
        );
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));

        let binary_expr = make_binary_expr_str(
            "9.0".to_string(),
            "9.0".to_string(),
            TokenType::Equal,
            "==".to_string(),
            None,
            123,
        );
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));

        let binary_expr = make_binary_expr_str(
            "9.0".to_string(),
            "9.0".to_string(),
            TokenType::Equal,
            "==".to_string(),
            None,
            123,
        );
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }
}
