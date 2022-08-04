mod parser;

use parser::{Parser, optional, repeat, not, peek, sequence, choice, Cursor};

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
	BinaryLeftToRight(&'static [Operator]),
	BinaryRightToLeft(&'static [Operator]),
	UnaryPrefix(&'static [Operator]),
	UnaryPostfix(&'static [Operator]),
}

struct Operator(&'static str);

use OperatorLevel::{BinaryLeftToRight, BinaryRightToLeft, UnaryPrefix, UnaryPostfix};

const OPERATORS: &'static [OperatorLevel] = &[
	BinaryLeftToRight(&[Operator("*"), Operator("/"), Operator("%")]),
	BinaryLeftToRight(&[Operator("+"), Operator("-")]),
	BinaryLeftToRight(&[Operator("<="), Operator("<"), Operator(">="), Operator(">")]),
	BinaryLeftToRight(&[Operator("=="), Operator("!=")]),
];

fn parse_expression<'a>(cursor: &mut Cursor<'a>, level: usize) -> Result<(), Cursor<'a>> {
	fn parse_operator<'a>(cursor: &mut Cursor<'a>, operators: &'static [Operator]) -> Option<&'static str> {
		for op in operators {
			if let Ok(_) = cursor.parse(op.0) {
				return Some(op.0);
			}
		}
		return None;
	}
	if level < OPERATORS.len() {
		match OPERATORS[level] {
			BinaryLeftToRight(operators) => {
				parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				while let Some(_) = parse_operator(cursor, operators) {
					skip_comments(cursor)?;
					parse_expression(cursor, level + 1)?;
					skip_comments(cursor)?;
				}
			},
			BinaryRightToLeft(operators) => {
				parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				if let Some(_) = parse_operator(cursor, operators) {
					skip_comments(cursor)?;
					parse_expression(cursor, level)?;
				}
			},
			UnaryPrefix(operators) => {
				if let Some(_) = parse_operator(cursor, operators) {
					skip_comments(cursor)?;
					parse_expression(cursor, level)?;
				} else {
					parse_expression(cursor, level + 1)?;
				}
			},
			UnaryPostfix(operators) => {
				parse_expression(cursor, level + 1)?;
				skip_comments(cursor)?;
				while let Some(_) = parse_operator(cursor, operators) {
					skip_comments(cursor)?;
				}
			},
		}
	} else {
		if let Ok(_) = cursor.parse('(') {
			skip_comments(cursor)?;
			parse_expression(cursor, 0)?;
			skip_comments(cursor)?;
			cursor.parse(')')?;
		} else if let Ok(_) = cursor.parse(peek(identifier_start_char)) {
			parse_identifier(cursor)?;
		} else if let Ok(_) = cursor.parse(peek('0'..='9')) {
			parse_number(cursor)?;
		} else {
			return cursor.error();
		}
	}
	Ok(())
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

fn parse_number<'a>(cursor: &mut Cursor<'a>) -> Result<f64, Cursor<'a>> {
	let s = cursor.parse(repeat('0'..='9'))?;
	Ok(s.parse().unwrap())
}

fn parse_statement<'a>(cursor: &mut Cursor<'a>) -> Result<(), Cursor<'a>> {
	if let Ok(_) = cursor.parse(keyword("let")) {
		skip_comments(cursor)?;
		parse_identifier(cursor)?;
		skip_comments(cursor)?;
		cursor.parse(';')?;
	} else {
		parse_expression(cursor, 0)?;
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
