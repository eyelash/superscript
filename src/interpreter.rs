use std::collections::HashMap;

#[derive(Clone, Debug)]
enum Value {
	Number(f64),
	Boolean(bool),
	Void,
}

struct Context<'a> {
	variables: HashMap<&'a str, Value>,
}

impl <'a> Context<'a> {
	fn lookup(&self, name: &'a str) -> Value {
		self.variables.get(name).cloned().unwrap()
	}
	fn set_variable(&mut self, name: &'a str, value: Value) {
		self.variables.insert(name, value);
	}
}

pub fn interpret_program(program: &crate::ast::Program) {
	if let Some(main_function) = program.get_main_function() {
		let mut context = Context {
			variables: HashMap::new(),
		};
		for statement in &main_function.statements {
			interpret_statement(&mut context, statement);
		}
	}
}

fn interpret_statement<'a>(context: &mut Context<'a>, statement: &crate::ast::Statement<'a>) {
	fn is_true(value: Value) -> bool {
		match value {
			Value::Boolean(b) => b,
			_ => panic!(),
		}
	}
	match statement {
		crate::ast::Statement::If(crate::ast::If{condition, statements}) => {
			if is_true(interpret_expression(context, condition)) {
				for statement in statements {
					interpret_statement(context, statement);
				}
			}
		},
		crate::ast::Statement::While(crate::ast::While{condition, statements}) => {
			while is_true(interpret_expression(context, condition)) {
				for statement in statements {
					interpret_statement(context, statement);
				}
			}
		},
		crate::ast::Statement::Return(expression) => {
			let result = interpret_expression(context, expression);
			println!("{:?}", result);
		},
		crate::ast::Statement::Expression(expression) => {
			interpret_expression(context, expression);
		},
	}
}

fn interpret_expression<'a>(context: &mut Context<'a>, expression: &crate::ast::Expression<'a>) -> Value {
	fn to_f64(value: Value) -> f64 {
		match value {
			Value::Number(f) => f,
			_ => panic!(),
		}
	}
	match expression {
		crate::ast::Expression::Number(s) => Value::Number(s.parse().unwrap()),
		crate::ast::Expression::Name(s) => context.lookup(s),
		crate::ast::Expression::ArithmeticExpression(expression) => {
			let left = to_f64(interpret_expression(context, &expression.left));
			let right = to_f64(interpret_expression(context, &expression.right));
			Value::Number(match expression.operation {
				crate::ast::ArithmeticOperation::Add => left + right,
				crate::ast::ArithmeticOperation::Subtract => left - right,
				crate::ast::ArithmeticOperation::Multiply => left * right,
				crate::ast::ArithmeticOperation::Divide => left / right,
				crate::ast::ArithmeticOperation::Remainder => left % right,
			})
		},
		crate::ast::Expression::RelationalExpression(expression) => {
			let left = to_f64(interpret_expression(context, &expression.left));
			let right = to_f64(interpret_expression(context, &expression.right));
			Value::Boolean(match expression.operation {
				crate::ast::RelationalOperation::Equal => left == right,
				crate::ast::RelationalOperation::NotEqual => left != right,
				crate::ast::RelationalOperation::LessThan => left < right,
				crate::ast::RelationalOperation::LessThanOrEqual => left <= right,
				crate::ast::RelationalOperation::GreaterThan => left > right,
				crate::ast::RelationalOperation::GreaterThanOrEqual => left >= right,
			})
		},
		crate::ast::Expression::Assign(name, expression) => {
			let name = match **name {
				crate::ast::Expression::Name(name) => name,
				_ => panic!(),
			};
			let value = interpret_expression(context, expression);
			context.set_variable(name, value.clone());
			value
		}
	}
}
