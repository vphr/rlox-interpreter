use std::cell::RefCell;

use crate::environment::*;
use crate::error::RloxError;
use crate::expr::*;
use crate::scanner::*;
use crate::stmt::*;

#[derive(Debug, Clone)]
pub struct Interpreter {
    environment: RefCell<Environment>,
}
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl ExprVisitor<Value> for Interpreter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Value, RloxError> {
        let left = self.evaluate(*expr.left.clone())?;
        let right = self.evaluate(*expr.right.clone())?;
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => match expr.operator.token_type {
                TokenType::Minus => Ok(Value::Number(l - r)),
                TokenType::Slash => Ok(Value::Number(l / r)),
                TokenType::Star => Ok(Value::Number(l * r)),
                TokenType::Plus => Ok(Value::Number(l + r)),
                TokenType::Greater => Ok(Value::Bool(l.gt(&r))),
                TokenType::GreaterEqual => Ok(Value::Bool(l.ge(&r))),
                TokenType::Less => Ok(Value::Bool(l.lt(&r))),
                TokenType::LessEqual => Ok(Value::Bool(l.le(&r))),
                _ => Err(RloxError::InterpreterError),
            },
            (Value::Str(l), Value::Str(r)) => match expr.operator.token_type {
                TokenType::Plus => Ok(Value::Str(l + &r)),
                _ => Err(RloxError::InterpreterError),
            },
            (left, right) => match expr.operator.token_type {
                TokenType::BangEqual => self.is_equal(left, right),
                TokenType::EqualEqual => self.is_equal(left, right),
                _ => Err(RloxError::InterpreterError),
            },
        }
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Value, RloxError> {
        self.evaluate(*expr.expression.clone())
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Value, RloxError> {
        let expr = expr.value.clone().expect("Valid literal expression");
        Ok(match expr {
            Literal::Identifier(i) => Value::Str(i),
            Literal::Str(s) => Value::Str(s),
            Literal::Number(n) => Value::Number(n),
            Literal::True => Value::Bool(true),
            Literal::False => Value::Bool(false),
            Literal::Nil => Value::Nil,
        })
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Value, RloxError> {
        let right = self.evaluate(*expr.right.clone())?;
        match expr.operator.token_type {
            TokenType::Minus => match right {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(RloxError::InterpreterError),
            },
            TokenType::Bang => Ok(Value::Bool(!self.is_truthy(&right))),
            _ => Err(RloxError::InterpreterError),
        }
    }

    fn visit_variable_expr(&self, variable: &VariableExpr) -> Result<Value, RloxError> {
        self.environment.borrow().get(&variable.name)
    }

    fn visit_assign_expr(&self, assign: &AssignExpr) -> Result<Value, RloxError> {
        let value = self.evaluate(*assign.value.clone())?;
        self.environment
            .borrow_mut()
            .assign(&assign.name, value.clone())?;
        Ok(value)
    }

    fn visit_logical_expr(&self, visitor: &LogicalExpr) -> Result<Value, RloxError> {
        let left = self.evaluate(*visitor.left.clone())?;

        if visitor.operator.token_type == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left);
            }
        }
        self.evaluate(*visitor.right.clone())
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_expression_stmt(&self, visitor: &ExpressionStmt) -> Result<(), RloxError> {
        let e = visitor.expression.as_ref();
        let ee = e.clone();
        self.evaluate(ee)?;
        Ok(())
    }

    fn visit_print_stmt(&self, visitor: &PrintStmt) -> Result<(), RloxError> {
        let e = visitor.expression.as_ref();
        let ee = e.clone();
        let value = self.evaluate(ee)?;
        println!("{}", self.stringify(value));
        Ok(())
    }

    fn visit_var_stmt(&self, visitor: &VarStmt) -> Result<(), RloxError> {
        let value = match &visitor.initializer {
            Some(val) => self.evaluate(*val.clone())?,
            None => Value::Nil,
        };
        self.environment
            .borrow_mut()
            .define(&visitor.name.lexeme, value);
        Ok(())
    }

    fn visit_block_stmt(&self, visitor: &BlockStmt) -> Result<(), RloxError> {
        self.execute_block(
            visitor,
            Environment::new_with_enclosing(self.environment.clone()),
        )?;
        Ok(())
    }

    fn visit_if_stmt(&self, visitor: &IfStmt) -> Result<(), RloxError> {
        if self.is_truthy(&self.evaluate(*visitor.condition.clone())?) {
            self.execute(*visitor.then_branch.clone())?
        } else if let Some(v) = &visitor.else_branch {
            return self.execute(*v.clone());
        }
        Ok(())
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: RefCell::new(Environment::new()),
        }
    }
    pub fn interpret(&self, statements: Vec<Stmt>) -> Result<(), RloxError> {
        for statement in statements {
            self.execute(statement)?
        }
        Ok(())
    }
    fn evaluate(&self, expr: Expr) -> Result<Value, RloxError> {
        expr.accept(self)
    }

    // anything except null and false is true
    fn is_truthy(&self, right: &Value) -> bool {
        match right {
            &Value::Bool(false) | &Value::Nil => false,
            _ => true,
        }
    }

    fn is_equal(&self, left: Value, right: Value) -> Result<Value, RloxError> {
        match (left, right) {
            (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l.eq(&r))),
            (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l.eq(&r))),
            (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l.eq(&r))),
            (Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
            _ => Ok(Value::Bool(false)),
        }
    }

    fn stringify(&self, value: Value) -> String {
        match value {
            Value::Str(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
        }
    }

    fn execute(&self, statement: Stmt) -> Result<(), RloxError> {
        statement.accept(self)
    }

    fn execute_block(&self, block: &BlockStmt, new_env: Environment) -> Result<(), RloxError> {
        let mut previous = std::mem::replace(&mut *self.environment.borrow_mut(), new_env);

        for statement in &block.statements {
            self.execute(statement.clone())?;
        }
        std::mem::swap(&mut *self.environment.borrow_mut(), &mut previous);
        Ok(())
    }
}
