use crate::printer::{bold, red};

pub type Location = usize;

pub struct Error {
	pub i: Location,
	pub msg: String,
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
