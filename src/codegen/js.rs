use crate::printer::{Printer, comma_separated};
use crate::ast::{Program, Function, Class, Statement, Expression};

pub fn generate<W: std::io::Write>(printer: &mut Printer<W>, program: &Program) {
	for function in &program.functions {
		generate_function(printer, function);
	}
	for class in &program.classes {
		generate_class(printer, class);
	}
}

fn generate_function<W: std::io::Write>(printer: &mut Printer<W>, function: &Function) {
	let arguments = function.arguments.iter().map(|(name, _)| name);
	printer.println(format_args!("function {}({}) {{", function.name, comma_separated(arguments)));
	printer.indented(|printer| {
		for statement in &function.statements {
			generate_statement(printer, statement);
		}
	});
	printer.println("}");
}

fn generate_class<W: std::io::Write>(printer: &mut Printer<W>, class: &Class) {
	printer.println(format_args!("class {} {{", class.name));
	printer.indented(|printer| {
		for (name, ty) in &class.fields {
			
		}
		for method in &class.methods {
			generate_method(printer, method);
		}
	});
	printer.println("}");
}

fn generate_method<W: std::io::Write>(printer: &mut Printer<W>, function: &Function) {
	let arguments = function.arguments.iter().map(|(name, _)| name);
	printer.println(format_args!("{}({}) {{", function.name, comma_separated(arguments)));
	printer.indented(|printer| {
		for statement in &function.statements {
			generate_statement(printer, statement);
		}
	});
	printer.println("}");
}

fn generate_statement<W: std::io::Write>(printer: &mut Printer<W>, statement: &Statement) {
	match statement {
		Statement::VariableDeclaration { name, expression } => {
			printer.println(format_args!("let {} = {};", name, DisplayExpression(expression)));
		},
		Statement::If(crate::ast::If{condition, statement, else_statement}) => {
			printer.println(format_args!("if ({})", DisplayExpression(condition)));
			printer.indented(|printer| generate_statement(printer, statement));
			if let Some(statement) = else_statement {
				printer.println("else");
				printer.indented(|printer| generate_statement(printer, statement));
			}
		},
		Statement::While(crate::ast::While{condition, statement}) => {
			printer.println(format_args!("while ({})", DisplayExpression(condition)));
			printer.indented(|printer| generate_statement(printer, statement));
		},
		Statement::Return(expression) => {
			printer.println(format_args!("return {};", DisplayExpression(expression)));
		},
		Statement::Expression(expression) => {
			printer.println(format_args!("{};", DisplayExpression(expression)));
		},
		Statement::Block(statements) => {
			printer.println('{');
			printer.indented(|printer| {
				for statement in statements {
					generate_statement(printer, statement);
				}
			});
			printer.println('}');
		},
	}
}

struct DisplayExpression<'a>(&'a Expression<'a>);

impl <'a> std::fmt::Display for DisplayExpression<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self.0 {
			Expression::Number(s) => write!(f, "{}", s)?,
			Expression::Name(s) => write!(f, "{}", s)?,
			Expression::ArithmeticExpression(e) => {
				use crate::ast::ArithmeticOperation::*;
				let operation = match e.operation {
					Add => "+",
					Subtract => "-",
					Multiply => "*",
					Divide => "/",
					Remainder => "%",
				};
				write!(f, "({} {} {})", DisplayExpression(&e.left), operation, DisplayExpression(&e.right))?;
			},
			Expression::RelationalExpression(e) => {
				use crate::ast::RelationalOperation::*;
				let operation = match e.operation {
					Equal => "===",
					NotEqual => "!==",
					LessThan => "<",
					LessThanOrEqual => "<=",
					GreaterThan => ">",
					GreaterThanOrEqual => ">=",
				};
				write!(f, "({} {} {})", DisplayExpression(&e.left), operation, DisplayExpression(&e.right))?;
			},
			Expression::LogicalExpression(e) => {
				use crate::ast::LogicalOperation::*;
				let operation = match e.operation {
					And => "&&",
					Or => "||",
				};
				write!(f, "({} {} {})", DisplayExpression(&e.left), operation, DisplayExpression(&e.right))?;
			},
			Expression::Not(e) => write!(f, "!{}", DisplayExpression(e))?,
			Expression::Assign { name, expression } => {
				write!(f, "({} = {})", DisplayExpression(name), DisplayExpression(expression))?;
			},
			Expression::Call { function, arguments } => {
				let arguments = arguments.iter().map(|argument| DisplayExpression(argument));
				write!(f, "{}({})", DisplayExpression(function), comma_separated(arguments))?;
			},
			Expression::ClassInstantiation { class, arguments } => {
				let arguments = arguments.iter().map(|argument| DisplayExpression(argument));
				write!(f, "new {}({})", class, comma_separated(arguments))?;
			},
			Expression::PropertyAccess { object, property } => {
				write!(f, "{}.{}", DisplayExpression(object), property)?;
			},
			Expression::MethodCall { object, method, arguments } => {
				let arguments = arguments.iter().map(|argument| DisplayExpression(argument));
				write!(f, "{}.{}({})", DisplayExpression(object), method, comma_separated(arguments))?;
			},
			Expression::This => write!(f, "this")?,
		};
		Ok(())
	}
}
