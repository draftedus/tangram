import { MulticlassClassifierProps } from './props'
import { LineChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { baselineColor, trainingColor } from 'common/tokens'
import { h } from 'preact'

export function MulticlassClassifierIndexPage(
	props: MulticlassClassifierProps,
) {
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
					<ui.NumberComparisonChart
						colorA={baselineColor}
						colorB={trainingColor}
						title="Accuracy"
						valueA={props.metrics.baselineAccuracy}
						valueATitle="Baseline"
						valueB={props.metrics.accuracy}
						valueBTitle="Training"
						valueFormatter={value => ui.formatPercent(value, 2)}
					/>
				</ui.Card>
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableRow>
							<ui.TableHeaderCell>{'Class'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Precision'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Recall'}</ui.TableHeaderCell>
						</ui.TableRow>
					</ui.TableHeader>
					<ui.Tablehyper::Body>
						{props.metrics.classMetrics.map((classMetrics, i) => {
							let className = props.metrics.classes[i]
							let precision = ui.formatPercent(classMetrics.precision, 2)
							let recall = ui.formatPercent(classMetrics.recall, 2)
							return (
								<ui.TableRow key={className}>
									<ui.TableCell>{className}</ui.TableCell>
									<ui.TableCell>{precision}</ui.TableCell>
									<ui.TableCell>{recall}</ui.TableCell>
								</ui.TableRow>
							)
						})}
					</ui.Tablehyper::Body>
				</ui.Table>
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
