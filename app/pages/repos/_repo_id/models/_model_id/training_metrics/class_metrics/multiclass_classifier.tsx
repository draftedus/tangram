import { ClassSelect } from './page'
import * as definitions from 'common/definitions'
import { MetricsRow } from 'common/metrics_row'
import { h, ui } from 'deps'

export type Props = {
	class: string
	classMetrics: {
		f1Score: number
		falseNegatives: number
		falsePositives: number
		precision: number
		recall: number
		trueNegatives: number
		truePositives: number
	}
	classes: string[]
	id: string
}

export function MulticlassClassifierClassMetricsPage(props: Props) {
	return (
		<ui.S1>
			<ui.H1>Training Metrics</ui.H1>
			<ui.TabBar>
				<ui.TabLink href="./">Overview</ui.TabLink>
				<ui.TabLink href="./class_metrics">Class Metrics</ui.TabLink>
			</ui.TabBar>
			<ClassSelect class={props.class} classes={props.classes} />
			<ui.S2>
				<ui.H2>Precision and Recall</ui.H2>
				<ui.P>{definitions.precisionRecall}</ui.P>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							key="precision"
							title="Precision"
							value={ui.formatPercent(props.classMetrics.precision, 2)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberChart
							key="recall"
							title="Recall"
							value={ui.formatPercent(props.classMetrics.recall, 2)}
						/>
					</ui.Card>
				</MetricsRow>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							key="f1Score"
							title="F1 Score"
							value={ui.formatPercent(props.classMetrics.f1Score, 2)}
						/>
					</ui.Card>
				</MetricsRow>
			</ui.S2>
			<ui.S2>
				<ui.H2>Confusion Matrix</ui.H2>
				<ui.P>{definitions.confusionMatrix}</ui.P>
				<ui.ConfusionMatrix
					classLabel={props.class}
					falseNegatives={props.classMetrics.falseNegatives}
					falsePositives={props.classMetrics.falsePositives}
					trueNegatives={props.classMetrics.trueNegatives}
					truePositives={props.classMetrics.truePositives}
				/>
			</ui.S2>
		</ui.S1>
	)
}
