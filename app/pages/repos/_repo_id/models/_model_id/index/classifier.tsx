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
		accuracy: number
		baselineAccuracy: number
		classMetrics: Array<{
			precision: number
			recall: number
		}>
		classes: string[]
	}
	title: string
	trainingSummary: {
		chosenModelTypeName: string
		columnCount: number
		modelComparisonMetricTypeName: string
		rowCount: number
		testFraction: number
	}
}

export function ClassifierIndexPage(props: Props) {
	return (
		<ui.S1>
			<ui.SpaceBetween>
				<ui.H1>{'Overview'}</ui.H1>
			</ui.SpaceBetween>
			<ui.S2>
				<ui.H2>{'Training Summary'}</ui.H2>
				<ui.P>
					{'Your dataset contained '}
					<b>{props.trainingSummary.rowCount}</b>
					{' rows and '}
					<b>{props.trainingSummary.columnCount}</b>
					{' columns. '}
					<b>{ui.formatPercent(1 - props.trainingSummary.testFraction)}</b>
					{' of the rows were used in training and '}
					<b>{ui.formatPercent(props.trainingSummary.testFraction)}</b>
					{' were used in testing.  The model with the highest '}
					<b>{props.trainingSummary.modelComparisonMetricTypeName}</b>
					{' was chosen.  The best model is a '}
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
						textColorA={baselineTextColor}
						textColorB={trainingTextColor}
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
					<ui.TableBody>
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
					</ui.TableBody>
				</ui.Table>
			</ui.S2>
		</ui.S1>
	)
}
