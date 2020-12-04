use crate::{
	chart::{
		ActiveHoverRegion, ChartImpl, DrawChartOptions, DrawChartOutput, DrawOverlayOptions,
		HoverRegion,
	},
	common::{
		compute_x_axis_grid_line_info, draw_x_axis_grid_lines, draw_x_axis_labels,
		draw_x_axis_title, draw_y_axis_title, ComputeXAxisGridLineInfoOptions,
		DrawXAxisGridLinesOptions, DrawXAxisLabelsOptions, DrawXAxisTitleOptions,
		DrawYAxisTitleOptions, Point, Rect,
	},
	config::{ChartColors, ChartConfig},
	tooltip::{draw_tooltip, DrawTooltipOptions, TooltipLabel},
};
use num_traits::ToPrimitive;
use web_sys::*;

pub struct FeatureContributionsChart;

impl ChartImpl for FeatureContributionsChart {
	type Options = FeatureContributionsChartOptions;
	type OverlayInfo = FeatureContributionsChartOverlayInfo;
	type HoverRegionInfo = FeatureContributionsChartHoverRegionInfo;

	fn draw_chart(
		options: DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo> {
		draw_feature_contributions_chart(options)
	}

	fn draw_overlay(options: DrawOverlayOptions<Self::OverlayInfo, Self::HoverRegionInfo>) {
		draw_feature_contributions_chart_overlay(options)
	}
}

/// These are the options for displaying a feature contributions chart.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartOptions {
	pub include_x_axis_title: Option<bool>,
	pub include_y_axis_labels: Option<bool>,
	pub include_y_axis_title: Option<bool>,
	pub negative_color: String,
	pub positive_color: String,
	pub series: Vec<FeatureContributionsChartSeries>,
}

