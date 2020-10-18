import * as ui from '@tangramhq/ui'
import { MetricsRow } from 'common/metrics_row'
import { h } from 'preact'

export type Props = {
	id: string
	metrics: {
		accuracy: number
		aucRoc: number
		baselineAccuracy: number
		precision: number
		recall: number
	}
	title: string
	trainingSummary: {
		chosenModelTypeName: string
		columnCount: number
		modelComparisonMetricTypeName: string
		testRowCount: number
		trainRowCount: number
	}
}

export function BinaryClassifierIndexPage(props: Props) {
	return (
		<ui.S1>
			<ui.H1>{'Overview'}</ui.H1>
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
						'Your model was evaluated on the test dataset and accurately classified '
					}
					<b>{ui.formatPercent(props.metrics.accuracy, 2)}</b>
					{' of the examples. This is compared with the baseline accuracy of '}
					<b>{ui.formatPercent(props.metrics.baselineAccuracy, 2)}</b>
					{
						', which is the accuracy achieved if the model always predicted the majority class.'
					}
				</ui.P>
				<ui.Card>
					<ui.NumberChart
						title="AUC ROC"
						value={ui.formatNumber(props.metrics.aucRoc)}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberChart
						title="Accuracy"
						value={ui.formatNumber(props.metrics.accuracy)}
					/>
				</ui.Card>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							title="Precision"
							value={ui.formatNumber(props.metrics.precision)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberChart
							title="Recall"
							value={ui.formatNumber(props.metrics.recall)}
						/>
					</ui.Card>
				</MetricsRow>
			</ui.S2>
		</ui.S1>
	)
}