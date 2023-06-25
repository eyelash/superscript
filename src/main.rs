mod error;
mod parser;
mod printer;
mod ast;
mod type_checker;

use error::{Error, Location};
use parser::{Parse, optional, repeat, not, peek, sequence, choice, ParseResult};
use printer::{bold, red, green};
use ast::Expression;

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
		BinaryOperator("||", Expression::or),
	]),
	BinaryLeftToRight(&[
		BinaryOperator("&&", Expression::and),
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
	UnaryPrefix(&[
		UnaryOperator("!", Expression::not),
	]),
];

struct Cursor<'a> {
	cursor: parser::Cursor<'a>,
	program: ast::Program<'a>,
}

impl <'a> Cursor<'a> {
	fn new(s: &'a str) -> Self {
		Cursor {
			cursor: parser::Cursor::new(s),
			program: ast::Program::new(),
		}
	}
	fn error<T, S: Into<String>>(&self, msg: S) -> Result<T, Error> {
		self.cursor.error(msg)
	}
	fn parse<P: Parse>(&mut self, mut p: P) -> Result<(&'a str, Location), Error> {
		self.cursor.parse(p)
	}
	fn expect(&mut self, s: &str) -> Result<(), Error> {
		self.cursor.expect(s)
	}
	fn mark_location(&mut self, expression: Box<Expression<'a>>, location: Location) -> Box<Expression<'a>> {
		self.program.locations.insert(&*expression, location);
		expression
	}
	fn skip_comments(&mut self) -> Result<(), Error> {
		self.parse(repeat(char::is_whitespace))?;
		loop {
			if let Ok(_) = self.parse("/*") {
				self.parse(repeat(sequence!(not("*/"), any_char)))?;
				self.expect("*/")?;
			} else if let Ok(_) = self.parse("//") {
				self.parse(repeat(sequence!(not('\n'), any_char)))?;
			} else {
				break;
			}
			self.parse(repeat(char::is_whitespace))?;
		}
		Ok(())
	}
	fn parse_expression(&mut self, level: usize) -> Result<Box<Expression<'a>>, Error> {
		fn parse_binary_operator<'a>(cursor: &mut Cursor<'a>, operators: &'static [BinaryOperator]) -> Option<(BinaryOperatorFunction, Location)> {
			for operator in operators {
				if let Ok((_, location)) = cursor.parse(operator.0) {
					return Some((operator.1, location));
				}
			}
			return None;
		}
		fn parse_unary_operator<'a>(cursor: &mut Cursor<'a>, operators: &'static [UnaryOperator]) -> Option<(UnaryOperatorFunction, Location)> {
			for operator in operators {
				if let Ok((_, location)) = cursor.parse(operator.0) {
					return Some((operator.1, location));
				}
			}
			return None;
		}
		if level < OPERATORS.len() {
			match OPERATORS[level] {
				BinaryLeftToRight(operators) => {
					let mut left = self.parse_expression(level + 1)?;
					self.skip_comments()?;
					while let Some((operator, location)) = parse_binary_operator(self, operators) {
						self.skip_comments()?;
						let right = self.parse_expression(level + 1)?;
						left = self.mark_location(operator(left, right), location);
						self.skip_comments()?;
					}
					Ok(left)
				},
				BinaryRightToLeft(operators) => {
					let left = self.parse_expression(level + 1)?;
					self.skip_comments()?;
					if let Some((operator, location)) = parse_binary_operator(self, operators) {
						self.skip_comments()?;
						let right = self.parse_expression(level)?;
						Ok(self.mark_location(operator(left, right), location))
					} else {
						Ok(left)
					}
				},
				UnaryPrefix(operators) => {
					if let Some((operator, location)) = parse_unary_operator(self, operators) {
						self.skip_comments()?;
						let expression = self.parse_expression(level)?;
						Ok(self.mark_location(operator(expression), location))
					} else {
						self.parse_expression(level + 1)
					}
				},
				UnaryPostfix(operators) => {
					let mut expression = self.parse_expression(level + 1)?;
					self.skip_comments()?;
					while let Some((operator, location)) = parse_unary_operator(self, operators) {
						expression = self.mark_location(operator(expression), location);
						self.skip_comments()?;
					}
					Ok(expression)
				},
			}
		} else {
			let mut expression = if let Ok(_) = self.parse('(') {
				self.skip_comments()?;
				let expression = self.parse_expression(0)?;
				self.skip_comments()?;
				self.expect(")")?;
				expression
			} else if let Ok(_) = self.parse(peek(identifier_start_char)) {
				let (s, location) = self.parse_identifier()?;
				self.mark_location(Box::new(Expression::Name(s)), location)
			} else if let Ok(_) = self.parse(peek('0'..='9')) {
				let (s, location) = self.parse_number()?;
				self.mark_location(Box::new(Expression::Number(s)), location)
			} else {
				return self.error("expected an expression");
			};
			self.skip_comments()?;
			while let Ok(_) = self.parse('(') {
				let mut arguments = Vec::new();
				self.skip_comments()?;
				while let Ok(_) = self.parse(not(')')) {
					arguments.push(self.parse_expression(0)?);
					self.skip_comments()?;
					match self.parse(',') {
						Ok(_) => {
							self.skip_comments()?;
							continue
						}
						Err(_) => break
					}
				}
				self.parse(')')?;
				expression = Box::new(Expression::Call {
					function: expression,
					arguments,
				});
				self.skip_comments()?;
			}
			Ok(expression)
		}
	}
	fn parse_identifier(&mut self) -> Result<(&'a str, Location), Error> {
		self.parse(sequence!(identifier_start_char, repeat(identifier_char)))
	}
	fn parse_number(&mut self) -> Result<(&'a str, Location), Error> {
		self.parse(repeat('0'..='9'))
	}
	fn parse_type(&mut self) -> Result<(ast::Type, Location), Error> {
		if let Ok((_, location)) = self.parse(keyword("number")) {
			Ok((ast::Type::Number, location))
		} else if let Ok((_, location)) = self.parse(keyword("boolean")) {
			Ok((ast::Type::Boolean, location))
		} else {
			self.error("expected a type")
		}
	}
	fn parse_statement(&mut self) -> Result<ast::Statement<'a>, Error> {
		if let Ok(_) = self.parse(keyword("let")) {
			self.skip_comments()?;
			self.parse_identifier()?;
			self.skip_comments()?;
			self.expect("=")?;
			self.skip_comments()?;
			let expression = self.parse_expression(0)?;
			self.skip_comments()?;
			self.expect(";")?;
			Ok(ast::Statement::Expression(expression))
		} else if let Ok(_) = self.parse(keyword("if")) {
			self.skip_comments()?;
			self.expect("(")?;
			self.skip_comments()?;
			let condition = self.parse_expression(0)?;
			self.skip_comments()?;
			self.expect(")")?;
			self.skip_comments()?;
			let statement = Box::new(self.parse_statement()?);
			Ok(ast::Statement::If(ast::If {
				condition,
				statement,
			}))
		} else if let Ok(_) = self.parse(keyword("while")) {
			self.skip_comments()?;
			self.expect("(")?;
			self.skip_comments()?;
			let condition = self.parse_expression(0)?;
			self.skip_comments()?;
			self.expect(")")?;
			self.skip_comments()?;
			let statement = Box::new(self.parse_statement()?);
			Ok(ast::Statement::While(ast::While {
				condition,
				statement,
			}))
		} else if let Ok(_) = self.parse(keyword("return")) {
			self.skip_comments()?;
			let expression = self.parse_expression(0)?;
			self.skip_comments()?;
			self.expect(";")?;
			Ok(ast::Statement::Return(expression))
		} else if let Ok(_) = self.parse('{') {
			self.skip_comments()?;
			let mut statements = Vec::new();
			while let Ok(_) = self.parse(not('}')) {
				statements.push(self.parse_statement()?);
				self.skip_comments()?;
			}
			self.expect("}")?;
			Ok(ast::Statement::Block(statements))
		} else {
			let expression = self.parse_expression(0)?;
			self.skip_comments()?;
			self.expect(";")?;
			Ok(ast::Statement::Expression(expression))
		}
	}
	fn parse_toplevel(&mut self) -> Result<(), Error> {
		if let Ok(_) = self.parse(keyword("class")) {
			self.skip_comments()?;
			self.parse_identifier()?;
			self.skip_comments()?;
			self.expect("{")?;
			self.skip_comments()?;
			self.expect("}")?;
			Ok(())
		} else if let Ok(_) = self.parse(keyword("function")) {
			self.skip_comments()?;
			let (name, _) = self.parse_identifier()?;
			self.skip_comments()?;
			self.expect("(")?;
			self.skip_comments()?;
			let mut arguments = Vec::new();
			while let Ok(_) = self.parse(not(')')) {
				let (name, _) = self.parse_identifier()?;
				self.skip_comments()?;
				self.expect(":")?;
				self.skip_comments()?;
				let (ty, _) = self.parse_type()?;
				arguments.push((name, ty));
				self.skip_comments()?;
				match self.parse(',') {
					Ok(_) => {
						self.skip_comments()?;
						continue
					}
					Err(_) => break
				}
			}
			self.expect(")")?;
			self.skip_comments()?;
			let return_type = if let Ok(_) = self.parse(':') {
				self.skip_comments()?;
				let (ty, _) = self.parse_type()?;
				self.skip_comments()?;
				ty
			} else {
				ast::Type::Void
			};
			self.expect("{")?;
			self.skip_comments()?;
			let mut statements = Vec::new();
			while let Ok(_) = self.parse(not('}')) {
				statements.push(self.parse_statement()?);
				self.skip_comments()?;
			}
			self.expect("}")?;
			self.program.functions.push(crate::ast::Function {
				name,
				arguments,
				return_type,
				statements,
			});
			Ok(())
		} else {
			self.error("expected a toplevel declaration")
		}
	}
}

