import * as ui from '@tangramhq/ui'
import { MetricsRow } from 'common/metrics_row'
import { baselineColor, selectedThresholdColor } from 'common/tokens'
import { h } from 'preact'
import { useState } from 'preact/hooks'

export type TuningProps = {
	baselineThreshold: number
	class: string
	metrics: Array<{
		accuracy: number
		f1Score: number
		falseNegatives: number
		falsePositives: number
		precision: number
		recall: number
		threshold: number
		trueNegatives: number
		truePositives: number
	}>
}

export function Tuning(props: TuningProps) {
	let thresholds = props.metrics.map(
		thresholdMetric => thresholdMetric.threshold,
	)

	let baselineIndex = thresholds.indexOf(props.baselineThreshold)

	let [selectedThresholdIndex, setSelectedThresholdIndex] = useState(
		baselineIndex,
	)
	let selectedThreshold = thresholds[selectedThresholdIndex]

	let baselineMetrics = props.metrics[baselineIndex]
	let selectedThresholdMetrics = props.metrics[selectedThresholdIndex]

	return (
		<ui.S1>
			<ui.H1>{'Tuning'}</ui.H1>
			<ui.S2>
				<ui.P>
					{
						'Drag the silder to see how metrics change with varying settings of the threshold.'
					}
				</ui.P>
				<ui.Slider
					color="var(--accent-color)"
					max={thresholds.length - 1}
					min={1}
					onChange={setSelectedThresholdIndex}
					step={1}
					value={selectedThresholdIndex}
					valueFormatter={index => ui.formatNumber(thresholds[index], 2)}
				/>
			</ui.S2>
			{selectedThreshold == 0.0 ? (
				<ui.Alert level={ui.Level.Info}>
					{
						'A threshold of 0 makes your model predict the same class for every input.'
					}
				</ui.Alert>
			) : selectedThreshold == 1.0 ? (
				<ui.Alert level={ui.Level.Info}>
					{
						'A threshold of 1 makes your model predict the same class for every input.'
					}
				</ui.Alert>
			) : null}
			<ui.S2>
				<MetricsRow>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={baselineColor}
							colorB={selectedThresholdColor}
							title="Accuracy"
							valueA={baselineMetrics.accuracy}
							valueATitle="Baseline"
							valueB={selectedThresholdMetrics.accuracy}
							valueBTitle="Selected Threshold"
							valueFormatter={value => ui.formatPercent(value, 2)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={baselineColor}
							colorB={selectedThresholdColor}
							title="F1 Score"
							valueA={
								baselineMetrics.f1Score !== null ? baselineMetrics.f1Score : NaN
							}
							valueATitle="Baseline"
							valueB={
								selectedThresholdMetrics.f1Score !== null
									? selectedThresholdMetrics.f1Score
									: NaN
							}
							valueBTitle="Selected Threshold"
							valueFormatter={value => ui.formatPercent(value, 2)}
						/>
					</ui.Card>
				</MetricsRow>
				<MetricsRow>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={baselineColor}
							colorB={selectedThresholdColor}
							title="Precision"
							valueA={
								baselineMetrics.precision !== null
									? baselineMetrics.precision
									: NaN
							}
							valueATitle="Baseline"
							valueB={
								selectedThresholdMetrics.precision !== null
									? selectedThresholdMetrics.precision
									: NaN
							}
							valueBTitle="Selected Threshold"
							valueFormatter={value => ui.formatPercent(value, 2)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA={baselineColor}
							colorB={selectedThresholdColor}
							title="Recall"
							valueA={baselineMetrics.recall}
							valueATitle="Baseline"
							valueB={selectedThresholdMetrics.recall}
							valueBTitle="Selected Threshold"
							valueFormatter={value => ui.formatPercent(value, 2)}
						/>
					</ui.Card>
				</MetricsRow>
			</ui.S2>
			<ui.S2>
				<ui.ConfusionMatrixComparison
					classLabel={props.class}
					colorA={baselineColor}
					colorB={selectedThresholdColor}
					valueA={{
						falseNegative: baselineMetrics.falseNegatives,
						falsePositive: baselineMetrics.falsePositives,
						trueNegative: baselineMetrics.trueNegatives,
						truePositive: baselineMetrics.truePositives,
					}}
					valueATitle="Baseline"
					valueATitleColor="#ccc"
					valueB={{
						falseNegative: selectedThresholdMetrics.falseNegatives,
						falsePositive: selectedThresholdMetrics.falsePositives,
						trueNegative: selectedThresholdMetrics.trueNegatives,
						truePositive: selectedThresholdMetrics.truePositives,
					}}
					valueBTitle="Selected Threshold"
					valueBTitleColor="#ccc"
				/>
			</ui.S2>
		</ui.S1>
	)
}
