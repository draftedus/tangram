import * as ui from '@tangramhq/ui'
import * as definitions from 'common/definitions'
import { MetricsRow } from 'common/metrics_row'
import { baselineColor, trainingColor } from 'common/tokens'
import { h } from 'preact'

export type Props = {
	accuracy: number
	aucRoc: number
	baselineAccuracy: number
	classes: string[]
	id: string
	losses: number[] | null
	precision: number
	recall: number
}

export function BinaryClassifierTrainingMetricsIndexPage(props: Props) {
	let loss = props.losses?.slice(-1).pop()
	return (
		<ui.S1>
			<ui.H1>{'Training Metrics'}</ui.H1>
			<ui.TabBar>
				<ui.TabLink href="./" selected={true}>
					{'Overview'}
				</ui.TabLink>
				<ui.TabLink href="precision_recall">{'PR Curve'}</ui.TabLink>
				<ui.TabLink href="roc">{'ROC Curve'}</ui.TabLink>
			</ui.TabBar>
			<ui.S2>
				<ui.P>
					{
						'At the end of training, your model was evaluated on a test dataset. All metrics in this section are from that evaluation. You can use these metrics to see how your model might perform on unseen production data.'
					}
				</ui.P>
			</ui.S2>
			<ui.S2>
				<ui.H2>{'Accuracy'}</ui.H2>
				<ui.P>{definitions.accuracy}</ui.P>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={baselineColor}
						colorB={trainingColor}
						title="Accuracy"
						valueA={props.baselineAccuracy}
						valueATitle="Baseline"
						valueB={props.accuracy}
						valueBTitle="Training"
						valueFormatter={value => ui.formatPercent(value, 2)}
					/>
				</ui.Card>
			</ui.S2>
			<ui.Card>
				<ui.NumberChart title="AUC ROC" value={ui.formatNumber(props.aucRoc)} />
			</ui.Card>
			<MetricsRow>
				<ui.Card>
					<ui.NumberChart
						title="Precision"
						value={ui.formatNumber(props.precision)}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberChart
						title="Recall"
						value={ui.formatNumber(props.recall)}
					/>
				</ui.Card>
			</MetricsRow>
			{loss !== undefined && (
				<ui.S2>
					<ui.H2>{'Loss'}</ui.H2>
					<ui.P>{definitions.logLoss}</ui.P>
					<ui.Card>
						<ui.NumberChart title="Log Loss" value={ui.formatNumber(loss)} />
					</ui.Card>
				</ui.S2>
			)}
		</ui.S1>
	)
}
