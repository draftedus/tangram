use crate::types;
use derive_more::Constructor;
use itertools::izip;
use ndarray::prelude::*;

#[derive(Constructor)]
pub struct Model<'a> {
	model: &'a types::Model,
}

impl<'a> std::fmt::Display for Model<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let overview = Overview::new(self.model);
		write!(f, "{}", overview)?;
		writeln!(f)?;
		let target_column = TargetColumn::new(
			&self.model.target_column,
			&self.model.stats.overall_target_column_stats,
		);
		write!(f, "{}", target_column)?;
		writeln!(f)?;
		let columns = Columns::new(&self.model.columns);
		write!(f, "{}", columns)?;
		writeln!(f)?;
		let metrics = Metrics::new(self.model);
		write!(f, "{}", metrics)?;
		writeln!(f, "{:?}", self.model.training_summary)?;
		Ok(())
	}
}

#[derive(Constructor)]
struct Overview<'a> {
	model: &'a types::Model,
}

impl<'a> std::fmt::Display for Overview<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "## Overview")?;
		writeln!(f)?;
		let model_name = match self.model.model_descriptor {
			types::ModelDescriptor::LinearRegressor(_) => "linear regressor",
			types::ModelDescriptor::LinearBinaryClassifier(_) => "linear binary classifier",
			types::ModelDescriptor::LinearMulticlassClassifier(_) => "linear multiclass classifier",
			types::ModelDescriptor::TreeRegressor(_) => "gradient boosted trees regressor",
			types::ModelDescriptor::TreeBinaryClassifier(_) => {
				"gradient boosted trees binary classifier"
			}
			types::ModelDescriptor::TreeMulticlassClassifier(_) => {
				"gradient boosted trees multiclass classifier"
			}
		};
		let target_column_name = self.model.target_column.column_name.as_str();
		writeln!(
			f,
			r#"You trained a {} to predict the column "{}"."#,
			model_name, target_column_name
		)?;
		writeln!(f)?;
		let values = match &self.model.task_descriptor {
			types::TaskDescriptor::Regression(_) => arr2(&[
				["Model Type", model_name],
				["Target Column", target_column_name],
			]),
			types::TaskDescriptor::Classification(_) => arr2(&[
				["Model Type", model_name],
				["Target Column", target_column_name],
			]),
		};
		let table = Table::new().values(&values);
		write!(f, "{}", table)?;
		Ok(())
	}
}

#[derive(Constructor)]
struct TargetColumn<'a> {
	target_column: &'a types::Column,
	target_column_stats: &'a types::ColumnStats,
}

impl<'a> std::fmt::Display for TargetColumn<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "## 🎯 Target Column")?;
		writeln!(f)?;
		let column_name = self.target_column.column_name.as_str();
		let column_type = match self.target_column.column_type {
			types::ColumnType::Unknown => "Unknown",
			types::ColumnType::Text => "Text",
			types::ColumnType::Number => "Number",
			types::ColumnType::Enum => "Enum",
		};
		let values = arr2(&[[column_name, column_type]]);
		let table = Table::new().header(&["Name", "Type"]).values(&values);
		write!(f, "{}", table)?;
		writeln!(f)?;

		if let types::ColumnType::Enum = self.target_column.column_type {
			let unique_values: Vec<&str> = match self.target_column_stats {
				types::ColumnStats::Enum(s) => {
					s.histogram.iter().map(|(k, _)| k.as_str()).collect()
				}
				_ => unreachable!(),
			};
			let values = Array2::from_shape_vec((unique_values.len(), 1), unique_values).unwrap();
			let table = Table::new().header(&["Unique Values"]).values(&values);
			write!(f, "{}", table)?;
		}
		Ok(())
	}
}

#[derive(Constructor)]
struct Columns<'a> {
	columns: &'a [types::Column],
}

impl<'a> std::fmt::Display for Columns<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "## Columns")?;
		writeln!(f)?;
		writeln!(
			f,
			"Your dataset has {} columns excluding the target column",
			self.columns.len()
		)?;
		writeln!(f)?;
		if self.columns.len() < 100 {
			let values: Array2<&str> =
				Array::from_shape_fn((self.columns.len(), 2), |(row, col)| match col {
					0 => self.columns[row].column_name.as_str(),
					1 => match self.columns[row].column_type {
						types::ColumnType::Unknown => "Unknown",
						types::ColumnType::Text => "Text",
						types::ColumnType::Number => "Number",
						types::ColumnType::Enum => "Enum",
					},
					_ => unreachable!(),
				});
			let header: &[&str] = &["Name", "Type"];
			let table = Table::new().header(header).values(&values);
			write!(f, "{}", table)?;
		} else {
			writeln!(
				f,
				"There are more than 100 columns so the columns are not printed",
			)?;
		}
		Ok(())
	}
}

