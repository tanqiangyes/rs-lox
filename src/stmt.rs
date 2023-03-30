use crate::error::*;
use crate::expr::*;

#[derive(Debug)]
pub enum Stmt {
      Expression(ExpressionStmt),
      Print(PrintStmt),
}

impl Stmt {
     pub fn accept<R>(&self, visitor: &dyn StmtVisitor<R>) -> Result<R, LoxError> {
          match self {
               Stmt::Expression(be) => be.accept(visitor),
               Stmt::Print(be) => be.accept(visitor),
          }
     }
}


#[derive(Debug)]
pub struct ExpressionStmt {
     pub expression: Expr,
}

impl ExpressionStmt {
     pub fn accept<R>(&self, visitor: &dyn StmtVisitor<R>) -> Result<R, LoxError> {
          visitor.visit_expression_stmt(self)
      }
}

#[derive(Debug)]
pub struct PrintStmt {
     pub expression: Expr,
}

impl PrintStmt {
     pub fn accept<R>(&self, visitor: &dyn StmtVisitor<R>) -> Result<R, LoxError> {
          visitor.visit_print_stmt(self)
      }
}
pub trait StmtVisitor<R> {
      fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<R, LoxError>; 
      fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<R, LoxError>; 
}
