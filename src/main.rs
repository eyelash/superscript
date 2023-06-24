mod error;
mod parser;
mod ast;
mod interpreter;
mod type_checker;

use error::{Error, print_error};
use parser::{Parser, optional, repeat, not, peek, sequence, choice, Cursor, ParseResult};
use ast::Expression;

fn any_char(_c: char) -> bool {
	true
}

fn skip_comments<'a>(cursor: &mut Cursor<'a>) -> Result<(), Error> {
	cursor.parse(repeat(char::is_whitespace))?;
	loop {
		if let Ok(_) = cursor.parse("/*") {
			cursor.parse(repeat(sequence!(not("*/"), any_char)))?;
			cursor.parse("*/")?;
		} else if let Ok(_) = cursor.parse("//") {
			cursor.parse(repeat(sequence!(not('\n'), any_char)))?;
		} else {
			break;
		}
		cursor.parse(repeat(char::is_whitespace))?;
	}
	Ok(())
}

enum OperatorLevel {
	BinaryLeftToRight(&'static [BinaryOperator]),
	BinaryRightToLeft(&'static [BinaryOperator]),
	UnaryPrefix(&'static [UnaryOperator]),
	UnaryPostfix(&'static [UnaryOperator]),
}

struct BinaryOperator(&'static str, for <'a> fn(Expression<'a>, Expression<'a>) -> Expression<'a>);
struct UnaryOperator(&'static str, for <'a> fn(Expression<'a>) -> Expression<'a>);

use OperatorLevel::{BinaryLeftToRight, BinaryRightToLeft, UnaryPrefix, UnaryPostfix};

const OPERATORS: &'static [OperatorLevel] = &[
	BinaryRightToLeft(&[
		BinaryOperator("=", Expression::assign),
	]),
	BinaryLeftToRight(&[
		BinaryOperator("==", Expression::equal),
		BinaryOperator("!=", Expression::not_equal),
	]),
	BinaryLeftToRight(&[
		BinaryOperator("<=", Expression::less_than_or_equal),
		BinaryOperator("<", Expression::less_than),
		BinaryOperator(">=", Expression::greater_than),
		BinaryOperator(">", Expression::greater_than_or_equal),
	]),
	BinaryLeftToRight(&[
		BinaryOperator("+", Expression::add),
		BinaryOperator("-", Expression::subtract),
	]),
	BinaryLeftToRight(&[
		BinaryOperator("*", Expression::multiply),
		BinaryOperator("/", Expression::divide),
		BinaryOperator("%", Expression::remainder),
	]),
];

fn parse_expression<'a>(cursor: &mut Cursor<'a>, level: usize) -> Result<Expression<'a>, Error> {
	fn parse_binary_operator<'a>(cursor: &mut Cursor<'a>, operators: &'static [BinaryOperator]) -> Option<&'static BinaryOperator> {
		for operator in operators {
			if let Ok(_) = cursor.parse(operator.0) {
				return Some(operator);
			}
		}
		return None;
	}
	fn parse_unary_operator<'a>(cursor: &mut Cursor<'a>, operators: &'static [UnaryOperator]) -> Option<&'static UnaryOperator> {
		for operator in operators {
			if let Ok(_) = cursor.parse(operator.0) {
				return Some(operator);
			}
		}
		return None;
	}
	if level < OPERATORS.len() {
		match OPERATORS[level] {
			BinaryLeftToRight(operators) => {
				let mut left = parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				while let Some(operator) = parse_binary_operator(cursor, operators) {
					skip_comments(cursor)?;
					let right = parse_expression(cursor, level + 1)?;
					left = operator.1(left, right);
					skip_comments(cursor)?;
				}
				Ok(left)
			},
			BinaryRightToLeft(operators) => {
				let left = parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				if let Some(operator) = parse_binary_operator(cursor, operators) {
					skip_comments(cursor)?;
					let right = parse_expression(cursor, level)?;
					Ok(operator.1(left, right))
				} else {
					Ok(left)
				}
			},
			UnaryPrefix(operators) => {
				if let Some(operator) = parse_unary_operator(cursor, operators) {
					skip_comments(cursor)?;
					let expression = parse_expression(cursor, level)?;
					Ok(operator.1(expression))
				} else {
					parse_expression(cursor, level + 1)
				}
			},
			UnaryPostfix(operators) => {
				let mut expression = parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				while let Some(operator) = parse_unary_operator(cursor, operators) {
					expression = operator.1(expression);
					skip_comments(cursor)?;
				}
				Ok(expression)
			},
		}
	} else {
		let mut expression = if let Ok(_) = cursor.parse('(') {
			skip_comments(cursor)?;
			let expression = parse_expression(cursor, 0)?;
			skip_comments(cursor)?;
			cursor.parse(')')?;
			expression
		} else if let Ok(_) = cursor.parse(peek(identifier_start_char)) {
			let s = parse_identifier(cursor)?;
			Expression::Name(s)
		} else if let Ok(_) = cursor.parse(peek('0'..='9')) {
			let s = parse_number(cursor)?;
			Expression::Number(s)
		} else {
			return cursor.error();
		};
		skip_comments(cursor)?;
		while let Ok(_) = cursor.parse('(') {
			let mut arguments = Vec::new();
			skip_comments(cursor)?;
			while let Ok(_) = cursor.parse(not(')')) {
				arguments.push(parse_expression(cursor, 0)?);
				skip_comments(cursor)?;
				match cursor.parse(',') {
					Ok(_) => {
						skip_comments(cursor)?;
						continue
					}
					Err(_) => break
				}
			}
			cursor.parse(')')?;
			// TODO: call
			skip_comments(cursor)?;
		}
		Ok(expression)
	}
}

fn identifier_start_char(c: char) -> bool {
	('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
}
fn identifier_char(c: char) -> bool {
	identifier_start_char(c) || ('0'..='9').contains(&c)
}

fn parse_identifier<'a>(cursor: &mut Cursor<'a>) -> Result<&'a str, Error> {
	cursor.parse(sequence!(identifier_start_char, repeat(identifier_char)))
}

fn keyword(k: &'static str) -> impl Parser {
	sequence!(k, not(identifier_char))
}

fn parse_number<'a>(cursor: &mut Cursor<'a>) -> Result<&'a str, Error> {
	cursor.parse(repeat('0'..='9'))
}

