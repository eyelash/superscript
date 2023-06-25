mod error;
mod parser;
mod ast;
mod interpreter;
mod type_checker;

use error::Error;
use parser::{Parser, optional, repeat, not, peek, sequence, choice, ParseResult};
use ast::Expression;

struct Cursor<'a> {
	cursor: parser::Cursor<'a>,
	program: ast::Program<'a>,
}

impl <'a> Cursor<'a> {
	pub fn new(s: &'a str) -> Self {
		Cursor {
			cursor: parser::Cursor::new(s),
			program: ast::Program::new(),
		}
	}
	pub fn error<T, S: Into<String>>(&self, msg: S) -> Result<T, Error> {
		self.cursor.error(msg)
	}
	pub fn parse<P: Parser>(&mut self, mut p: P) -> Result<&'a str, Error> {
		self.cursor.parse(p)
	}
	pub fn expect(&mut self, s: &str) -> Result<(), Error> {
		self.cursor.expect(s)
	}
	pub fn get_location(&self) -> usize {
		self.cursor.get_location()
	}
	pub fn mark_location(&mut self, expression: Box<Expression<'a>>, location: usize) -> Box<Expression<'a>> {
		self.program.locations.insert(&*expression, location);
		expression
	}
}

fn any_char(_c: char) -> bool {
	true
}

fn skip_comments<'a>(cursor: &mut Cursor<'a>) -> Result<(), Error> {
	cursor.parse(repeat(char::is_whitespace))?;
	loop {
		if let Ok(_) = cursor.parse("/*") {
			cursor.parse(repeat(sequence!(not("*/"), any_char)))?;
			cursor.expect("*/")?;
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

type BinaryOperatorFunction = for <'a> fn(Box<Expression<'a>>, Box<Expression<'a>>) -> Box<Expression<'a>>;
type UnaryOperatorFunction = for <'a> fn(Box<Expression<'a>>) -> Box<Expression<'a>>;
struct BinaryOperator(&'static str, BinaryOperatorFunction);
struct UnaryOperator(&'static str, UnaryOperatorFunction);

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

fn parse_expression<'a>(cursor: &mut Cursor<'a>, level: usize) -> Result<Box<Expression<'a>>, Error> {
	fn parse_binary_operator<'a>(cursor: &mut Cursor<'a>, operators: &'static [BinaryOperator]) -> Option<(BinaryOperatorFunction, usize)> {
		let location = cursor.get_location();
		for operator in operators {
			if let Ok(_) = cursor.parse(operator.0) {
				return Some((operator.1, location));
			}
		}
		return None;
	}
	fn parse_unary_operator<'a>(cursor: &mut Cursor<'a>, operators: &'static [UnaryOperator]) -> Option<(UnaryOperatorFunction, usize)> {
		let location = cursor.get_location();
		for operator in operators {
			if let Ok(_) = cursor.parse(operator.0) {
				return Some((operator.1, location));
			}
		}
		return None;
	}
	if level < OPERATORS.len() {
		match OPERATORS[level] {
			BinaryLeftToRight(operators) => {
				let mut left = parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				while let Some((operator, location)) = parse_binary_operator(cursor, operators) {
					skip_comments(cursor)?;
					let right = parse_expression(cursor, level + 1)?;
					left = cursor.mark_location(operator(left, right), location);
					skip_comments(cursor)?;
				}
				Ok(left)
			},
			BinaryRightToLeft(operators) => {
				let left = parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				if let Some((operator, location)) = parse_binary_operator(cursor, operators) {
					skip_comments(cursor)?;
					let right = parse_expression(cursor, level)?;
					Ok(cursor.mark_location(operator(left, right), location))
				} else {
					Ok(left)
				}
			},
			UnaryPrefix(operators) => {
				if let Some((operator, location)) = parse_unary_operator(cursor, operators) {
					skip_comments(cursor)?;
					let expression = parse_expression(cursor, level)?;
					Ok(cursor.mark_location(operator(expression), location))
				} else {
					parse_expression(cursor, level + 1)
				}
			},
			UnaryPostfix(operators) => {
				let mut expression = parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				while let Some((operator, location)) = parse_unary_operator(cursor, operators) {
					expression = cursor.mark_location(operator(expression), location);
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
			cursor.expect(")")?;
			expression
		} else if let Ok(_) = cursor.parse(peek(identifier_start_char)) {
			let location = cursor.get_location();
			let s = parse_identifier(cursor)?;
			cursor.mark_location(Box::new(Expression::Name(s)), location)
		} else if let Ok(_) = cursor.parse(peek('0'..='9')) {
			let location = cursor.get_location();
			let s = parse_number(cursor)?;
			cursor.mark_location(Box::new(Expression::Number(s)), location)
		} else {
			return cursor.error("expected an expression");
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
			condition,
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
			condition,
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

fn parse_toplevel<'a>(cursor: &mut Cursor<'a>) -> Result<(), Error> {
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
		cursor.program.functions.push(crate::ast::Function {
			name,
			arguments,
			statements,
		});
		Ok(())
	} else {
		cursor.error("expected a toplevel declaration")
	}
}

fn parse_file<'a>(mut cursor: Cursor<'a>) -> Result<ast::Program<'a>, Error> {
	skip_comments(&mut cursor)?;
	while let Ok(_) = cursor.parse(peek(any_char)) {
		parse_toplevel(&mut cursor)?;
		skip_comments(&mut cursor)?;
	}
	Ok(cursor.program)
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
			let cursor = Cursor::new(file.as_str());
			match parse_file(cursor) {
				Ok(program) => {
					match type_checker::type_check(&program) {
						Ok(_) => {
							println!("{}", bold(green("type check successful")));
							interpreter::interpret_program(&program);
						},
						Err(e) => e.print(file.as_str(), std::io::stderr().lock()).unwrap(),
					}
				},
				Err(e) => e.print(file.as_str(), std::io::stderr().lock()).unwrap(),
			}
		},
		None => eprintln!("{}: no input file", bold(red("error"))),
	}
}