/// This is the configuration used across all feature contributions charts.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartConfig {
	pub arrow_depth: f64,
	pub bar_gap: f64,
	pub series_gap: f64,
	pub series_width: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartSeries {
	pub baseline: f64,
	pub baseline_label: String,
	pub output: f64,
	pub output_label: String,
	pub title: String,
	pub values: Vec<FeatureContributionsChartValue>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartValue {
	pub feature: String,
	pub value: f64,
}

#[derive(Clone)]
pub struct FeatureContributionsChartHoverRegionInfo {
	rect: Rect,
	color: String,
	direction: FeatureContributionsBoxDirection,
	label: String,
	tooltip_origin_pixels: Point,
}

pub struct FeatureContributionsChartOverlayInfo {
	chart_box: Rect,
}

fn draw_feature_contributions_chart(
	options: DrawChartOptions<FeatureContributionsChartOptions>,
) -> DrawChartOutput<FeatureContributionsChartOverlayInfo, FeatureContributionsChartHoverRegionInfo>
{
	let DrawChartOptions {
		chart_colors,
		chart_config,
		ctx,
		options,
	} = options;
	let FeatureContributionsChartOptions {
		include_x_axis_title,
		include_y_axis_labels,
		include_y_axis_title,
		negative_color,
		positive_color,
		series: data,
	} = &options;
	let include_x_axis_title = include_x_axis_title.unwrap_or(false);
	let include_y_axis_labels = include_y_axis_labels.unwrap_or(false);
	let include_y_axis_title = include_y_axis_title.unwrap_or(false);

	let canvas = ctx.canvas().unwrap();
	let height = canvas.client_height().to_f64().unwrap();
	let width = canvas.client_width().to_f64().unwrap();
	let ChartConfig {
		bottom_padding,
		font_size,
		label_padding,
		left_padding,
		right_padding,
		top_padding,
		..
	} = chart_config;

	let annotations_padding = 80.0;
	let mut hover_regions: Vec<HoverRegion<FeatureContributionsChartHoverRegionInfo>> = Vec::new();

	// Compute the bounds.
	let min_baseline = data
		.iter()
		.map(|series| series.baseline)
		.min_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();
	let min_output = data
		.iter()
		.map(|series| series.output)
		.min_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();
	let x_min = min_baseline.min(min_output);
	let x_max = data
		.iter()
		.map(|series| {
			let sum_of_positive_values = series
				.values
				.iter()
				.filter_map(|value| {
					if value.value > 0.0 {
						Some(value.value)
					} else {
						None
					}
				})
				.sum::<f64>();
			series.baseline + sum_of_positive_values
		})
		.max_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();

	let y_axis_labels_width = data
		.iter()
		.flat_map(|series| {
			series
				.values
				.iter()
				.map(|value| ctx.measure_text(&value.feature).unwrap().width())
		})
		.max_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();
	let chart_width = width
		- (left_padding
			+ if include_y_axis_labels {
				y_axis_labels_width + label_padding
			} else {
				0.0
			} + right_padding
			+ if include_y_axis_title {
				font_size + label_padding
			} else {
				0.0
			} + annotations_padding);

	let chart_height = height
		- (top_padding
			+ font_size + label_padding
			+ font_size + label_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + bottom_padding);

	let chart_box = Rect {
		h: chart_height,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + font_size
			+ label_padding,
	};

	let top_x_axis_title_box = Rect {
		h: *font_size,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: *top_padding,
	};
	if include_x_axis_title {
		draw_x_axis_title(DrawXAxisTitleOptions {
			rect: top_x_axis_title_box,
			ctx: ctx.clone(),
			title: "Contributions",
		})
	}

	let top_x_axis_labels_box = Rect {
		h: *font_size,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			},
	};

	let bottom_x_axis_labels_box = Rect {
		h: *font_size,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: top_padding
			+ font_size + label_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + chart_height
			+ label_padding,
	};

	let y_axis_labels_box = Rect {
		h: chart_height,
		w: y_axis_labels_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			},
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + font_size
			+ label_padding,
	};

	let y_axis_titles_box = Rect {
		h: chart_height,
		w: *font_size,
		x: *left_padding,
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + font_size
			+ label_padding,
	};

	if include_y_axis_title {
		draw_y_axis_title(DrawYAxisTitleOptions {
			rect: y_axis_titles_box,
			ctx: ctx.clone(),
			title: "Class",
		});
	}

	// Compute the grid line info.
	let x_axis_grid_line_info = compute_x_axis_grid_line_info(ComputeXAxisGridLineInfoOptions {
		chart_width: chart_box.w,
		ctx: ctx.clone(),
		x_max,
		x_min,
		x_axis_grid_line_interval: None,
	});

	draw_x_axis_grid_lines(DrawXAxisGridLinesOptions {
		rect: chart_box,
		ctx: ctx.clone(),
		x_axis_grid_line_info: x_axis_grid_line_info.clone(),
		chart_colors,
		chart_config,
	});

	draw_x_axis_labels(DrawXAxisLabelsOptions {
		rect: top_x_axis_labels_box,
		ctx: ctx.clone(),
		grid_line_info: x_axis_grid_line_info.clone(),
		width,
		labels: &None,
	});

	draw_x_axis_labels(DrawXAxisLabelsOptions {
		rect: bottom_x_axis_labels_box,
		ctx: ctx.clone(),
		grid_line_info: x_axis_grid_line_info.clone(),
		width,
		labels: &None,
	});

	let categories = data.iter().map(|series| &series.title).collect::<Vec<_>>();
	if include_y_axis_labels {
		draw_feature_contributions_chart_y_axis_labels(
			DrawFeatureContributionsChartYAxisLabelsOptions {
				rect: y_axis_labels_box,
				categories: &categories,
				ctx: ctx.clone(),
				width,
				chart_config,
			},
		);
	}

	// Draw the series separators.
	for i in 0..categories.len() {
		let y = chart_box.y
			+ i.to_f64().unwrap() * chart_config.feature_contributions_series_height
			+ (i - 1).to_f64().unwrap() * chart_config.feature_contributions_series_gap
			+ chart_config.feature_contributions_series_gap / 2.0;
		ctx.save();
		ctx.set_stroke_style(&chart_colors.grid_line_color.into());
		ctx.move_to(chart_box.x, y);
		ctx.line_to(chart_box.x + chart_box.w, y);
		ctx.stroke();
		ctx.restore();
	}

	let value_width_multiplier = chart_box.w / (x_max - x_min);
	for (series_index, series) in data.iter().enumerate() {
		let sum_positives = series
			.values
			.iter()
			.filter(|value| value.value > 0.0)
			.map(|value| value.value)
			.sum::<f64>();
		let min = series.baseline.min(series.output);
		let max = series.baseline + sum_positives;
		let width = max - min;
		let box_height = (chart_config.feature_contributions_series_height
			- chart_config.feature_contributions_bar_gap)
			/ 2.0;
		let rect = Rect {
			h: chart_config.feature_contributions_series_height,
			w: width * value_width_multiplier,
			x: chart_box.x + (min - x_min) * value_width_multiplier,
			y: chart_box.y
				+ (chart_config.feature_contributions_series_gap
					+ chart_config.feature_contributions_series_height)
					* series_index.to_f64().unwrap(),
		};
		let output = draw_feature_contribution_series(DrawFeatureContributionSeriesOptions {
			rect,
			box_height,
			ctx: ctx.clone(),
			negative_color,
			positive_color,
			series,
			value_width_multiplier,
			chart_config,
			chart_colors,
		});
		hover_regions.extend(output.hover_regions);
	}

	DrawChartOutput {
		hover_regions,
		overlay_info: FeatureContributionsChartOverlayInfo { chart_box },
	}
}

