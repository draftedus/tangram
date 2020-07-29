import * as definitions from 'common/definitions'
import { MetricsRow } from 'common/metrics_row'
import { h, ui } from 'deps'

export type Props = {
	class: string
	classMetrics: Array<{
		f1Score: number
		falseNegatives: number
		falsePositives: number
		precision: number
		recall: number
		trueNegatives: number
		truePositives: number
	}>
	classes: string[]
	id: string
	selectedClass: string
}

export function MulticlassClassifierClassMetricsPage(props: Props) {
	let selectedClassIndex = props.classes.indexOf(props.selectedClass)
	let selectedClassTaskMetrics = props.classMetrics[selectedClassIndex]
	return (
		<ui.S1>
			<ui.H1>Training Metrics</ui.H1>
			<ui.TabBar>
				<ui.TabLink href="/models/${props.id}/training_metrics/">
					Overview
				</ui.TabLink>
				<ui.TabLink href="/models/${props.id}/training_metrics/class_metrics">
					Class Metrics
				</ui.TabLink>
			</ui.TabBar>
			<ui.SelectField options={props.classes} />
			<ui.S2>
				<ui.H2>Precision and Recall</ui.H2>
				<ui.P>{definitions.precisionRecall}</ui.P>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							key="precision"
							title="Precision"
							value={ui.formatPercent(selectedClassTaskMetrics.precision, 2)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberChart
							key="recall"
							title="Recall"
							value={ui.formatPercent(selectedClassTaskMetrics.recall, 2)}
						/>
					</ui.Card>
				</MetricsRow>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							key="f1Score"
							title="F1 Score"
							value={ui.formatPercent(selectedClassTaskMetrics.f1Score, 2)}
						/>
					</ui.Card>
				</MetricsRow>
			</ui.S2>
			<ui.S2>
				<ui.H2>Confusion Matrix</ui.H2>
				<ui.P>{definitions.confusionMatrix}</ui.P>
				<ui.ConfusionMatrix
					classLabel={props.selectedClass}
					falseNegatives={selectedClassTaskMetrics.falseNegatives}
					falsePositives={selectedClassTaskMetrics.falsePositives}
					trueNegatives={selectedClassTaskMetrics.trueNegatives}
					truePositives={selectedClassTaskMetrics.truePositives}
				/>
			</ui.S2>
		</ui.S1>
	)
}
