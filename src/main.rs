mod parser;
mod ast;

use parser::{Parser, optional, repeat, not, peek, sequence, choice, Cursor};
use ast::Expression;

fn any_char(_c: char) -> bool {
	true
}

fn skip_comments<'a>(cursor: &mut Cursor<'a>) -> Result<(), Cursor<'a>> {
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

fn parse_expression<'a>(cursor: &mut Cursor<'a>, level: usize) -> Result<Expression<'a>, Cursor<'a>> {
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
		if let Ok(_) = cursor.parse('(') {
			skip_comments(cursor)?;
			let expression = parse_expression(cursor, 0)?;
			skip_comments(cursor)?;
			cursor.parse(')')?;
			Ok(expression)
		} else if let Ok(_) = cursor.parse(peek(identifier_start_char)) {
			let s = parse_identifier(cursor)?;
			Ok(Expression::Name(s))
		} else if let Ok(_) = cursor.parse(peek('0'..='9')) {
			let s = parse_number(cursor)?;
			Ok(Expression::Number(s))
		} else {
			cursor.error()
		}
	}
}

fn identifier_start_char(c: char) -> bool {
	('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
}
fn identifier_char(c: char) -> bool {
	identifier_start_char(c) || ('0'..='9').contains(&c)
}

fn parse_identifier<'a>(cursor: &mut Cursor<'a>) -> Result<&'a str, Cursor<'a>> {
	cursor.parse(sequence!(identifier_start_char, repeat(identifier_char)))
}

fn keyword(k: &'static str) -> impl Parser {
	sequence!(k, not(identifier_char))
}

fn parse_number<'a>(cursor: &mut Cursor<'a>) -> Result<&'a str, Cursor<'a>> {
	cursor.parse(repeat('0'..='9'))
}

fn parse_statement<'a>(cursor: &mut Cursor<'a>) -> Result<(), Cursor<'a>> {
	if let Ok(_) = cursor.parse(keyword("let")) {
		skip_comments(cursor)?;
		parse_identifier(cursor)?;
		skip_comments(cursor)?;
		cursor.parse(';')?;
	} else {
		let expression = parse_expression(cursor, 0)?;
		let result = interpret_expression(&expression);
		println!("result = {}", result);
		skip_comments(cursor)?;
		cursor.parse(';')?;
	}
	Ok(())
}

fn parse_toplevel<'a>(cursor: &mut Cursor<'a>) -> Result<(), Cursor<'a>> {
	if let Ok(_) = cursor.parse(keyword("class")) {
		skip_comments(cursor)?;
		parse_identifier(cursor)?;
		skip_comments(cursor)?;
		cursor.parse('{')?;
		skip_comments(cursor)?;
		cursor.parse('}')?;
		Ok(())
	} else if let Ok(_) = cursor.parse(keyword("func")) {
		skip_comments(cursor)?;
		parse_identifier(cursor)?;
		skip_comments(cursor)?;
		cursor.parse('(')?;
		skip_comments(cursor)?;
		while let Ok(_) = cursor.parse(not(')')) {
			parse_identifier(cursor)?;
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
		skip_comments(cursor)?;
		cursor.parse('{')?;
		skip_comments(cursor)?;
		while let Ok(_) = cursor.parse(not('}')) {
			parse_statement(cursor)?;
			skip_comments(cursor)?;
		}
		cursor.parse('}')?;
		Ok(())
	} else {
		cursor.error()
	}
}

fn parse_file<'a>(cursor: &mut Cursor<'a>) -> Result<(), Cursor<'a>> {
	skip_comments(cursor)?;
	while let Ok(_) = cursor.parse(peek(any_char)) {
		parse_toplevel(cursor)?;
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

fn print_error<W: std::io::Write>(cursor: &Cursor, mut write: W) -> std::io::Result<()> {
	writeln!(write, "{}:", bold(red("error")))?;
	let mut start = 0;
	let mut end = cursor.s.len();
	let mut num = 0;
	for (i, c) in cursor.s.char_indices() {
		if c == '\n' {
			if i < cursor.i {
				start = i + c.len_utf8();
				num += 1;
			} else {
				end = i;
				break;
			}
		}
	}
	let line = cursor.s.get(start..end).unwrap();
	writeln!(write, "{} | {}", num, line)?;
	write!(write, "{} | ", num)?;
	for (_, c) in line.char_indices().take_while(|(i, _)| start + *i < cursor.i) {
		let c = if c.is_whitespace() { c } else { ' ' };
		write!(write, "{}", c)?;
	}
	writeln!(write, "^")?;
	Ok(())
}

fn interpret_expression(expression: &Expression) -> f64 {
	match expression {
		Expression::Number(s) => s.parse().unwrap(),
		Expression::Name(s) => panic!(),
		Expression::ArithmeticExpression(expression) => {
			let left = interpret_expression(&expression.left);
			let right = interpret_expression(&expression.right);
			match expression.operation {
				ast::ArithmeticOperation::Add => left + right,
				ast::ArithmeticOperation::Subtract => left - right,
				ast::ArithmeticOperation::Multiply => left * right,
				ast::ArithmeticOperation::Divide => left / right,
				ast::ArithmeticOperation::Remainder => left % right,
			}
		},
		Expression::RelationalExpression(expression) => {
			fn to_f64(b: bool) -> f64 {
				if b { 1.0 } else { 0.0 }
			}
			let left = interpret_expression(&expression.left);
			let right = interpret_expression(&expression.right);
			match expression.operation {
				ast::RelationalOperation::Equal => to_f64(left == right),
				ast::RelationalOperation::NotEqual => to_f64(left != right),
				ast::RelationalOperation::LessThan => to_f64(left < right),
				ast::RelationalOperation::LessThanOrEqual => to_f64(left <= right),
				ast::RelationalOperation::GreaterThan => to_f64(left > right),
				ast::RelationalOperation::GreaterThanOrEqual => to_f64(left >= right),
			}
		},
	}
}

fn main() {
	match std::env::args().nth(1) {
		Some(arg) => {
			let file = std::fs::read_to_string(arg).unwrap();
			let mut cursor = Cursor::new(file.as_str());
			match parse_file(&mut cursor) {
				Ok(()) => println!("{}", bold(green("success"))),
				Err(c) => print_error(&c, std::io::stderr().lock()).unwrap(),
			}
		},
		None => eprintln!("{}: no input file", bold(red("error"))),
	}
}