struct DrawFeatureContributionSeriesOptions<'a> {
	chart_config: &'a ChartConfig,
	chart_colors: &'a ChartColors,
	rect: Rect,
	box_height: f64,
	ctx: CanvasRenderingContext2d,
	negative_color: &'a str,
	positive_color: &'a str,
	series: &'a FeatureContributionsChartSeries,
	value_width_multiplier: f64,
}

struct DrawFeatureContributionsSeriesOutput {
	hover_regions: Vec<HoverRegion<FeatureContributionsChartHoverRegionInfo>>,
}

fn draw_feature_contribution_series(
	options: DrawFeatureContributionSeriesOptions,
) -> DrawFeatureContributionsSeriesOutput {
	let mut hover_regions: Vec<HoverRegion<FeatureContributionsChartHoverRegionInfo>> = Vec::new();
	let DrawFeatureContributionSeriesOptions {
		rect,
		box_height,
		ctx,
		negative_color,
		positive_color,
		series,
		value_width_multiplier,
		chart_config,
		chart_colors,
	} = options;
	let min = series.baseline.min(series.output);

	// Draw the positive boxes which start at the baseline and go to the max, ending with the remaining features rect.
	let mut positive_values: Vec<FeatureContributionsChartValue> = series
		.values
		.iter()
		.filter(|value| value.value > 0.0)
		.cloned()
		.collect();
	positive_values.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
	let mut x = rect.x + (series.baseline - min) * value_width_multiplier;
	// Draw the baseline value and label.
	ctx.set_text_baseline("bottom");
	ctx.set_text_align("right");
	ctx.fill_text(
		"baseline",
		x - chart_config.label_padding,
		rect.y + box_height / 2.0,
	)
	.unwrap();
	ctx.set_text_baseline("top");
	ctx.set_text_align("right");
	ctx.fill_text(
		&series.baseline_label,
		x - chart_config.label_padding,
		rect.y + box_height / 2.0,
	)
	.unwrap();
	let mut positive_values_index = 0;
	while positive_values_index < positive_values.len() {
		let feature_contribution_value = positive_values.get(positive_values_index).unwrap();
		let width = feature_contribution_value.value * value_width_multiplier;
		if width < chart_config.feature_contributions_arrow_depth * 2.0 {
			break;
		}
		let value_box = Rect {
			h: box_height,
			w: width,
			x,
			y: rect.y,
		};
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: value_box,
			color: positive_color.to_owned(),
			ctx: ctx.clone(),
			direction: FeatureContributionsBoxDirection::Negative,
			label: feature_contribution_value.feature.clone(),
			chart_config,
		});
		hover_regions.push(feature_contributions_chart_hover_region(
			FeatureContributionsChartHoverRegionOptions {
				rect: value_box,
				color: positive_color.to_owned(),
				direction: FeatureContributionsBoxDirection::Negative,
				label: feature_contribution_value.feature.clone(),
				tooltip_origin_pixels: Point {
					x: value_box.x + value_box.w / 2.0,
					y: value_box.y,
				},
			},
		));
		x += width;
		positive_values_index += 1;
	}
	let mut n_remaining_features = 0;
	let mut remaining_features_box_width = 0.0;
	while positive_values_index < positive_values.len() {
		let feature_contribution_value = positive_values.get(positive_values_index).unwrap();
		let width = feature_contribution_value.value * value_width_multiplier;
		remaining_features_box_width += width;
		n_remaining_features += 1;
	}
	if remaining_features_box_width > 0.0 {
		let remaining_features_box = Rect {
			h: box_height,
			w: remaining_features_box_width,
			x,
			y: rect.y,
		};
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: remaining_features_box,
			color: format!("{}33", positive_color),
			ctx: ctx.clone(),
			direction: FeatureContributionsBoxDirection::Negative,
			label: format!("{} other features", n_remaining_features),
			chart_config,
		});
		hover_regions.push(feature_contributions_chart_hover_region(
			FeatureContributionsChartHoverRegionOptions {
				rect: remaining_features_box,
				color: format!("{}33", positive_color),
				direction: FeatureContributionsBoxDirection::Negative,
				label: format!("{} other features", n_remaining_features),
				tooltip_origin_pixels: Point {
					x: remaining_features_box.x + remaining_features_box.w / 2.0,
					y: remaining_features_box.y,
				},
			},
		))
	}

	// Draw the negative boxes which start at the max and go to the output, starting with the remaining features rect.
	x = rect.x + rect.w;
	let y = rect.y + box_height + chart_config.feature_contributions_bar_gap;
	// let negativeValues = series.values
	// 	.filter(({ value }) => value < 0)
	// 	.sort((a, b) => (a.value > b.value ? -1 : 1))
	// remainingFeaturesBoxWidth = 0
	// nRemainingFeatures = 0
	// let negativeValuesIndex = 0
	// while (negativeValuesIndex < negativeValues.length) {
	// 	let featureContributionValue = negativeValues[negativeValuesIndex]
	// 	if (featureContributionValue === undefined) throw Error()
	// 	let width = featureContributionValue.value * valueWidthMultiplier
	// 	if (width < -chartConfig.featureContributionsArrowDepth * 2) {
	// 		break
	// 	}
	// 	remainingFeaturesBoxWidth += width
	// 	nRemainingFeatures += 1
	// 	negativeValuesIndex += 1
	// }
	// if (remainingFeaturesBoxWidth < 0) {
	// 	let remainingFeaturesBox = {
	// 		h: boxHeight,
	// 		w: remainingFeaturesBoxWidth,
	// 		x,
	// 		y,
	// 	}
	// 	x += remainingFeaturesBoxWidth
	// 	drawFeatureContributionBox({
	// 		box: remainingFeaturesBox,
	// 		color: `${negativeColor}33`,
	// 		ctx,
	// 		direction: FeatureContributionsBoxDirection.Positive,
	// 		label: `${nRemainingFeatures} other features`,
	// 	})
	// 	hoverRegions.push(
	// 		featureContributionsChartHoverRegion({
	// 			box: remainingFeaturesBox,
	// 			color: `${negativeColor}33`,
	// 			direction: FeatureContributionsBoxDirection.Positive,
	// 			label: `${nRemainingFeatures} other features`,
	// 			tooltipOriginPixels: {
	// 				...remainingFeaturesBox,
	// 				x: remainingFeaturesBox.x + remainingFeaturesBox.w / 2,
	// 			},
	// 		}),
	// 	)
	// }
	// for (let i = negativeValuesIndex; i < negativeValues.length; i++) {
	// 	let featureContributionValue = negativeValues[i]
	// 	if (featureContributionValue === undefined) throw Error()
	// 	let width = featureContributionValue.value * valueWidthMultiplier
	// 	let valueBox = {
	// 		h: boxHeight,
	// 		w: width,
	// 		x,
	// 		y,
	// 	}
	// 	drawFeatureContributionBox({
	// 		box: valueBox,
	// 		color: negativeColor,
	// 		ctx,
	// 		direction: FeatureContributionsBoxDirection.Positive,
	// 		label: `${featureContributionValue.feature}`,
	// 	})
	// 	hoverRegions.push(
	// 		featureContributionsChartHoverRegion({
	// 			box: valueBox,
	// 			color: negativeColor,
	// 			direction: FeatureContributionsBoxDirection.Positive,
	// 			label: `${featureContributionValue.feature}`,
	// 			tooltipOriginPixels: {
	// 				...valueBox,
	// 				x: valueBox.x + valueBox.w / 2,
	// 			},
	// 		}),
	// 	)
	// 	x += width
	// }
	// // Draw the output value and label.
	// ctx.textBaseline = "bottom"
	// ctx.fillText(
	// 	`output`,
	// 	x - chartConfig.labelPadding,
	// 	rect.y + boxHeight + chartConfig.featureContributionsBarGap + boxHeight / 2,
	// )
	// ctx.textBaseline = "top"
	// ctx.fillText(
	// 	series.outputLabel,
	// 	x - chartConfig.labelPadding,
	// 	rect.y + boxHeight + chartConfig.featureContributionsBarGap + boxHeight / 2,
	// )

	DrawFeatureContributionsSeriesOutput { hover_regions }
}

