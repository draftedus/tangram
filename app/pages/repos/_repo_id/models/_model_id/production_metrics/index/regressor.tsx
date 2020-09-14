import { LineChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { MetricsRow } from 'common/metrics_row'
import {
	DateWindow,
	DateWindowInterval,
	DateWindowSelectField,
	intervalChartTitle,
} from 'common/time'
import { productionColor, trainingColor } from 'common/tokens'
import { h } from 'preact'

export type Props = {
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	mse: {
		production: number | null
		training: number
	}
	mseChart: {
		data: Array<{
			label: string
			mse: number | null
		}>
		trainingMse: number
	}
	rmse: {
		production: number | null
		training: number
	}
	trueValuesCount: number
}

export function RegressorProductionMetricsPage(props: Props) {
	let mseData = [
		{
			color: trainingColor,
			data: [
				{
					x: 0,
					y: props.mseChart.trainingMse,
				},
				{
					x: props.mseChart.data.length - 1,
					y: props.mseChart.trainingMse,
				},
			],
			title: 'Training Root Mean Squared Error',
		},
		{
			color: productionColor,
			data: props.mseChart.data.map((entry, index) => ({
				label: entry.label,
				x: index,
				y: entry.mse,
			})),
			title: 'Production Mean Squared Error',
		},
	]
	let mseChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		'Mean Squared Error',
	)
	return (
		<ui.S1>
			<ui.H1>{'Production Metrics'}</ui.H1>
			<ui.S2>
				<DateWindowSelectField dateWindow={props.dateWindow} />
				<ui.P>
					{'You have logged '}
					<b>{props.trueValuesCount}</b>
					{' true values for this date range.'}
				</ui.P>
				{mseData && (
					<ui.Card>
						<LineChart
							data={mseData}
							id="mse"
							labels={props.mseChart.data.map(entry => entry.label)}
							title={mseChartTitle}
							xAxisGridLineInterval={{ k: 1, p: 0 }}
							yAxisTitle="Root Mean Squared Error"
						/>
					</ui.Card>
				)}
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							title="True Value Count"
							value={props.trueValuesCount.toString()}
						/>
					</ui.Card>
				</MetricsRow>
				<MetricsRow>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={trainingColor}
							colorB={productionColor}
							title="Root Mean Squared Error"
							valueA={props.rmse?.training}
							valueATitle="Training"
							valueB={props.rmse?.production}
							valueBTitle="Production"
							valueFormatter={value => ui.formatNumber(value)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={trainingColor}
							colorB={productionColor}
							title="Mean Squared Error"
							valueA={props.mse?.training}
							valueATitle="Training"
							valueB={props.mse?.production}
							valueBTitle="Production"
							valueFormatter={value => ui.formatNumber(value)}
						/>
					</ui.Card>
				</MetricsRow>
			</ui.S2>
		</ui.S1>
	)
}
