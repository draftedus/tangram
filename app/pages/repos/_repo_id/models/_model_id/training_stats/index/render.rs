use tangram_app_common::metrics_row::MetricsRow;
use tangram_app_common::tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::html::{self, component, html};
use tangram_ui as ui;

#[derive(Clone)]
pub struct Props {
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub target_column_stats_table: TargetColumnStatsTableProps,
	pub column_stats_table: ColumnStatsTableProps,
	pub column_count: usize,
	pub row_count: usize,
}

#[derive(Clone)]
pub struct ColumnStatsTableProps {
	pub column_stats_table_rows: Vec<ColumnStatsTableRow>,
}

#[derive(Clone)]
pub struct TargetColumnStatsTableProps {
	pub target_column_stats_table_row: ColumnStatsTableRow,
}

#[derive(Clone)]
pub struct ColumnStatsTableRow {
	pub invalid_count: Option<usize>,
	pub max: Option<f32>,
	pub mean: Option<f32>,
	pub min: Option<f32>,
	pub name: String,
	pub std: Option<f32>,
	pub column_type: ColumnType,
	pub unique_count: Option<usize>,
	pub variance: Option<f32>,
}

#[derive(PartialEq, Clone)]
pub enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let description = "The following are statistics for the columns in your dataset. A column is a Number column if every non-null value in the column parses as a finite floating point number. A column is an Enum column if it is not a Number column and the count of unique non-null values is less than 100. A column is a Text column if it is neither a Number column nor an Enum column.";
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::TrainingStats}
		>
			<ui::S1>
				<ui::H1 center={false}>{"Training Stats"}</ui::H1>
				<ui::S2>
					<ui::H2 center={false}>{"Target Column"}</ui::H2>
					<TargetColumnStatsTable props={props.target_column_stats_table} />
				</ui::S2>
				<ui::S2>
					<ui::H2 center={false}>{"Columns"}</ui::H2>
					<ui::P>{description}</ui::P>
					<MetricsRow>
					</MetricsRow>
					<ui::Card>
						<ui::NumberChart
							title={"Rows".to_owned()}
							value={props.row_count.to_string()}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title={"Columns".to_owned()}
							value={props.column_count.to_string()}
						/>
					</ui::Card>
					<ColumnStatsTable props={props.column_stats_table} />
				</ui::S2>
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}

#[component]
fn TargetColumnStatsTable(props: TargetColumnStatsTableProps) {
	html! {
		<ui::Table width={"100%".to_owned()}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell
						text_align={None}
						color={None}
						expand={false}
					>
						{"Column"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						text_align={None}
						color={None}
						expand={false}
					>
						{"Type"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						text_align={None}
						color={None}
						expand={false}
					>
						{"Unique Count"}
					</ui::TableHeaderCell>
					{if props.target_column_stats_table_row.column_type == ColumnType::Number {
						Some(html! {
							<>
								<ui::TableHeaderCell
									text_align={None}
									color={None}
									expand={false}
								>
									{"Min"}
								</ui::TableHeaderCell>
								<ui::TableHeaderCell
									text_align={None}
									color={None}
									expand={false}
								>
									{"Max"}
								</ui::TableHeaderCell>
								<ui::TableHeaderCell
									text_align={None}
									color={None}
									expand={false}
								>
									{"Mean"}
								</ui::TableHeaderCell>
								<ui::TableHeaderCell
									text_align={None}
									color={None}
									expand={false}
								>
									{"Std"}
								</ui::TableHeaderCell>
							</>
						})
					} else {
						None
					}}
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				<ui::TableRow color={None}>
					<ui::TableCell
						expand={false}
						color={None}
					>
						<ui::Link
							title={None}
							class={None}
							href={format!("./columns/{}", props.target_column_stats_table_row.name)}
						>
							{props.target_column_stats_table_row.name}
						</ui::Link>
					</ui::TableCell>
					<ui::TableCell
						expand={false}
						color={None}
					>
						{
							column_type_token(&props.target_column_stats_table_row.column_type)
						}
					</ui::TableCell>
					<ui::TableCell
						expand={false}
						color={None}
					>
						{props.target_column_stats_table_row.unique_count.map(|unique_count| unique_count.to_string())}
					</ui::TableCell>
					{if props.target_column_stats_table_row.column_type == ColumnType::Number {
						Some(html! {
						<>
							<ui::TableCell color={None} expand={None}>
								{props.target_column_stats_table_row.min.unwrap().to_string()}
							</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{props.target_column_stats_table_row.max.unwrap().to_string()}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{props.target_column_stats_table_row.mean.unwrap().to_string()}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{props.target_column_stats_table_row.std.unwrap().to_string()}
						</ui::TableCell>
						</>
					})
				} else {
						None
					}}
				</ui::TableRow>
			</ui::TableBody>
		</ui::Table>
	}
}

#[component]
fn ColumnStatsTable(props: ColumnStatsTableProps) {
	html! {
		<ui::Table width={"100%".to_owned()}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Column"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Type"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Unique Values Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Null Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Min"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Max"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Mean"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						text_align={None}
						expand={None}
					>
						{"Std"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.column_stats_table_rows.iter().map(|column_stats| html! {
					<ui::TableRow color={None}>
						<ui::TableCell color={None} expand={None}>
							{if column_stats.column_type == ColumnType::Unknown {
								html! {
									<>
										{column_stats.name.clone()}
									</>
								}
							} else {
								html! {
									<ui::Link
										class={None}
										title={None}
										href={format!("./columns/{}", column_stats.name)}
									>
										{column_stats.name.clone()}
									</ui::Link>
								}
							}}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{column_type_token(&column_stats.column_type)}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{column_stats.unique_count.map(|unique_count| unique_count.to_string())}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{column_stats.invalid_count.map(|invalid_count| invalid_count.to_string())}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{column_stats.min.map(ui::format_number)}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{column_stats.max.map(ui::format_number)}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{column_stats.mean.map(ui::format_number)}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{column_stats.std.map(ui::format_number)}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

fn column_type_token(column_type: &ColumnType) -> Option<html::Node> {
	match column_type {
		ColumnType::Number => Some(html! {<NumberColumnToken />}),
		ColumnType::Enum => Some(html! { <EnumColumnToken /> }),
		ColumnType::Text => Some(html! { <TextColumnToken /> }),
		ColumnType::Unknown => None,
	}
}