struct DrawFeatureContributionsChartYAxisLabelsOptions<'a> {
	chart_config: &'a ChartConfig,
	rect: Rect,
	categories: &'a [&'a String],
	ctx: CanvasRenderingContext2d,
	width: f64,
}

fn draw_feature_contributions_chart_y_axis_labels(
	options: DrawFeatureContributionsChartYAxisLabelsOptions,
) {
	let DrawFeatureContributionsChartYAxisLabelsOptions {
		chart_config,
		rect,
		categories,
		ctx,
		width,
	} = options;
	ctx.set_text_align("end");
	for (i, label) in categories.iter().enumerate() {
		let label_offset = chart_config.feature_contributions_series_height / 2.0
			+ (chart_config.feature_contributions_series_gap
				+ chart_config.feature_contributions_series_height)
				* i.to_f64().unwrap();
		ctx.set_text_baseline("middle");
		ctx.fill_text(label, rect.x + rect.w, rect.y + label_offset)
			.unwrap();
	}
}

fn draw_feature_contributions_chart_overlay(
	options: DrawOverlayOptions<
		FeatureContributionsChartOverlayInfo,
		FeatureContributionsChartHoverRegionInfo,
	>,
) {
	let DrawOverlayOptions {
		active_hover_regions,
		ctx,
		overlay_info,
		overlay_div,
		chart_colors,
		chart_config,
	} = options;
	draw_feature_contribution_tooltips(DrawFeatureContributionTooltipsOptions {
		active_hover_regions,
		chart_box: overlay_info.chart_box,
		ctx: ctx.clone(),
		overlay_div,
		chart_colors,
		chart_config,
	});
	for active_hover_region in active_hover_regions {
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: active_hover_region.info.rect,
			color: "#00000022".to_owned(),
			ctx: ctx.clone(),
			direction: active_hover_region.info.direction.clone(),
			label: "".to_owned(),
			chart_config,
		});
	}
}

