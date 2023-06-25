pub type Location = usize;

pub struct Error {
	pub i: Location,
	pub msg: String,
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

impl Error {
	pub fn print<W: std::io::Write>(&self, s: &str, mut write: W) -> std::io::Result<()> {
		writeln!(write, "{}: {}", bold(red("error")), self.msg)?;
		let mut start = 0;
		let mut end = s.len();
		let mut num = 0;
		for (i, c) in s.char_indices() {
			if c == '\n' {
				if i < self.i {
					start = i + c.len_utf8();
					num += 1;
				} else {
					end = i;
					break;
				}
			}
		}
		let line = s.get(start..end).unwrap();
		writeln!(write, "{} | {}", num, line)?;
		write!(write, "{} | ", num)?;
		for (_, c) in line.char_indices().take_while(|(i, _)| start + *i < self.i) {
			let c = if c.is_whitespace() { c } else { ' ' };
			write!(write, "{}", c)?;
		}
		writeln!(write, "^")?;
		Ok(())
	}
}
