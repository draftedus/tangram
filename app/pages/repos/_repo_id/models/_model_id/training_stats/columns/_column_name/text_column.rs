use super::page::TextProps;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_deps::html::{self, component, html};
use tangram_deps::num_traits::ToPrimitive;
use tangram_ui as ui;

#[component]
pub fn TextColumn(props: TextProps) {
	let series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_string(),
		data: props
			.tokens
			.iter()
			.enumerate()
			.map(|(i, token)| BarChartPoint {
				label: token.token.clone(),
				x: i.to_f64().unwrap(),
				y: Some(token.count.to_f64().unwrap()),
			})
			.collect(),
		title: Some("Token Count".to_owned()),
	}];
	html! {
		<ui::S1>
			<ui::H1 center={false}>{props.name}</ui::H1>
			<ui::S2>
				<ui::Card>
					<BarChart
						class={None}
						hide_legend={None}
						id={None}
						series={series}
						should_draw_x_axis_labels={None}
						y_axis_grid_line_interval={None}
						should_draw_y_axis_labels={None}
						group_gap={None}
						title={format!("{} Most Frequent Tokens", props.tokens.len())}
						x_axis_title={None}
						y_axis_title={None}
						y_max={None}
						y_min={None}
					/>
				</ui::Card>
				<ui::Table width={"100%".to_owned()}>
					<ui::TableHeader>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Token"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Count"}
						</ui::TableHeaderCell>
					</ui::TableHeader>
					<ui::TableBody>
					{props.tokens.iter().map(|token| html! {
						<ui::TableRow color={None}>
							<ui::TableCell
								expand={None}
								color={None}
							>
								{token.token.to_string()}
							</ui::TableCell>
							<ui::TableCell
								expand={None}
								color={None}
							>
								{token.count.to_string()}
							</ui::TableCell>
						</ui::TableRow>
					}).collect::<Vec<_>>()}
					</ui::TableBody>
				</ui::Table>
			</ui::S2>
		</ui::S1>
	}
}
