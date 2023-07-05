use std::collections::HashMap;
use crate::scoped_hash_map::ScopedHashMap;
use crate::error::{Error, Location};
use crate::ast::Type;

struct Context<'a> {
	variables: ScopedHashMap<&'a str, Type>,
	program: &'a crate::ast::Program<'a>,
}

pub fn type_check(program: &crate::ast::Program) -> Result<(), Error> {
	let mut context = Context {
		variables: ScopedHashMap::new(),
		program,
	};
	for function in &program.functions {
		check_function(&mut context, function)?;
	}
	Ok(())
}

fn check_function<'a>(context: &mut Context<'a>, function: &crate::ast::Function<'a>) -> Result<(), Error> {
	context.variables.push_scope();
	for (name, ty) in &function.arguments {
		context.variables.insert(name, ty.clone());
	}
	for statement in &function.statements {
		check_statement(context, statement)?;
	}
	context.variables.pop_scope();
	Ok(())
}

fn check_statement<'a>(context: &mut Context<'a>, statement: &crate::ast::Statement<'a>) -> Result<(), Error> {
	use crate::ast::{Statement::*, If, While};
	match statement {
		VariableDeclaration { name, expression } => {
			if let Some(_) = context.variables.get_local(name) {
				return error(context, expression, format!("variable \"{}\" already defined", name));
			}
			let ty = check_expression(context, expression)?;
			context.variables.insert(name, ty.clone());
		},
		If(If{condition, statement, else_statement}) => {
			assert_type(context, condition, Type::Boolean)?;
			check_statement(context, statement)?;
			if let Some(else_statement) = else_statement {
				check_statement(context, else_statement)?;
			}
		},
		While(While{condition, statement}) => {
			assert_type(context, condition, Type::Boolean)?;
			check_statement(context, statement)?;
		},
		Return(expression) => {
			check_expression(context, expression)?;
		},
		Expression(expression) => {
			check_expression(context, expression)?;
		},
		Block(statements) => {
			context.variables.push_scope();
			for statement in statements {
				check_statement(context, statement)?;
			}
			context.variables.pop_scope();
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
					match context.variables.get(&s).cloned() {
						Some(ty) => {
							assert_type(context, expression, ty.clone())?;
							Ok(ty)
						},
						None => error(context, name, format!("undefined variable \"{}\"", s)),
					}
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
