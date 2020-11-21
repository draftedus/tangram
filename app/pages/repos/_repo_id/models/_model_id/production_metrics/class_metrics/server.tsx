import { Props } from "./props"
import { LineChart, LineStyle, PointStyle } from "@tangramhq/charts"
import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { ClassSelectField } from "common/class_select_field"
import { DateWindowSelectField } from "common/date_window_select_field"
import * as definitions from "common/definitions"
import { MetricsRow } from "common/metrics_row"
import { renderPage } from "common/render"
import { intervalChartTitle } from "common/time"
import { productionColor, trainingColor } from "common/tokens"
import { ModelLayout, ModelSideNavItem } from "layouts/model_layout"
import { Fragment, h } from "preact"

export default (pageInfo: PageInfo, props: Props) => {
	let selectedClassIndex = props.classes.indexOf(props.class)
	let selectedIntervalClassMetrics = props.classMetrics[selectedClassIndex]
	if (selectedIntervalClassMetrics === undefined) throw Error()
	let selectedOverallClassMetrics = props.overall.classMetrics
		? props.overall.classMetrics[selectedClassIndex]
		: null
	if (selectedOverallClassMetrics === undefined) throw Error()

	let precisionIntervalChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		"Precision",
	)

	let recallIntervalChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		"Recall",
	)

	let f1ScoreIntervalChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		"F1 Score",
	)

	let chartLabels = selectedIntervalClassMetrics.intervals.map(
		interval => interval.label,
	)
	let precisionChartData = [
		{
			color: trainingColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.precision.training,
			})),
			lineStyle: LineStyle.Dashed,
			pointStyle: PointStyle.Hidden,
			title: "Training Precision",
		},
		{
			color: productionColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.precision.production,
			})),
			title: "Production Precision",
		},
	]

	let recallChartData = [
		{
			color: trainingColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.recall.training,
			})),
			lineStyle: LineStyle.Dashed,
			pointStyle: PointStyle.Hidden,
			title: "Training Recall",
		},
		{
			color: productionColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.recall.production,
			})),
			title: "Production Recall",
		},
	]

	let f1ScoreChartData = [
		{
			color: trainingColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.f1Score.training,
			})),
			lineStyle: LineStyle.Dashed,
			pointStyle: PointStyle.Hidden,
			title: "Training F1 Score",
		},
		{
			color: productionColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.f1Score.production,
			})),
			title: "Production F1 Score",
		},
	]

	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pageInfo={pageInfo}
			selectedItem={ModelSideNavItem.ProductionMetrics}
		>
			<ui.S1>
				<ui.H1>{"Production Metrics"}</ui.H1>
				<ui.TabBar>
					<ui.TabLink href="./">{"Overview"}</ui.TabLink>
					<ui.TabLink href="class_metrics" selected={true}>
						{"Class Metrics"}
					</ui.TabLink>
				</ui.TabBar>
				<ui.Form>
					<DateWindowSelectField dateWindow={props.dateWindow} />
					<ClassSelectField class={props.class} classes={props.classes} />
					<noscript>
						<ui.Button>{"Submit"}</ui.Button>
					</noscript>
				</ui.Form>
				{selectedOverallClassMetrics !== null && (
					<>
						<ui.S2>
							<ui.H2>{"Precision and Recall"}</ui.H2>
							<ui.P>{definitions.precisionRecall}</ui.P>
							<MetricsRow>
								<ui.Card>
									<ui.NumberComparisonChart
										colorA={trainingColor}
										colorB={productionColor}
										title="Precision"
										valueA={selectedOverallClassMetrics.precision.training}
										valueATitle="Training"
										valueB={selectedOverallClassMetrics.precision.production}
										valueBTitle="Production"
										valueFormatter={value => ui.formatPercent(value, 2)}
									/>
								</ui.Card>
								<ui.Card>
									<ui.NumberComparisonChart
										colorA={trainingColor}
										colorB={productionColor}
										title="Recall"
										valueA={selectedOverallClassMetrics.recall.training}
										valueATitle="Training"
										valueB={selectedOverallClassMetrics.recall.production}
										valueBTitle="Production"
										valueFormatter={value => ui.formatPercent(value, 2)}
									/>
								</ui.Card>
							</MetricsRow>
							<ui.Card>
								<LineChart
									data={precisionChartData}
									id="precision_intervals"
									labels={chartLabels}
									title={precisionIntervalChartTitle}
									xAxisGridLineInterval={{ k: 1, p: 0 }}
									yMax={1}
									yMin={0}
								/>
							</ui.Card>
							<ui.Card>
								<LineChart
									data={recallChartData}
									id="recall_intervals"
									labels={chartLabels}
									title={recallIntervalChartTitle}
									xAxisGridLineInterval={{ k: 1, p: 0 }}
									yMax={1}
									yMin={0}
								/>
							</ui.Card>
							<MetricsRow>
								<ui.Card>
									<ui.NumberComparisonChart
										colorA={trainingColor}
										colorB={productionColor}
										title="F1 Score"
										valueA={selectedOverallClassMetrics.f1Score.training}
										valueATitle="Training"
										valueB={selectedOverallClassMetrics.f1Score.production}
										valueBTitle="Production"
										valueFormatter={value => ui.formatPercent(value, 2)}
									/>
								</ui.Card>
							</MetricsRow>
							<ui.Card>
								<LineChart
									data={f1ScoreChartData}
									id="f1_intervals"
									labels={chartLabels}
									title={f1ScoreIntervalChartTitle}
									xAxisGridLineInterval={{ k: 1, p: 0 }}
									yMax={1}
									yMin={0}
								/>
							</ui.Card>
						</ui.S2>
						<ui.S2>
							<ui.H2>{"Confusion Matrix"}</ui.H2>
							<ui.P>{definitions.confusionMatrix}</ui.P>
							<ui.ConfusionMatrix
								classLabel={props.class}
								falseNegatives={
									selectedOverallClassMetrics.confusionMatrix.falseNegatives
								}
								falsePositives={
									selectedOverallClassMetrics.confusionMatrix.falsePositives
								}
								trueNegatives={
									selectedOverallClassMetrics.confusionMatrix?.trueNegatives
								}
								truePositives={
									selectedOverallClassMetrics.confusionMatrix?.truePositives
								}
							/>
						</ui.S2>
						<ui.S2>
							<ui.H2>{"Production v. Training Confusion Matrix"}</ui.H2>
							<ui.P>{definitions.confusionMatrix}</ui.P>
							{selectedOverallClassMetrics.comparison && (
								<ui.ConfusionMatrixComparison
									classLabel={props.class}
									colorA={trainingColor}
									colorB={productionColor}
									valueA={{
										falseNegative:
											selectedOverallClassMetrics.comparison
												.falseNegativeFraction.training,
										falsePositive:
											selectedOverallClassMetrics.comparison
												.falsePositiveFraction.training,
										trueNegative:
											selectedOverallClassMetrics.comparison
												.trueNegativeFraction.training,
										truePositive:
											selectedOverallClassMetrics.comparison
												.truePositiveFraction.training,
									}}
									valueATitle="Training"
									valueATitleColor="#ccc"
									valueB={{
										falseNegative:
											selectedOverallClassMetrics.comparison
												.falseNegativeFraction.production,
										falsePositive:
											selectedOverallClassMetrics.comparison
												.falsePositiveFraction.production,
										trueNegative:
											selectedOverallClassMetrics.comparison
												.trueNegativeFraction.production,
										truePositive:
											selectedOverallClassMetrics.comparison
												.truePositiveFraction.production,
									}}
									valueBTitle="Production"
									valueBTitleColor="#ccc"
									valueFormatter={value => ui.formatPercent(value, 2)}
								/>
							)}
						</ui.S2>
					</>
				)}
			</ui.S1>
		</ModelLayout>,
	)
}
