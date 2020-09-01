import * as definitions from 'common/definitions'
import { MetricsRow } from 'common/metrics_row'
import { DateWindow, DateWindowInterval, intervalChartTitle } from 'common/time'
import { productionColor, trainingColor } from 'common/tokens'
import { Fragment, PinwheelInfo, h, renderPage, ui } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'

export type Props = {
	class: string
	classMetrics: Array<{
		className: string
		intervals: Array<{
			f1Score: {
				production: number | null
				training: number
			}
			label: string
			precision: {
				production: number | null
				training: number
			}
			recall: {
				production: number | null
				training: number
			}
		}>
	}>
	classes: string[]
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	id: string
	modelLayoutInfo: ModelLayoutInfo
	overall: {
		classMetrics: OverallClassMetrics[]
		label: string
	}
	pinwheelInfo: PinwheelInfo
	title: string
}

export type OverallClassMetrics = {
	className: string
	comparison: {
		falseNegativeFraction: {
			production: number | null
			training: number
		}
		falsePositiveFraction: {
			production: number | null
			training: number
		}
		trueNegativeFraction: {
			production: number | null
			training: number
		}
		truePositiveFraction: {
			production: number | null
			training: number
		}
	}
	confusionMatrix: {
		falseNegatives: number | null
		falsePositives: number | null
		trueNegatives: number | null
		truePositives: number | null
	}
	f1Score: {
		production: number | null
		training: number
	}
	precision: {
		production: number | null
		training: number
	}
	recall: {
		production: number | null
		training: number
	}
}

export default function ProductionMetricsPage(props: Props) {
	let selectedClassIndex = props.classes.indexOf(props.class)
	let selectedIntervalClassMetrics = props.classMetrics[selectedClassIndex]
	let selectedOverallClassMetrics = props.overall.classMetrics
		? props.overall.classMetrics[selectedClassIndex]
		: null

	let precisionIntervalChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		'Precision',
	)

	let recallIntervalChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		'Recall',
	)

	let f1ScoreIntervalChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		'F1 Score',
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
			lineStyle: ui.LineStyle.Dashed,
			pointStyle: ui.PointStyle.Hidden,
			title: 'Training Precision',
		},
		{
			color: productionColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.precision.production,
			})),
			title: 'Production Precision',
		},
	]

	let recallChartData = [
		{
			color: trainingColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.recall.training,
			})),
			lineStyle: ui.LineStyle.Dashed,
			pointStyle: ui.PointStyle.Hidden,
			title: 'Training Recall',
		},
		{
			color: productionColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.recall.production,
			})),
			title: 'Production Recall',
		},
	]

	let f1ScoreChartData = [
		{
			color: trainingColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.f1Score.training,
			})),
			lineStyle: ui.LineStyle.Dashed,
			pointStyle: ui.PointStyle.Hidden,
			title: 'Training F1 Score',
		},
		{
			color: productionColor,
			data: selectedIntervalClassMetrics.intervals.map((interval, index) => ({
				x: index,
				y: interval.f1Score.production,
			})),
			title: 'Production F1 Score',
		},
	]

	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.ProductionMetrics}
		>
			<ui.S1>
				<ui.H1>{'Production Metrics'}</ui.H1>
				<ui.TabBar>
					<ui.TabLink href="./">{'Overview'}</ui.TabLink>
					<ui.TabLink href="class_metrics" selected={true}>
						{'Class Metrics'}
					</ui.TabLink>
				</ui.TabBar>
				<div>
					<ui.Form>
						<ui.SelectField
							id="date-window-select"
							label="Date Window"
							name="date_window"
							options={Object.values(DateWindow)}
							value={props.dateWindow}
						/>
						<ui.SelectField
							id="class-select"
							label="Class"
							name="class"
							options={props.classes}
							value={props.class}
						/>
						<noscript>
							<ui.Button>{'Submit'}</ui.Button>
						</noscript>
					</ui.Form>
				</div>
				{selectedOverallClassMetrics !== null && (
					<>
						<ui.S2>
							<ui.H2>{'Precision and Recall'}</ui.H2>
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
								<ui.LineChart
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
								<ui.LineChart
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
								<ui.LineChart
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
							<ui.H2>{'Confusion Matrix'}</ui.H2>
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
							<ui.H2>{'Production v. Training Confusion Matrix'}</ui.H2>
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