fn any_char(_c: char) -> bool {
	true
}

fn identifier_start_char(c: char) -> bool {
	('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
}
fn identifier_char(c: char) -> bool {
	identifier_start_char(c) || ('0'..='9').contains(&c)
}

fn keyword(k: &'static str) -> impl Parse {
	sequence!(k, not(identifier_char))
}

fn parse_file<'a>(mut cursor: Cursor<'a>) -> Result<ast::Program<'a>, Error> {
	cursor.skip_comments()?;
	while let Ok(_) = cursor.parse(peek(any_char)) {
		cursor.parse_toplevel()?;
		cursor.skip_comments()?;
	}
	Ok(cursor.program)
}

fn main() {
	match std::env::args().nth(1) {
		Some(arg) => {
			let file = std::fs::read_to_string(arg).unwrap();
			let cursor = Cursor::new(file.as_str());
			match parse_file(cursor) {
				Ok(program) => {
					match type_checker::type_check(&program) {
						Ok(_) => println!("{}", bold(green("type check successful"))),
						Err(e) => e.print(file.as_str(), std::io::stderr().lock()).unwrap(),
					}
				},
				Err(e) => e.print(file.as_str(), std::io::stderr().lock()).unwrap(),
			}
		},
		None => eprintln!("{}: no input file", bold(red("error"))),
	}
}
