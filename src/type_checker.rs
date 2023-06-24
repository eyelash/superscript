use std::collections::HashMap;
use crate::error::Error;

#[derive(Clone, PartialEq, Eq, Debug)]
enum Type {
	Number,
	Boolean,
	Void,
}

struct Context<'a> {
	variables: HashMap<&'a str, Type>
}

pub fn type_check(program: &crate::ast::Program) -> Result<(), Error> {
	for function in &program.functions {
		check_function(function)?;
	}
	Ok(())
}

fn check_function(function: &crate::ast::Function) -> Result<(), Error> {
	let mut context = Context {
		variables: HashMap::new(),
	};
	for statement in &function.statements {
		check_statement(&mut context, statement)?;
	}
	Ok(())
}

fn check_statement<'a>(context: &mut Context<'a>, statement: &crate::ast::Statement<'a>) -> Result<(), Error> {
	use crate::ast::{Statement::*, If, While};
	match statement {
		If(If{condition, statements}) => {
			assert_type(context, condition, Type::Boolean)?;
			for statement in statements {
				check_statement(context, statement)?;
			}
		},
		While(While{condition, statements}) => {
			assert_type(context, condition, Type::Boolean)?;
			for statement in statements {
				check_statement(context, statement)?;
			}
		},
		Return(expression) => {
			check_expression(context, expression)?;
		},
		Expression(expression) => {
			check_expression(context, expression)?;
		},
	}
	Ok(())
}

fn check_expression<'a>(context: &mut Context<'a>, expression: &crate::ast::Expression<'a>) -> Result<Type, Error> {
	use crate::ast::Expression::*;
	match expression {
		Number(s) => Ok(Type::Number),
		Name(s) => {
			match context.variables.get(s) {
				None => error("undefined variable"),
				Some(ty) => Ok(ty.clone())
			}
		},
		ArithmeticExpression(expression) => {
			assert_type(context, &*expression.left, Type::Number)?;
			assert_type(context, &*expression.right, Type::Number)?;
			Ok(Type::Number)
		},
		RelationalExpression(expression) => {
			assert_type(context, &*expression.left, Type::Number)?;
			assert_type(context, &*expression.right, Type::Number)?;
			Ok(Type::Boolean)
		},
		Assign(name, expression) => {
			match **name {
				Name(s) => {
					let ty = check_expression(context, expression)?;
					context.variables.insert(s, ty.clone());
					Ok(ty)
				},
				_ => error("left hand of an assignment must be a name"),
			}
		}
	}
}

fn assert_type<'a>(context: &mut Context<'a>, expression: &crate::ast::Expression<'a>, ty: Type) -> Result<(), Error> {
	if check_expression(context, expression)? == ty {
		Ok(())
	} else {
		error("type mismatch")
	}
}

fn error<T, S: Into<String>>(msg: S) -> Result<T, Error> {
	Err(Error {
		i: 0,
		msg: msg.into(),
	})
}