#[derive(Constructor)]
struct ColumnStats<'a> {
	stats: &'a types::Stats,
}

impl<'a> std::fmt::Display for ColumnStats<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (i, column_stats) in self.stats.overall_column_stats.iter().enumerate() {
			if i != 0 {
				writeln!(f)?;
			}
			writeln!(f, "### {}", column_stats.column_name())?;
			writeln!(f)?;
			let column_type = match column_stats.column_type() {
				types::ColumnType::Unknown => "Unknown",
				types::ColumnType::Text => "Text",
				types::ColumnType::Number => "Number",
				types::ColumnType::Enum => "Enum",
			};
			let values = arr2(&[["Type", column_type]]);
			let table = Table::new().values(&values);
			write!(f, "{}", table)?;
			match column_stats {
				types::ColumnStats::Unknown(_) => {}
				types::ColumnStats::Text(_) => {}
				types::ColumnStats::Number(number_column_stats) => {
					let number_column_stats = NumberColumnStats::new(number_column_stats);
					writeln!(f)?;
					write!(f, "{}", number_column_stats)?;
				}
				types::ColumnStats::Enum(_) => {}
			}
		}
		Ok(())
	}
}

#[derive(Constructor)]
struct NumberColumnStats<'a> {
	column_stats: &'a types::NumberColumnStats,
}

impl<'a> std::fmt::Display for NumberColumnStats<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let min = self.column_stats.min.to_string();
		let max = self.column_stats.max.to_string();
		let mean = self.column_stats.mean.to_string();
		let std = self.column_stats.variance.sqrt().to_string();
		let values = arr2(&[
			["Min", &min],
			["Max", &max],
			["Mean", &mean],
			["Std Dev", &std],
		]);
		let table = Table::new().values(&values);
		write!(f, "{}", table)?;
		Ok(())
	}
}

#[derive(Constructor)]
struct Metrics<'a> {
	model: &'a types::Model,
}

impl<'a> std::fmt::Display for Metrics<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "## Metrics")?;
		writeln!(f)?;
		match &self.model.task_descriptor {
			types::TaskDescriptor::Regression(_) => {
				writeln!(f, "The baseline RMSE is the RMSE that a model would get if it always predicted the mean value in the training dataset of the target column.")?;
			}
			types::TaskDescriptor::Classification(_) => {
				writeln!(f, "The baseline accuracy is the accuracy that a model would get if it always predicted the majority class.")?;
			}
		}
		writeln!(f)?;
		let baseline = match &self.model.task_metrics {
			types::TaskMetrics::Regression(m) => m.baseline_rmse.to_string(),
			types::TaskMetrics::Classification(m) => m.baseline_accuracy.to_string(),
		};
		match (&self.model.task_metrics, &self.model.model_metrics) {
			(types::TaskMetrics::Regression(metrics), _) => {
				let mse = metrics.mse.to_string();
				let rmse = metrics.rmse.to_string();
				let mae = metrics.mae.to_string();
				let r2 = metrics.r2.to_string();
				let values = arr2(&[
					["Mean Squared Error (MSE)", &mse],
					["Root Mean Squared Error (RMSE)", &rmse],
					["Mean Absolute Error (MAE)", &mae],
					["R2", &r2],
				]);
				let table = Table::new().values(&values);
				write!(f, "{}", table)?;
			}
			(
				types::TaskMetrics::Classification(metrics),
				types::ModelMetrics::LinearBinaryClassifier(model_metrics),
			) => {
				let accuracy = metrics.accuracy.to_string();
				let auc = model_metrics.auc_roc.to_string();
				let values = arr2(&[
					["AUC", &auc],
					["Accuracy", &accuracy],
					["Baseline Accuracy", &baseline],
				]);
				let table = Table::new().values(&values);
				write!(f, "{}", table)?;
			}
			(
				types::TaskMetrics::Classification(metrics),
				types::ModelMetrics::TreeBinaryClassifier(model_metrics),
			) => {
				let accuracy = metrics.accuracy.to_string();
				let auc = model_metrics.auc_roc.to_string();
				let values = arr2(&[
					["AUC", &auc],
					["Accuracy", &accuracy],
					["Baseline Accuracy", &baseline],
				]);
				let table = Table::new().values(&values);
				write!(f, "{}", table)?;
			}
			(types::TaskMetrics::Classification(metrics), _) => {
				let accuracy = metrics.accuracy.to_string();
				let values = arr2(&[["Accuracy", &accuracy], ["Baseline Accuracy", &baseline]]);
				let table = Table::new().values(&values);
				write!(f, "{}", table)?;
			}
		};
		Ok(())
	}
}

