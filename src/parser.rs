pub trait Parser {
	fn parse(&mut self, s: &str) -> Option<usize>;
}

impl Parser for char {
	fn parse(&mut self, s: &str) -> Option<usize> {
		match s.chars().next() {
			Some(c) if *self == c => Some(c.len_utf8()),
			_ => None,
		}
	}
}

impl Parser for std::ops::RangeInclusive<char> {
	fn parse(&mut self, s: &str) -> Option<usize> {
		match s.chars().next() {
			Some(c) if self.contains(&c) => Some(c.len_utf8()),
			_ => None,
		}
	}
}

impl <F: FnMut(char) -> bool> Parser for F {
	fn parse(&mut self, s: &str) -> Option<usize> {
		match s.chars().next() {
			Some(c) if self(c) => Some(c.len_utf8()),
			_ => None,
		}
	}
}

impl Parser for &str {
	fn parse(&mut self, s: &str) -> Option<usize> {
		if s.starts_with(*self) {
			Some(self.len())
		} else {
			None
		}
	}
}

struct Optional<P: Parser>(P);

impl <P: Parser> Parser for Optional<P> {
	fn parse(&mut self, s: &str) -> Option<usize> {
		match self.0.parse(s) {
			Some(i) => Some(i),
			None => Some(0),
		}
	}
}

pub fn optional<P: Parser>(p: P) -> impl Parser {
	Optional(p)
}

struct Repetition<P: Parser>(P);

impl <P: Parser> Parser for Repetition<P> {
	fn parse(&mut self, mut s: &str) -> Option<usize> {
		let mut sum = 0;
		while let Some(len) = self.0.parse(s) {
			s = s.split_at(len).1;
			sum += len;
		}
		Some(sum)
	}
}

pub fn repeat<P: Parser>(p: P) -> impl Parser {
	Repetition(p)
}

struct Not<P: Parser>(P);

impl <P: Parser> Parser for Not<P> {
	fn parse(&mut self, s: &str) -> Option<usize> {
		match self.0.parse(s) {
			Some(_) => None,
			None => Some(0),
		}
	}
}

pub fn not<P: Parser>(p: P) -> impl Parser {
	Not(p)
}

struct Peek<P: Parser>(P);

impl <P: Parser> Parser for Peek<P> {
	fn parse(&mut self, s: &str) -> Option<usize> {
		match self.0.parse(s) {
			Some(_) => Some(0),
			None => None,
		}
	}
}

pub fn peek<P: Parser>(p: P) -> impl Parser {
	Peek(p)
}

struct FunctionParser<F>(F);

impl <F: Fn(&mut Cursor) -> Option<()>> Parser for FunctionParser<F> {
	fn parse(&mut self, s: &str) -> Option<usize> {
		let mut cursor = Cursor::new(s);
		match self.0(&mut cursor) {
			Some(()) => Some(cursor.i),
			None => None,
		}
	}
}

pub fn from_function<F: Fn(&mut Cursor) -> Option<()>>(f: F) -> impl Parser {
	FunctionParser(f)
}

struct Sequence<P0, P1>(P0, P1);

impl <P0: Parser, P1: Parser> Parser for Sequence<P0, P1> {
	fn parse(&mut self, s: &str) -> Option<usize> {
		let len0 = self.0.parse(s)?;
		let (_, s) = s.split_at(len0);
		let len1 = self.1.parse(s)?;
		Some(len0 + len1)
	}
}

pub fn sequence0<P0: Parser, P1: Parser>(p0: P0, p1: P1) -> impl Parser {
	Sequence(p0, p1)
}
macro_rules! sequence {
	($p0:expr, $($p:expr),+) => {
		$crate::parser::sequence0($p0, $crate::parser::sequence!($($p),+))
	};
	($p:expr) => {
		$p
	};
}
pub(crate) use sequence;

struct Choice<P0, P1>(P0, P1);

impl <P0: Parser, P1: Parser> Parser for Choice<P0, P1> {
	fn parse(&mut self, s: &str) -> Option<usize> {
		if let Some(len) = self.0.parse(s) {
			return Some(len);
		}
		if let Some(len) = self.1.parse(s) {
			return Some(len);
		}
		None
	}
}

pub fn choice0<P0: Parser, P1: Parser>(p0: P0, p1: P1) -> impl Parser {
	Choice(p0, p1)
}
macro_rules! choice {
	($p0:expr, $($p:expr),+) => {
		$crate::parser::choice0($p0, $crate::parser::choice!($($p),+))
	};
	($p:expr) => {
		$p
	};
}
pub(crate) use choice;

pub fn parse<'a, P: Parser>(mut p: P, s: &'a str) -> Option<(&'a str, &'a str)> {
	p.parse(s).map(|i| s.split_at(i))
}

pub struct Cursor<'a> {
	pub s: &'a str,
	pub i: usize,
}

pub struct Error<'a> {
	pub s: &'a str,
	pub i: usize,
	pub msg: String,
}

impl <'a> Cursor<'a> {
	pub fn new(s: &'a str) -> Self {
		Cursor {
			s,
			i: 0,
		}
	}
	pub fn error<T>(&self) -> Result<T, Error<'a>> {
		Err(Error {
			s: self.s,
			i: self.i,
			msg: String::new(),
		})
	}
	pub fn parse<P: Parser>(&mut self, mut p: P) -> Result<&'a str, Error<'a>> {
		let (_, s) = self.s.split_at(self.i);
		match p.parse(s) {
			Some(i) => {
				self.i += i;
				let (result, _) = s.split_at(i);
				Ok(result)
			},
			None => self.error(),
		}
	}
	pub fn expect(&mut self, s: &str) -> Result<(), Error<'a>> {
		match self.parse(s) {
			Ok(_) => Ok(()),
			Err(err) => Err(Error {
				msg: format!("expected {}", s),
				..err
			}),
		}
	}
}

pub trait ParseResult {
	fn set_error_message<S: Into<String>>(self, msg: S) -> Self;
}

impl <'a, T> ParseResult for Result<T, Error<'a>> {
	fn set_error_message<S: Into<String>>(self, msg: S) -> Self {
		self.map_err(|err| Error {
			msg: msg.into(),
			..err
		})
	}
}
