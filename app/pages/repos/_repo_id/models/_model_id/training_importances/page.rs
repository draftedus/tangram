use html::{component, html};
use num_traits::ToPrimitive;
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

pub struct Props {
	pub id: String,
	pub n_features: usize,
	pub feature_importances: Vec<FeatureImportance>,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(Clone)]
pub struct FeatureImportance {
	pub feature_importance_value: f32,
	pub feature_name: String,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::TrainingImportances}
		>
			<ui::S1>
				<ui::H1 center={false}>
					{"Training Feature Importances"}
				</ui::H1>
				<ui::P>
					{format!("Your model had a total of {} features.", props.n_features)}
				</ui::P>
				<FeatureImportancesTable feature_importances={props.feature_importances} n_features={props.n_features} />
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}

#[component]
fn FeatureImportancesTable(feature_importances: Vec<FeatureImportance>, n_features: usize) {
	let bar_chart_series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: feature_importances
			.iter()
			.enumerate()
			.map(|(index, feature_importance)| BarChartPoint {
				label: feature_importance.feature_name.clone(),
				x: index.to_f64().unwrap(),
				y: Some(
					feature_importance
						.feature_importance_value
						.to_f64()
						.unwrap(),
				),
			})
			.collect(),
		title: Some("Feature Importance".to_owned()),
	}];
	html! {
		<>
			<ui::Card>
				<BarChart
					class={None}
					group_gap={None}
					hide_legend={None}
					id={"feature_importances".to_owned()}
					series={bar_chart_series}
					should_draw_x_axis_labels={false}
					should_draw_y_axis_labels={false}
					title={format!("Feature Importances for Top {} Features", n_features)}
					x_axis_title={"Feature Name".to_owned()}
					y_axis_grid_line_interval={None}
					y_axis_title={"Feature Importance Value".to_owned()}
					y_min={None}
					y_max={None}
				/>
			</ui::Card>
			<ui::Table width={"100%".to_owned()}>
				<ui::TableHeader>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Feature Name"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Feature Importance Value"}
					</ui::TableHeaderCell>
				</ui::TableHeader>
				<ui::TableBody>
				{feature_importances.iter().map(|feature_importance| html! {
					<ui::TableRow color={None}>
						<ui::TableCell color={None} expand={None}>
							{feature_importance.feature_name.clone()}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{feature_importance.feature_importance_value.to_string()}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
		</>
	}
}
