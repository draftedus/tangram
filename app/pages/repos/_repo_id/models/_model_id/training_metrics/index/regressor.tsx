import { MetricsRow } from 'common/metrics_row'
import {
	baselineColor,
	baselineTextColor,
	trainingColor,
	trainingTextColor,
} from 'common/tokens'
import { h, ui } from 'deps'

export type Props = {
	baselineMse: number
	baselineRmse: number
	id: string
	mse: number
	rmse: number
}

export function RegressorTrainingMetricsIndexPage(props: Props) {
	return (
		<ui.S1>
			<ui.H1>Training Metrics</ui.H1>
			<ui.S2>
				<ui.P>
					At the end of training, your model was evaluated on a test dataset.
					All metrics in this section are from that evaluation. You can use
					these metrics to see how your model might perform on unseen production
					data.
				</ui.P>
			</ui.S2>
			<ui.S2>
				<MetricsRow>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={baselineColor}
							colorB={trainingColor}
							textColorA={baselineTextColor}
							textColorB={trainingTextColor}
							title="Root Mean Squared Error"
							valueA={props.baselineRmse}
							valueATitle="Baseline Mean Squared Error"
							valueB={props.rmse}
							valueBTitle="Root Mean Squared Error"
							valueFormatter={value => ui.formatNumber(value)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={baselineColor}
							colorB={trainingColor}
							textColorA={baselineTextColor}
							textColorB={trainingTextColor}
							title="Mean Squared Error"
							valueA={props.baselineMse}
							valueATitle="Mean Squared Error"
							valueB={props.mse}
							valueBTitle="Mean Squared Error"
							valueFormatter={value => ui.formatNumber(value)}
						/>
					</ui.Card>
				</MetricsRow>
			</ui.S2>
		</ui.S1>
	)
}
