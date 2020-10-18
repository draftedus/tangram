import { LineChart, LineStyle, PointStyle } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { DateWindowSelectField } from 'common/date_window_select_field'
import * as definitions from 'common/definitions'
import { MetricsRow } from 'common/metrics_row'
import { DateWindow, DateWindowInterval, intervalChartTitle } from 'common/time'
import { productionColor, trainingColor } from 'common/tokens'
import { h } from 'preact'

export type Props = {
	accuracyChart: {
		data: Array<{
			accuracy: number | null
			label: string
		}>
		trainingAccuracy: number
	}
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	id: string
	overall: {
		accuracy: {
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
		trueValuesCount: number
	}
	true_values_count_chart: Array<{ count: number; label: string }>
}

export function BinaryClassifierProductionMetricsIndexPage(props: Props) {
	let chartLabels = props.accuracyChart.data.map(entry => entry.label)
	let accuracyData = [
		{
			color: trainingColor,
			data: props.accuracyChart.data.map((_, index) => ({
				x: index,
				y: props.accuracyChart.trainingAccuracy,
			})),
			lineStyle: LineStyle.Dashed,
			pointStyle: PointStyle.Hidden,
			title: 'Training Accuracy',
		},
		{
			color: productionColor,
			data: props.accuracyChart.data.map((entry, index) => ({
				x: index,
				y: entry.accuracy,
			})),
			title: 'Production Accuracy',
		},
	]
	let accuracyChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		'Accuracy',
	)
	return (
		<ui.S1>
			<ui.H1>{'Production Metrics'}</ui.H1>
			<ui.S2>
				<ui.Form>
					<DateWindowSelectField dateWindow={props.dateWindow} />
					<noscript>
						<ui.Button>{'Submit'}</ui.Button>
					</noscript>
				</ui.Form>
				<ui.P>
					{'You have logged '}
					<b>{props.overall.trueValuesCount}</b>
					{' true values for this date range.'}
				</ui.P>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							title="True Value Count"
							value={props.overall.trueValuesCount.toString()}
						/>
					</ui.Card>
				</MetricsRow>
			</ui.S2>
			<ui.S2>
				<ui.H2>{'Accuracy'}</ui.H2>
				<ui.P>{definitions.accuracy}</ui.P>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						title="Accuracy"
						valueA={props.overall.accuracy?.training}
						valueATitle="Training"
						valueB={props.overall.accuracy?.production}
						valueBTitle="Production"
						valueFormatter={value => ui.formatPercent(value, 2)}
					/>
				</ui.Card>
				<ui.Card>
					<LineChart
						data={accuracyData}
						id="accuracy"
						labels={chartLabels}
						title={accuracyChartTitle}
						xAxisGridLineInterval={{ k: 1, p: 0 }}
						yMax={1}
						yMin={0}
					/>
				</ui.Card>
			</ui.S2>
		</ui.S1>
	)
}