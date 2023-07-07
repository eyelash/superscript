use std::fmt::{Display, Formatter, Result};

struct Bold<T>(T);

impl <T: Display> Display for Bold<T> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "\x1B[1m{}\x1B[22m", self.0)?;
		Ok(())
	}
}

pub fn bold<T: Display>(t: T) -> impl Display {
	Bold(t)
}

struct Red<T>(T);

impl <T: Display> Display for Red<T> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "\x1B[31m{}\x1B[39m", self.0)?;
		Ok(())
	}
}

pub fn red<T: Display>(t: T) -> impl Display {
	Red(t)
}

struct Green<T>(T);

impl <T: Display> Display for Green<T> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "\x1B[32m{}\x1B[39m", self.0)?;
		Ok(())
	}
}

pub fn green<T: Display>(t: T) -> impl Display {
	Green(t)
}

struct CommaSeparated<T>(T);

impl <D: Display, T: IntoIterator<Item=D> + Clone> Display for CommaSeparated<T> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let mut i = self.0.clone().into_iter();
		if let Some(d) = i.next() {
			write!(f, "{}", d)?;
			while let Some(d) = i.next() {
				write!(f, ", {}", d)?;
			}
		}
		Ok(())
	}
}

pub fn comma_separated<D: Display, T: IntoIterator<Item=D> + Clone>(t: T) -> impl Display {
	CommaSeparated(t)
}

pub struct Printer<W> {
	write: W,
	indentation: usize,
}

impl<W: std::io::Write> Printer<W> {
	pub fn new(write: W) -> Self {
		Printer {
			write,
			indentation: 0,
		}
	}
	pub fn println<D: Display>(&mut self, d: D) -> std::io::Result<()> {
		for _ in 0..self.indentation {
			write!(self.write, "\t")?;
		}
		writeln!(self.write, "{}", d)?;
		Ok(())
	}
	pub fn increase_indentation(&mut self) {
		self.indentation += 1;
	}
	pub fn decrease_indentation(&mut self) {
		self.indentation -= 1;
	}
	pub fn indented<F: FnOnce(&mut Self)>(&mut self, mut f: F) {
		self.indentation += 1;
		f(self);
		self.indentation -= 1;
	}
}