fn parse_statement<'a>(cursor: &mut Cursor<'a>) -> Result<ast::Statement<'a>, Error> {
	if let Ok(_) = cursor.parse(keyword("let")) {
		skip_comments(cursor)?;
		parse_identifier(cursor)?;
		skip_comments(cursor)?;
		cursor.expect("=")?;
		skip_comments(cursor)?;
		let expression = parse_expression(cursor, 0)?;
		skip_comments(cursor)?;
		cursor.expect(";")?;
		Ok(ast::Statement::Expression(expression))
	} else if let Ok(_) = cursor.parse(keyword("if")) {
		skip_comments(cursor)?;
		cursor.expect("(")?;
		skip_comments(cursor)?;
		let condition = parse_expression(cursor, 0)?;
		skip_comments(cursor)?;
		cursor.expect(")")?;
		skip_comments(cursor)?;
		cursor.expect("{")?;
		skip_comments(cursor)?;
		let mut statements = Vec::new();
		while let Ok(_) = cursor.parse(not('}')) {
			statements.push(parse_statement(cursor)?);
			skip_comments(cursor)?;
		}
		cursor.expect("}")?;
		Ok(ast::Statement::If(ast::If {
			condition: Box::new(condition),
			statements,
		}))
	} else if let Ok(_) = cursor.parse(keyword("while")) {
		skip_comments(cursor)?;
		cursor.expect("(")?;
		skip_comments(cursor)?;
		let condition = parse_expression(cursor, 0)?;
		skip_comments(cursor)?;
		cursor.expect(")")?;
		skip_comments(cursor)?;
		cursor.expect("{")?;
		skip_comments(cursor)?;
		let mut statements = Vec::new();
		while let Ok(_) = cursor.parse(not('}')) {
			statements.push(parse_statement(cursor)?);
			skip_comments(cursor)?;
		}
		cursor.expect("}")?;
		Ok(ast::Statement::While(ast::While {
			condition: Box::new(condition),
			statements,
		}))
	} else if let Ok(_) = cursor.parse(keyword("return")) {
		skip_comments(cursor)?;
		let expression = parse_expression(cursor, 0)?;
		skip_comments(cursor)?;
		cursor.expect(";")?;
		Ok(ast::Statement::Return(expression))
	} else {
		let expression = parse_expression(cursor, 0)?;
		skip_comments(cursor)?;
		cursor.expect(";")?;
		Ok(ast::Statement::Expression(expression))
	}
}

