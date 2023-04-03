use crate::error::*;
use crate::expr::{
    BinaryExpr, Expr, ExprVisitor, GroupingExpr, LiteralExpr, UnaryExpr, VariableExpr,
};

pub struct AstPrinter;
impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> Result<String, LoxResult> {
        expr.accept(self)
    }

    pub fn parenthesize(&self, name: &String, exprs: &[&Expr]) -> Result<String, LoxResult> {
        let mut builder = format!("({name}");

        for expr in exprs {
            builder = format!("{builder} {}", expr.accept(self)?);
        }
        builder = format!("{builder})");
        Ok(builder)
    }
}

impl ExprVisitor<String> for AstPrinter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<String, LoxResult> {
        self.parenthesize(&expr.operator.as_string(), &[&expr.left, &expr.right])
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<String, LoxResult> {
        self.parenthesize(&"group".to_string(), &[&expr.expression])
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<String, LoxResult> {
        if let Some(value) = &expr.value {
            Ok(value.to_string())
        } else {
            Ok("nil".to_string())
        }
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<String, LoxResult> {
        self.parenthesize(&expr.operator.lexeme, &[&expr.right])
    }

    fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<String, LoxResult> {
        Ok(format!("var {}", expr.accept(self)?))
    }
}
