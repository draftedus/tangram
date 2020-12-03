use html::{component, html};
use num_traits::ToPrimitive;
use tangram_app_common::{
	predict::{PredictionResult, PredictionResultProps},
	tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken, UnknownColumnToken},
};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue},
	components::{BarChart, BoxChart},
};
use tangram_ui as ui;

#[derive(Clone)]
pub struct Props {
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub inner: Inner,
}

#[derive(Clone)]
pub enum Inner {
	PredictionForm(PredictionFormProps),
	PredictionResult(PredictionResultProps),
}

#[derive(Clone)]
pub struct PredictionFormProps {
	pub form: PredictForm,
}

#[derive(Clone)]
pub struct PredictForm {
	pub fields: Vec<Column>,
}

#[derive(Clone)]
pub enum Column {
	Unknown(UnknownFieldProps),
	Number(NumberFieldProps),
	Enum(EnumFieldProps),
	Text(TextFieldProps),
}

#[derive(Clone)]
pub struct UnknownFieldProps {
	pub name: String,
	pub value: String,
}

#[derive(Clone)]
pub struct NumberFieldProps {
	pub name: String,
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub value: String,
}

#[derive(Clone)]
pub struct EnumFieldProps {
	pub name: String,
	pub options: Vec<String>,
	pub value: String,
	pub histogram: Vec<(String, u64)>,
}

#[derive(Clone)]
pub struct TextFieldProps {
	pub name: String,
	pub value: String,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::PredictionForm(inner) => {
			html! {<PredictionFormPage form={inner.form} />}
		}
		Inner::PredictionResult(inner) => {
			html! {<PredictionResult props={inner}/>}
		}
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::Prediction}
		>
			<ui::S1>
				<ui::H1 center={false}>{"Prediction"}</ui::H1>
				{inner}
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}

#[component]
fn PredictionFormPage(form: PredictForm) {
	let fields = form
		.fields
		.into_iter()
		.map(|field| match field {
			Column::Unknown(field) => {
				html! { <UnknownField field={field}/> }
			}
			Column::Number(field) => {
				html! {<NumberField field={field} />}
			}
			Column::Enum(field) => {
				html! {<EnumField field={field} />}
			}
			Column::Text(field) => {
				html! {<TextField field={field} />}
			}
		})
		.collect::<Vec<_>>();
	html! {
		<ui::Form
			action={None}
			enc_type={None}
			id={None}
			post={None}
			autocomplete={None}
		>
			<div class="predict-form-grid">
				{fields}
			</div>
			<ui::Button
				button_type={ui::ButtonType::Submit}
				color={None}
				disabled={None}
				download={None}
				href={None}
				id={None}
			>
				{"Predict"}
			</ui::Button>
		</ui::Form>
	}
}

#[component]
fn UnknownField(field: UnknownFieldProps) {
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<UnknownColumnToken />
				</div>
				<ui::TextField
					autocomplete={None}
					disabled={None}
					required={None}
					readonly={None}
					placeholder={None}
					label={field.name.clone()}
					name={field.name}
					value={field.value}
				/>
			</div>
			<div></div>
		</>
	}
}

#[component]
fn NumberField(field: NumberFieldProps) {
	let column_chart_series = vec![BoxChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: vec![BoxChartPoint {
			label: field.name.to_owned(),
			x: 0.0,
			y: Some(BoxChartValue {
				min: field.min.to_f64().unwrap(),
				max: field.max.to_f64().unwrap(),
				p25: field.p25.to_f64().unwrap(),
				p50: field.p50.to_f64().unwrap(),
				p75: field.p75.to_f64().unwrap(),
			}),
		}],
		title: Some("quartiles".to_owned()),
	}];
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<NumberColumnToken />
				</div>
				<ui::TextField
					autocomplete={None}
					placeholder={None}
					readonly={None}
					disabled={None}
					required={None}
					label={field.name.clone()}
					name={field.name.clone()}
					value={field.value}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BoxChart
					title={None}
					x_axis_title={None}
					y_axis_title={None}
					y_min={None}
					y_max={None}
					class={"column-chart".to_owned()}
					hide_legend={true}
					id={field.name}
					series={column_chart_series}
					should_draw_x_axis_labels={false}
					should_draw_y_axis_labels={false}
				/>
			</div>
		</>
	}
}

#[component]
fn EnumField(field: EnumFieldProps) {
	let series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: field
			.histogram
			.iter()
			.enumerate()
			.map(|(index, (label, value))| BarChartPoint {
				label: label.to_owned(),
				x: index.to_f64().unwrap(),
				y: Some(value.to_f64().unwrap()),
			})
			.collect::<Vec<_>>(),
		title: Some("histogram".to_owned()),
	}];
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<EnumColumnToken />
				</div>
				<ui::SelectField
					id={None}
					required={None}
					disabled={None}
					placeholder={None}
					label={field.name.clone()}
					name={field.name.clone()}
					options={field.options.iter().map(|option| ui::SelectFieldOption {
						text: option.to_owned(),
						value: option.to_owned(),
					}).collect::<Vec<_>>()}
					value={field.value}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BarChart
					title={None}
					x_axis_title={None}
					y_axis_title={None}
					y_axis_grid_line_interval={None}
					y_min={None}
					y_max={None}
					class={"column-chart".to_owned()}
					hide_legend={true}
					id={field.name}
					series={series}
					should_draw_x_axis_labels={false}
					should_draw_y_axis_labels={false}
					group_gap={None}
				/>
			</div>
		</>
	}
}

#[component]
fn TextField(field: TextFieldProps) {
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<TextColumnToken />
				</div>
				<ui::TextField
					readonly={None}
					disabled={None}
					required={None}
					placeholder={None}
					autocomplete={None}
					label={field.name.clone()}
					name={field.name}
					value={field.value}
				/>
			</div>
			<div />
		</>
	}
}