fn parse_toplevel<'a>(program: &mut ast::Program<'a>, cursor: &mut Cursor<'a>) -> Result<(), Error> {
	if let Ok(_) = cursor.parse(keyword("class")) {
		skip_comments(cursor)?;
		parse_identifier(cursor)?;
		skip_comments(cursor)?;
		cursor.expect("{")?;
		skip_comments(cursor)?;
		cursor.expect("}")?;
		Ok(())
	} else if let Ok(_) = cursor.parse(keyword("func")) {
		skip_comments(cursor)?;
		let name = parse_identifier(cursor)?;
		skip_comments(cursor)?;
		cursor.expect("(")?;
		skip_comments(cursor)?;
		let mut arguments = Vec::new();
		while let Ok(_) = cursor.parse(not(')')) {
			arguments.push(parse_identifier(cursor)?);
			skip_comments(cursor)?;
			match cursor.parse(',') {
				Ok(_) => {
					skip_comments(cursor)?;
					continue
				}
				Err(_) => break
			}
		}
		cursor.expect(")")?;
		skip_comments(cursor)?;
		cursor.expect("{")?;
		skip_comments(cursor)?;
		let mut statements = Vec::new();
		while let Ok(_) = cursor.parse(not('}')) {
			statements.push(parse_statement(cursor)?);
			skip_comments(cursor)?;
		}
		cursor.expect("}")?;
		program.functions.push(crate::ast::Function {
			name,
			arguments,
			statements,
		});
		Ok(())
	} else {
		cursor.error()
	}
}

fn parse_file<'a>(program: &mut ast::Program<'a>, cursor: &mut Cursor<'a>) -> Result<(), Error> {
	skip_comments(cursor)?;
	while let Ok(_) = cursor.parse(peek(any_char)) {
		parse_toplevel(program, cursor)?;
		skip_comments(cursor)?;
	}
	Ok(())
}

struct Bold<T>(T);

impl <T: std::fmt::Display> std::fmt::Display for Bold<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "\x1B[1m{}\x1B[22m", self.0)?;
		Ok(())
	}
}

fn bold<T: std::fmt::Display>(t: T) -> Bold<T> {
	Bold(t)
}

struct Red<T>(T);

impl <T: std::fmt::Display> std::fmt::Display for Red<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "\x1B[31m{}\x1B[39m", self.0)?;
		Ok(())
	}
}

fn red<T: std::fmt::Display>(t: T) -> Red<T> {
	Red(t)
}

struct Green<T>(T);

impl <T: std::fmt::Display> std::fmt::Display for Green<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "\x1B[32m{}\x1B[39m", self.0)?;
		Ok(())
	}
}

fn green<T: std::fmt::Display>(t: T) -> Green<T> {
	Green(t)
}

fn main() {
	match std::env::args().nth(1) {
		Some(arg) => {
			let file = std::fs::read_to_string(arg).unwrap();
			let mut cursor = Cursor::new(file.as_str());
			let mut program = ast::Program::new();
			match parse_file(&mut program, &mut cursor) {
				Ok(()) => {
					match type_checker::type_check(&program) {
						Ok(_) => println!("{}", bold(green("type check successful"))),
						Err(e) => print_error(&e, file.as_str(), std::io::stderr().lock()).unwrap(),
					}
					interpreter::interpret_program(&program)
				},
				Err(e) => print_error(&e, file.as_str(), std::io::stderr().lock()).unwrap(),
			}
		},
		None => eprintln!("{}: no input file", bold(red("error"))),
	}
}
