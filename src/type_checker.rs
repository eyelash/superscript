use std::collections::HashMap;
use crate::error::{Error, Location};
use crate::ast::Type;

struct Context<'a> {
	variables: HashMap<&'a str, Type>,
	program: &'a crate::ast::Program<'a>,
}

pub fn type_check(program: &crate::ast::Program) -> Result<(), Error> {
	for function in &program.functions {
		check_function(program, function)?;
	}
	Ok(())
}

fn check_function(program: &crate::ast::Program, function: &crate::ast::Function) -> Result<(), Error> {
	let mut context = Context {
		variables: HashMap::new(),
		program: &program,
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
				None => error(context, expression, format!("undefined variable \"{}\"", s)),
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
		LogicalExpression(expression) => {
			assert_type(context, &*expression.left, Type::Boolean)?;
			assert_type(context, &*expression.right, Type::Boolean)?;
			Ok(Type::Boolean)
		},
		Not(expression) => {
			assert_type(context, &*expression, Type::Boolean)?;
			Ok(Type::Boolean)
		},
		Assign { name, expression } => {
			match **name {
				Name(s) => {
					let ty = check_expression(context, expression)?;
					context.variables.insert(s, ty.clone());
					Ok(ty)
				},
				_ => error(context, name, "left hand of an assignment must be a name"),
			}
		},
		Call { function, arguments } => {
			match **function {
				Name(s) => {
					match context.program.get_function(s) {
						Some(f) => {
							if arguments.len() != f.arguments.len() {
								error(context, function, "invalid number of arguments")
							} else {
								let argument_types = f.arguments.iter().map(|(_, ty)| ty);
								for (argument, expected_ty) in arguments.iter().zip(argument_types) {
									let actual_ty = check_expression(context, argument)?;
									if &actual_ty != expected_ty {
										return error(context, argument, format!("invalid argument type: expected {:?} but found {:?}", expected_ty, actual_ty));
									}
								}
								Ok(f.return_type.clone())
							}
						},
						None => error(context, function, format!("undefined function \"{}\"", s)),
					}
				},
				_ => error(context, function, "left hand of a call must be a name"),
			}
		},
	}
}

fn assert_type<'a>(context: &mut Context<'a>, expression: &crate::ast::Expression<'a>, expected_ty: Type) -> Result<(), Error> {
	let actual_ty = check_expression(context, expression)?;
	if actual_ty == expected_ty {
		Ok(())
	} else {
		let msg = format!("type mismatch: expected a {:?} but found a {:?}", expected_ty, actual_ty);
		error(context, expression, msg)
	}
}

fn error<'a, T, S: Into<String>>(context: &mut Context<'a>, expression: &crate::ast::Expression<'a>, msg: S) -> Result<T, Error> {
	let key: * const crate::ast::Expression<'a> = expression;
	let i = context.program.locations.get(&key).copied().unwrap_or_default();
	Err(Error {
		i,
		msg: msg.into(),
	})
}