struct DrawFeatureContributionTooltipsOptions<'a> {
	chart_colors: &'a ChartColors,
	chart_config: &'a ChartConfig,
	active_hover_regions: &'a [ActiveHoverRegion<FeatureContributionsChartHoverRegionInfo>],
	chart_box: Rect,
	ctx: CanvasRenderingContext2d,
	overlay_div: HtmlElement,
}

fn draw_feature_contribution_tooltips(options: DrawFeatureContributionTooltipsOptions) {
	let DrawFeatureContributionTooltipsOptions {
		chart_colors,
		chart_config,
		active_hover_regions,
		chart_box,
		ctx,
		overlay_div,
	} = options;
	for active_hover_region in active_hover_regions {
		let label = TooltipLabel {
			color: active_hover_region.info.color.clone(),
			text: active_hover_region.info.label.clone(),
		};
		draw_tooltip(DrawTooltipOptions {
			center_horizontal: Some(true),
			container: overlay_div.clone(),
			labels: vec![label],
			origin: active_hover_region.info.tooltip_origin_pixels,
			chart_colors,
			chart_config,
			flip_y_offset: None,
		});
	}
}

struct FeatureContributionsChartHoverRegionOptions {
	rect: Rect,
	color: String,
	direction: FeatureContributionsBoxDirection,
	label: String,
	tooltip_origin_pixels: Point,
}

