import {
	baselineColor,
	baselineTextColor,
	trainingColor,
	trainingTextColor,
} from 'common/tokens'
import { h, ui } from 'deps'

export type Props = {
	id: string
	metrics: {
		baselineMse: number
		baselineRmse: number
		mse: number
		rmse: number
	}
	trainingSummary: {
		chosenModelTypeName: string
		columnCount: number
		modelComparisonMetricTypeName: string
		rowCount: number
		targetColumn: string
		testFraction: number
	}
}

export function RegressorIndexPage(props: Props) {
	return (
		<ui.S1>
			<ui.SpaceBetween>
				<ui.H1>Overview</ui.H1>
			</ui.SpaceBetween>
			<ui.S2>
				<ui.H2>Training Summary</ui.H2>
				<ui.P>
					{'Your dataset included '}
					<b>{props.trainingSummary.rowCount}</b>
					{' rows and '}
					<b>{props.trainingSummary.columnCount}</b>
					{' columns. '}
					<b>{ui.formatPercent(1 - props.trainingSummary.testFraction)}</b>
					{' of the rows were used in training and '}
					<b>{ui.formatPercent(props.trainingSummary.testFraction)}</b>
					{' were used in testing. '}
					{'The model with the highest '}
					<b>{props.trainingSummary.modelComparisonMetricTypeName}</b>
					{' was chosen. The model is a '}
					<b>{props.trainingSummary.chosenModelTypeName}</b>.
				</ui.P>
			</ui.S2>
			<ui.S2>
				<ui.H2>Metrics</ui.H2>
				<ui.P>
					{
						'Your model was evaluated on the test dataset and had a root mean squared error of '
					}
					<b>{ui.formatNumber(props.metrics.rmse)}</b>.
					{' This is compared with the baseline root mean squared error of '}
					<b>{ui.formatNumber(props.metrics.baselineRmse)}</b>
					{
						', which is the accuracy achieved if the model always predicted the mean of the target column.'
					}
				</ui.P>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={baselineColor}
						colorB={trainingColor}
						textColorA={baselineTextColor}
						textColorB={trainingTextColor}
						title="Root Mean Squared Error"
						valueA={props.metrics.baselineRmse}
						valueATitle="Baseline Root Mean Squared Error"
						valueB={props.metrics.rmse}
						valueBTitle="Root Mean Squared Error"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
			</ui.S2>
		</ui.S1>
	)
}
