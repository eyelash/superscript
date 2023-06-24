use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Debug)]
enum Type {
	Number,
	Boolean,
	Void,
}

struct Context<'a> {
	variables: HashMap<&'a str, Type>
}

pub fn type_check(program: &crate::ast::Program) -> Result<(), &'static str> {
	for function in &program.functions {
		check_function(function)?;
	}
	Ok(())
}

fn check_function(function: &crate::ast::Function) -> Result<(), &'static str> {
	let mut context = Context {
		variables: HashMap::new(),
	};
	for statement in &function.statements {
		check_statement(&mut context, statement)?;
	}
	Ok(())
}

fn check_statement<'a>(context: &mut Context<'a>, statement: &crate::ast::Statement<'a>) -> Result<(), &'static str> {
	match statement {
		crate::ast::Statement::If(crate::ast::If{condition, statements}) => {
			assert_type(context, condition, Type::Boolean)?;
			for statement in statements {
				check_statement(context, statement)?;
			}
		},
		crate::ast::Statement::While(crate::ast::While{condition, statements}) => {
			assert_type(context, condition, Type::Boolean)?;
			for statement in statements {
				check_statement(context, statement)?;
			}
		},
		crate::ast::Statement::Return(expression) => {
			check_expression(context, expression)?;
		},
		crate::ast::Statement::Expression(expression) => {
			check_expression(context, expression)?;
		},
	}
	Ok(())
}

fn check_expression<'a>(context: &mut Context<'a>, expression: &crate::ast::Expression<'a>) -> Result<Type, &'static str> {
	match expression {
		crate::ast::Expression::Number(s) => Ok(Type::Number),
		crate::ast::Expression::Name(s) => {
			match context.variables.get(s) {
				None => Err("undefined variable"),
				Some(ty) => Ok(ty.clone())
			}
		},
		crate::ast::Expression::ArithmeticExpression(expression) => {
			assert_type(context, &*expression.left, Type::Number)?;
			assert_type(context, &*expression.right, Type::Number)?;
			Ok(Type::Number)
		},
		crate::ast::Expression::RelationalExpression(expression) => {
			assert_type(context, &*expression.left, Type::Number)?;
			assert_type(context, &*expression.right, Type::Number)?;
			Ok(Type::Boolean)
		},
		crate::ast::Expression::Assign(name, expression) => {
			match **name {
				crate::ast::Expression::Name(s) => {
					let ty = check_expression(context, expression)?;
					context.variables.insert(s, ty.clone());
					Ok(ty)
				},
				_ => Err("left hand of an assignment must be a name"),
			}
		}
	}
}

fn assert_type<'a>(context: &mut Context<'a>, expression: &crate::ast::Expression<'a>, ty: Type) -> Result<(), &'static str> {
	if check_expression(context, expression)? == ty {
		Ok(())
	} else {
		Err("type mismatch")
	}
}