fn feature_contributions_chart_hover_region(
	options: FeatureContributionsChartHoverRegionOptions,
) -> HoverRegion<FeatureContributionsChartHoverRegionInfo> {
	let FeatureContributionsChartHoverRegionOptions {
		rect,
		color,
		direction,
		label,
		tooltip_origin_pixels,
	} = options;
	HoverRegion {
		distance: Box::new(move |x, y| (rect.x - x).powi(2) + (rect.y - y).powi(2)),
		hit_test: Box::new(move |x, y| {
			x > rect.x.min(rect.x + rect.w)
				&& x < rect.x.max(rect.x + rect.w)
				&& y > rect.y && y < rect.y + rect.h
		}),
		info: FeatureContributionsChartHoverRegionInfo {
			rect,
			color,
			direction,
			label,
			tooltip_origin_pixels,
		},
	}
}

struct DrawFeatureContributionBoxOptions<'a> {
	chart_config: &'a ChartConfig,
	rect: Rect,
	color: String,
	ctx: CanvasRenderingContext2d,
	direction: FeatureContributionsBoxDirection,
	label: String,
}

#[derive(Clone, Copy)]
enum FeatureContributionsBoxDirection {
	Positive,
	Negative,
}

fn draw_feature_contribution_box(options: DrawFeatureContributionBoxOptions) {
	let DrawFeatureContributionBoxOptions {
		chart_config,
		rect,
		color,
		ctx,
		direction,
		label,
	} = options;

	let text_padding = 4.0;
	let arrow_depth = if let FeatureContributionsBoxDirection::Negative = direction {
		chart_config.feature_contributions_arrow_depth
	} else {
		chart_config.feature_contributions_arrow_depth
	};
	let width = rect.w;

	ctx.save();
	ctx.set_stroke_style(&color.clone().into());
	ctx.set_fill_style(&color.into());
	ctx.set_line_width(1.0);
	ctx.set_line_cap("butt");

	ctx.begin_path();
	ctx.move_to(rect.x, rect.y);
	let draw_end_arrow = true;
	let draw_start_arrow = true;

	// Draw the endpoint.
	if draw_end_arrow {
		ctx.line_to(rect.x + width - arrow_depth, rect.y);
		ctx.line_to(rect.x + width, rect.y + rect.h / 2.0);
		ctx.line_to(rect.x + width - arrow_depth, rect.y + rect.h);
	} else {
		ctx.line_to(rect.x + width, rect.y);
		ctx.line_to(rect.x + width, rect.y + rect.h);
	}

	// Draw the startpoint.
	if draw_start_arrow {
		ctx.line_to(rect.x, rect.y + rect.h);
		ctx.line_to(rect.x + arrow_depth, rect.y + rect.h / 2.0);
		ctx.line_to(rect.x, rect.y);
	} else {
		ctx.line_to(rect.x, rect.y + rect.h);
		ctx.line_to(rect.x, rect.y);
	}

	ctx.fill();

	let label_width = ctx.measure_text(&label).unwrap().width();
	ctx.set_text_baseline("middle");
	ctx.set_text_align("center");
	ctx.set_fill_style(&"#fff".into());

	let max_label_width =
		rect.w.abs() - text_padding - chart_config.feature_contributions_arrow_depth * 2.0;
	if label_width <= max_label_width {
		ctx.fill_text(
			&label,
			rect.x + (rect.w + arrow_depth) / 2.0,
			rect.y + rect.h / 2.0,
		)
		.unwrap();
	}

	ctx.restore();
}
