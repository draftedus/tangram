pub fn format_percent(value: f32) -> String {
	format_percent_with_precision(value, 2)
}

pub fn format_percent_with_precision(value: f32, precision: usize) -> String {
	if f32::abs(value - 1.0) <= f32::EPSILON {
		"100%".to_owned()
	} else {
		format!("{:.1$}%", value * 100.0, precision)
	}
}

#[test]
fn test_format_percent() {
	assert_eq!(format_percent(0.0), "0.00%");
	assert_eq!(format_percent(0.424292), "42.43%");
	assert_eq!(format_percent_with_precision(0.424292, 3), "42.429%");
	assert_eq!(format_percent(1.00), "100%");
}
