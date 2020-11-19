pub struct ChartConfig {
	pub axis_width: usize,
	pub bar_gap: usize,
	pub bar_group_gap: usize,
	pub bar_stroke_width: usize,
	pub bottom_padding: usize,
	pub feature_contributions_arrow_depth: usize,
	pub feature_contributions_bar_gap: usize,
	pub feature_contributions_series_gap: usize,
	pub feature_contributions_series_height: usize,
	pub font: &'static str,
	pub font_size: usize,
	pub label_padding: usize,
	pub left_padding: usize,
	pub max_corner_radius: usize,
	pub point_halo_radius: usize,
	pub point_radius: usize,
	pub right_padding: usize,
	pub spline_tension: usize,
	pub tooltip_border_radius: usize,
	pub tooltip_padding: usize,
	pub tooltip_shadow_blur: usize,
	pub tooltip_target_radius: usize,
	pub top_padding: usize,
}

pub const CHART_CONFIG: ChartConfig = ChartConfig {
	axis_width: 2,
	bar_gap: 2,
	bar_group_gap: 4,
	bar_stroke_width: 2,
	bottom_padding: 8,
	feature_contributions_arrow_depth: 4,
	feature_contributions_bar_gap: 10,
	feature_contributions_series_gap: 20,
	feature_contributions_series_height: 100,
	font: "14px JetBrains Mono",
	font_size: 14,
	label_padding: 8,
	left_padding: 8,
	max_corner_radius: 8,
	point_halo_radius: 8,
	point_radius: 4,
	right_padding: 8,
	spline_tension: 0,
	tooltip_border_radius: 4,
	tooltip_padding: 4,
	tooltip_shadow_blur: 2,
	tooltip_target_radius: 5,
	top_padding: 8,
};

pub struct ChartColors {
	axis_color: &'static str,
	border_color: &'static str,
	crosshairs_color: &'static str,
	grid_line_color: &'static str,
	label_color: &'static str,
	text_color: &'static str,
	title_color: &'static str,
	tooltip_background_color: &'static str,
	tooltip_shadow_color: &'static str,
}

pub const LIGHT_CHART_COLORS: ChartColors = ChartColors {
	axis_color: "#BBBBBB",
	border_color: "#EEEEEE",
	crosshairs_color: "#666666",
	grid_line_color: "#EEEEEE",
	label_color: "#666666",
	text_color: "#222222",
	title_color: "#222222",
	tooltip_background_color: "#FFFFFF",
	tooltip_shadow_color: "rgba(0, 0, 0, .1)",
};

pub const DARK_CHART_COLORS: ChartColors = ChartColors {
	axis_color: "#AAAAAA",
	border_color: "#333333",
	crosshairs_color: "#AAAAAA",
	grid_line_color: "#222222",
	label_color: "#888888",
	text_color: "#EEEEEE",
	title_color: "#EEEEEE",
	tooltip_background_color: "#333333",
	tooltip_shadow_color: "rgba(0, 0, 0, .1)",
};

pub const CHART_COLORS: ChartColors = LIGHT_CHART_COLORS;
