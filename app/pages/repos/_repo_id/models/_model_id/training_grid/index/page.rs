use html::{component, html};
use tangram_app_layouts::document::PageInfo;
use tangram_app_layouts::model_layout::ModelLayoutInfo;
use tangram_app_layouts::model_layout::{ModelLayout, ModelSideNavItem};
use tangram_ui as ui;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub num_models: usize,
	pub trained_models_metrics: Vec<TrainedModel>,
	pub best_model_metrics: TrainedModel,
	pub model_comparison_metric_name: String,
	pub best_model_hyperparameters: Vec<(String, String)>,
}

#[derive(serde::Serialize, Clone)]
pub struct TrainedModel {
	pub identifier: String,
	pub model_comparison_metric_value: f32,
	pub model_type: String,
	pub time: String,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let description = format!(
		"Tangram trained {} models. The models were compared by evaluating their performance on a hold out portion of the dataset and the model with the best score was chosen.",
		props.num_models,
	);
	let html = html! {
		<ModelLayout page_info={page_info} info={props.model_layout_info} selected_item={ModelSideNavItem::TrainingGrid}>
			<ui::S1>
				<ui::H1 center={None}>{"Training Summary"}</ui::H1>
				<ui::P>
					{description}
				</ui::P>
				<ui::S2>
					<ui::H2 center={None}>{"Best Model Metrics"}</ui::H2>
					<WinningModelMetricsTable best_model={props.best_model_metrics} model_comparison_metric_name={props.model_comparison_metric_name.clone()}/>
				</ui::S2>
				<ui::S2>
					<ui::H2 center={None}>{"Best Model Hyperparameters"}</ui::H2>
					<ModelHyperparametersTable hyperparameters={props.best_model_hyperparameters} />
				</ui::S2>
				<ui::S2>
					<ui::H2 center={None}>{"All Models"}</ui::H2>
					<AllTrainedModelsMetricsTable trained_models={props.trained_models_metrics} model_comparison_metric_name={props.model_comparison_metric_name}/>
				</ui::S2>
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}

#[component]
fn WinningModelMetricsTable(best_model: TrainedModel, model_comparison_metric_name: String) {
	html! {
		<ui::Table width={Some("100%".to_owned())}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Number"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Type"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Training Time"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						{model_comparison_metric_name}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableRow color={None}>
				<ui::TableCell color={None} expand={None}>
					{best_model.identifier}
				</ui::TableCell>
				<ui::TableCell color={None} expand={None}>
					{best_model.model_type}
				</ui::TableCell>
				<ui::TableCell color={None} expand={None}>
					{best_model.time}
				</ui::TableCell>
				<ui::TableCell color={None} expand={None}>
					{best_model.model_comparison_metric_value.to_string()}
				</ui::TableCell>
			</ui::TableRow>
		</ui::Table>
	}
}

#[component]
fn AllTrainedModelsMetricsTable(
	trained_models: Vec<TrainedModel>,
	model_comparison_metric_name: String,
) {
	html! {
		<ui::Table width={Some("100%".to_owned())}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Number"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Type"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Training Time"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						{model_comparison_metric_name}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			{trained_models.into_iter().map(|trained_model| {
				html! {
					<ui::TableRow color={None}>
						<ui::TableCell color={None} expand={None}>
							{trained_model.identifier}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{trained_model.model_type}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{trained_model.time}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{trained_model.model_comparison_metric_value.to_string()}
						</ui::TableCell>
					</ui::TableRow>
				}
			}).collect::<Vec<_>>()}
		</ui::Table>
	}
}

#[component]
fn ModelHyperparametersTable(hyperparameters: Vec<(String, String)>) {
	html! {
		<ui::Table width={Some("100%".to_owned())}>
		{hyperparameters.into_iter().map(|(hyperparam_name, hyperparam_value)| {
			html! {
				<ui::TableRow color={None}>
					<ui::TableHeaderCell color={None} text_align={None} expand={Some(false)}>
						{hyperparam_name}
					</ui::TableHeaderCell>
					<ui::TableCell color={None} expand={Some(true)}>
						{hyperparam_value}
					</ui::TableCell>
				</ui::TableRow>
			}
		}).collect::<Vec<_>>()}
		</ui::Table>
	}
}
