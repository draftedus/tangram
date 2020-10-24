import { RegressorProps } from './props'
import { LineChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { baselineColor, trainingColor } from 'common/tokens'
import { h } from 'preact'

export function RegressorIndexPage(props: RegressorProps) {
	let lossesChartData = props.lossesChartData && [
		{
			color: ui.colors.blue,
			data: props.lossesChartData.map((loss, index) => ({
				x: index,
				y: loss,
			})),
			title: 'loss',
		},
	]
	return (
		<ui.S1>
			<ui.SpaceBetween>
				<ui.H1>{'Overview'}</ui.H1>
			</ui.SpaceBetween>
			<ui.S2>
				<ui.H2>{'Training Summary'}</ui.H2>
				<ui.P>
					{'Your dataset contained '}
					<b>
						{props.trainingSummary.trainRowCount +
							props.trainingSummary.testRowCount}
					</b>
					{' rows and '}
					<b>{props.trainingSummary.columnCount}</b>
					{' columns. '}
					<b>{props.trainingSummary.trainRowCount}</b>
					{' of the rows were used in training and '}
					<b>{props.trainingSummary.testRowCount}</b>
					{' were used in testing. The model with the highest '}
					<b>{props.trainingSummary.modelComparisonMetricTypeName}</b>
					{' was chosen. The best model is a '}
					<b>{props.trainingSummary.chosenModelTypeName}</b>
					{'.'}
				</ui.P>
			</ui.S2>
			<ui.S2>
				<ui.H2>{'Metrics'}</ui.H2>
				<ui.P>
					{
						'Your model was evaluated on the test dataset and had a root mean squared error of '
					}
					<b>{ui.formatNumber(props.metrics.rmse)}</b>
					{'. This is compared with the baseline root mean squared error of '}
					<b>{ui.formatNumber(props.metrics.baselineRmse)}</b>
					{
						', which is the accuracy achieved if the model always predicted the mean of the target column.'
					}
				</ui.P>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={baselineColor}
						colorB={trainingColor}
						title="Root Mean Squared Error"
						valueA={props.metrics.baselineRmse}
						valueATitle="Baseline Root Mean Squared Error"
						valueB={props.metrics.rmse}
						valueBTitle="Root Mean Squared Error"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
				{lossesChartData && (
					<LineChart
						data={lossesChartData}
						id="loss_curve"
						title="Training Loss Curve"
						xAxisTitle="Epoch"
						yAxisTitle="Loss"
						yMin={0}
					/>
				)}
			</ui.S2>
		</ui.S1>
	)
}
