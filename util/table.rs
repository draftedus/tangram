use crate::zip;
use ndarray::prelude::*;

pub struct Table<'a> {
	padding: usize,
	header: &'a [&'a str],
	values: &'a Array2<&'a str>,
}

impl<'a> std::fmt::Display for Table<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let n_columns = self.header.len();
		let mut column_widths: Vec<_> = vec![0; n_columns];
		// update column widths with header
		column_widths
			.iter_mut()
			.zip(self.header)
			.for_each(|(column_width, header)| *column_width = header.len());
		// update column widths with values
		zip!(&mut column_widths, self.values.axis_iter(Axis(1))).for_each(|(column_width, col)| {
			col.iter().for_each(|value| {
				*column_width = usize::max(*column_width, value.len());
			})
		});
		// write header
		let line = Line {
			column_widths: &column_widths,
			padding: self.padding,
		};
		let row = Row {
			column_widths: &column_widths,
			padding: self.padding,
			values: self.header,
		};
		writeln!(f, "{}", row)?;
		writeln!(f, "{}", line)?;
		// write values
		for row in self.values.genrows() {
			let row = Row {
				column_widths: &column_widths,
				padding: self.padding,
				values: row.as_slice().unwrap(),
			};
			writeln!(f, "{}", row)?;
		}
		Ok(())
	}
}

struct Line<'a> {
	column_widths: &'a [usize],
	padding: usize,
}

impl<'a> std::fmt::Display for Line<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "|")?;
		for column_width in self.column_widths.iter() {
			for _ in 0..column_width + 2 * self.padding {
				write!(f, "-")?;
			}
			write!(f, "|")?;
		}
		Ok(())
	}
}

struct Row<'a> {
	column_widths: &'a [usize],
	padding: usize,
	values: &'a [&'a str],
}

impl<'a> std::fmt::Display for Row<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "|")?;
		for (column_width, value) in self.column_widths.iter().zip(self.values) {
			for _ in 0..self.padding {
				write!(f, " ")?;
			}
			write!(f, "{}", value)?;
			for _ in 0..column_width + self.padding - value.len() {
				write!(f, " ")?;
			}
			write!(f, "|")?;
		}
		Ok(())
	}
}