#[derive(Constructor)]
struct ClassMetrics<'a> {
	class_metrics: &'a types::ClassMetrics,
}

impl<'a> std::fmt::Display for ClassMetrics<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, r#"### Class: "{}""#, self.class_metrics.class_name)?;
		writeln!(f)?;
		let actual_true = format!(r#"Actual "{}""#, self.class_metrics.class_name);
		let actual_false = format!(r#"Actual Not "{}""#, self.class_metrics.class_name);
		let header = &["", &actual_true, &actual_false];
		let predicted_true = format!(r#"Predicted "{}""#, self.class_metrics.class_name);
		let true_positives = self.class_metrics.true_positives.to_string();
		let false_positives = self.class_metrics.false_positives.to_string();
		let predicted_false = format!(r#"Predicted Not "{}""#, self.class_metrics.class_name);
		let true_negatives = self.class_metrics.true_negatives.to_string();
		let false_negatives = self.class_metrics.false_negatives.to_string();
		let values = arr2(&[
			[
				predicted_true.as_str(),
				true_positives.as_str(),
				false_positives.as_str(),
			],
			[
				predicted_false.as_str(),
				false_negatives.as_str(),
				true_negatives.as_str(),
			],
		]);
		let table = Table::new().header(header).values(&values);
		write!(f, "{}", table)?;
		writeln!(f)?;
		let accuracy = self.class_metrics.accuracy.to_string();
		let precision = self.class_metrics.precision.to_string();
		let recall = self.class_metrics.recall.to_string();
		let values = arr2(&[
			["Accuracy", &accuracy],
			["Precision", &precision],
			["Recall", &recall],
		]);
		let table = Table::new().values(&values);
		write!(f, "{}", table)?;
		Ok(())
	}
}

pub struct Table<'a> {
	padding: usize,
	header: Option<&'a [&'a str]>,
	values: Option<&'a Array2<&'a str>>,
}

impl<'a> Default for Table<'a> {
	fn default() -> Self {
		Self {
			header: None,
			padding: 1,
			values: None,
		}
	}
}

impl<'a> Table<'a> {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn header(mut self, header: &'a [&'a str]) -> Self {
		self.header = Some(header);
		self
	}
	pub fn values(mut self, values: &'a Array2<&'a str>) -> Self {
		self.values = Some(values);
		self
	}
}

impl<'a> std::fmt::Display for Table<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let n_columns = None
			.or_else(|| self.header.map(|header| header.len()))
			.or_else(|| self.values.map(|values| values.ncols()))
			.unwrap_or(0);
		let mut column_widths: Vec<_> = vec![0; n_columns];
		// update column widths with header
		if let Some(header) = self.header {
			column_widths
				.iter_mut()
				.zip(header)
				.for_each(|(column_width, header)| *column_width = header.len());
		}
		// update column widths with values
		if let Some(values) = self.values {
			izip!(&mut column_widths, values.gencolumns()).for_each(|(column_width, col)| {
				col.iter().for_each(|value| {
					*column_width = usize::max(*column_width, value.len());
				})
			});
		}
		// write header
		let line = Line {
			column_widths: &column_widths,
			padding: self.padding,
		};
		if let Some(header) = self.header {
			let row = Row {
				column_widths: &column_widths,
				padding: self.padding,
				values: header,
			};
			writeln!(f, "{}", row)?;
			writeln!(f, "{}", line)?;
		}
		// write values
		if let Some(values) = self.values {
			for row in values.genrows() {
				let row = Row {
					column_widths: &column_widths,
					padding: self.padding,
					values: row.as_slice().unwrap(),
				};
				writeln!(f, "{}", row)?;
			}
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
