// use crate::config::{ChartConfig, CHART_CONFIG};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Point {
	x: f32,
	y: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ChartBox {
	h: f32,
	w: f32,
	x: f32,
	y: f32,
}

// The interval is k * 10 ** p. k will always be 1, 2, or 5.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct GridLineInterval {
	k: usize,
	p: usize,
}

// pub struct GridLineInfo {
// 	interval: f32,
// 	interval_pixels: f32,
// 	k: f32,
// 	num_grid_lines: usize,
// 	p: usize,
// 	start: f32,
// 	start_pixels: f32,
// }

// pub struct ComputeBoxesOptions {
// 	ctx: web_sys::CanvasRenderingContext2d,
// 	height: f32,
// 	include_x_axis_labels: bool,
// 	include_x_axis_title: bool,
// 	include_y_axis_labels: bool,
// 	include_y_axis_title: bool,
// 	width: f32,
// 	x_axis_grid_line_interval: Option<GridLineInterval>,
// 	y_axis_grid_line_interval: Option<GridLineInterval>,
// 	y_max: f32,
// 	y_min: f32,
// }

// pub struct ComputeBoxesOutput {
// 	chart_box: ChartBox,
// 	x_axis_labels_box: ChartBox,
// 	x_axis_title_box: ChartBox,
// 	y_axis_grid_line_info: GridLineInfo,
// 	y_axis_labels_box: ChartBox,
// 	y_axis_title_box: ChartBox,
// }

// pub fn compute_boxes(options: ComputeBoxesOptions) -> ComputeBoxesOutput {
// 	let ComputeBoxesOptions {
// 		ctx,
// 		height,
// 		include_x_axis_labels,
// 		include_x_axis_title,
// 		include_y_axis_labels,
// 		include_y_axis_title,
// 		width,
// 		y_max,
// 		y_min,
// 		..
// 	} = options;
// 	let ChartConfig {
// 		bottom_padding,
// 		font_size,
// 		label_padding,
// 		left_padding,
// 		right_padding,
// 		top_padding,
// 		..
// 	} = CHART_CONFIG;

// 	let x_axis_labels_padding = if include_x_axis_labels {
// 		label_padding + font_size
// 	} else {
// 		0.0
// 	};
// 	let x_axis_title_padding = if include_x_axis_title {
// 		label_padding + font_size
// 	} else {
// 		0.0
// 	};
// 	let chart_height = options.height - top_padding
// 		+ x_axis_labels_padding
// 		+ x_axis_title_padding
// 		+ bottom_padding;

// 	let y_axis_grid_line_info = compute_y_axis_grid_line_info(ComputeYAxisGridLineInfoOptions {
// 		chart_height,
// 		font_size,
// 		y_axis_grid_line_interval: options.y_axis_grid_line_interval,
// 		y_max,
// 		y_min,
// 	});
// 	let y_axis_labels_width = compute_axis_labels_max_width(ctx, y_axis_grid_line_info);

// 	let y_axis_title_padding = if include_y_axis_title {
// 		font_size + label_padding
// 	} else {
// 		0.0
// 	};
// 	let y_axis_labels_padding = if include_y_axis_labels {
// 		y_axis_labels_width + label_padding
// 	} else {
// 		0.0
// 	};
// 	let chart_width =
// 		width - left_padding + y_axis_title_padding + y_axis_labels_padding + right_padding;

// 	let x_axis_labels_box = ChartBox {
// 		h: if include_x_axis_labels {
// 			font_size.to_f32().unwrap()
// 		} else {
// 			0.0
// 		},
// 		w: chart_width.to_f32().unwrap(),
// 		x: left_padding + x_axis_labels_padding + y_axis_labels_padding,
// 		y: top_padding
// 			+ chart_height
// 			+ (if include_x_axis_labels {
// 				label_padding
// 			} else {
// 				0.0
// 			}),
// 	};

// 	// let x_axis_title_box = {
// 	// 	h: includeXAxisTitle ? fontSize : 0,
// 	// 	w: chartWidth,
// 	// 	x:
// 	// 		leftPadding +
// 	// 		(includeYAxisTitle ? fontSize + labelPadding : 0) +
// 	// 		(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0),
// 	// 	y:
// 	// 		topPadding +
// 	// 		chartHeight +
// 	// 		(includeXAxisLabels ? labelPadding + fontSize : 0) +
// 	// 		(includeXAxisTitle ? labelPadding : 0),
// 	// }

// 	// let yAxisTitleBox = {
// 	// 	h: chartHeight,
// 	// 	w: fontSize,
// 	// 	x: leftPadding,
// 	// 	y: topPadding,
// 	// }

// 	// let yAxisLabelsBox = {
// 	// 	h: chartHeight,
// 	// 	w: yAxisLabelsWidth,
// 	// 	x: leftPadding + (includeYAxisTitle ? fontSize + labelPadding : 0),
// 	// 	y: topPadding,
// 	// }

// 	// let chartBox = {
// 	// 	h: chartHeight,
// 	// 	w: chartWidth,
// 	// 	x:
// 	// 		leftPadding +
// 	// 		(includeYAxisTitle ? fontSize + labelPadding : 0) +
// 	// 		(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0),
// 	// 	y: topPadding,
// 	// }

// 	todo!()
// 	// ComputeBoxesOutput {
// 	// chart_box,
// 	// x_axis_labels_box
// 	// x_axis_title_box
// 	// y_axis_grid_line_info
// 	// y_axis_labels_box
// 	// y_axis_title_box
// 	// }
// }

// pub struct ComputeYAxisGridLineInfoOptions {
// 	chart_height: f32,
// 	font_size: f32,
// 	y_axis_grid_line_interval: Option<GridLineInterval>,
// 	y_max: f32,
// 	y_min: f32,
// }

// pub fn compute_y_axis_grid_line_info(options: ComputeYAxisGridLineInfoOptions) -> GridLineInfo {
// 	// let { chartHeight, fontSize, yMax, yMin } = options
// 	// let yAxisGridLineInterval: GridLineInterval = !options.yAxisGridLineInterval
// 	// 	? computeGridLineInterval(yMin, yMax, chartHeight, fontSize)
// 	// 	: options.yAxisGridLineInterval
// 	// return computeGridLineInfo(yMin, yMax, chartHeight, yAxisGridLineInterval)
// 	todo!()
// }

// pub fn compute_axis_labels_max_width(
// 	ctx: web_sys::CanvasRenderingContext2d,
// 	grid_line_info: GridLineInfo,
// ) -> f32 {
// 	todo!()
// 	// return Math.max(
// 	// 	...times(gridLineInfo.numGridLines, gridLineIndex => {
// 	// 		let gridLineValue =
// 	// 			gridLineInfo.start + gridLineIndex * gridLineInfo.interval
// 	// 		let label = formatNumber(gridLineValue)
// 	// 		return ctx.measureText(label).width
// 	// 	}),
// 	// )
// }
