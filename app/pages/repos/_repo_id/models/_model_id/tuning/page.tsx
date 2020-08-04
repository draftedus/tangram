import { MetricsRow } from 'common/metrics_row'
import {
	baselineColor,
	baselineTextColor,
	selectedThresholdColor,
	selectedThresholdTextColor,
} from 'common/tokens'
import { Client, h, r, ui, useState } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	inner: InnerProps | null
	modelLayoutProps: ModelLayoutProps
}

export type InnerProps = {
	baselineThreshold: string
	classes: string[]
	metrics: Array<
		Array<{
			accuracy: number
			f1Score: number
			falseNegatives: number
			falsePositives: number
			precision: number
			recall: number
			threshold: string
			trueNegatives: number
			truePositives: number
		}>
	>
}

export default function TuningPage(props: Props) {
	let inner
	if (props.inner) {
		inner = (
			<Client component={TuningPageInner} id="tuning" props={props.inner} />
		)
	} else {
		inner = (
			<ui.S1>
				<ui.P>{'Tuning is not supported for this model.'}</ui.P>
			</ui.S1>
		)
	}
	return <ModelLayout {...props.modelLayoutProps}>{inner}</ModelLayout>
}

export function TuningPageInner(props: InnerProps) {
	let [selectedClass, setSelectedClass] = useState(props.classes[1])
	let selectedClassIndex = props.classes.indexOf(selectedClass)

	let selectedClassThresholdMetrics = props.metrics[selectedClassIndex]

	let thresholds = selectedClassThresholdMetrics.map(
		thresholdMetric => thresholdMetric.threshold,
	)

	let baselineIndex = thresholds.indexOf(props.baselineThreshold)

	let [selectedThresholdIndex, setSelectedThresholdIndex] = useState(
		baselineIndex,
	)
	let selectedThreshold = thresholds[selectedThresholdIndex]

	let baselineMetrics = selectedClassThresholdMetrics[baselineIndex]
	let selectedThresholdMetrics =
		selectedClassThresholdMetrics[selectedThresholdIndex]

	let onChange = (value: string | null) => {
		setSelectedClass(r(value))
	}

	return (
		<ui.S1>
			<ui.H1>{'Tuning'}</ui.H1>
			<ui.S2>
				<ui.P>
					{
						'Drag the silder to see how metrics change with varying settings of the'
					}
					{'threshold.'}
				</ui.P>
				<ui.Slider
					color={ui.variables.colors.accent}
					max={thresholds.length - 1}
					min={1}
					onChange={setSelectedThresholdIndex}
					step={1}
					value={selectedThresholdIndex}
					valueFormatter={index => thresholds[index]}
				/>
			</ui.S2>
			<ui.S2>
				<ui.SelectField
					label="Select Class"
					onChange={onChange}
					options={props.classes}
					value={selectedClass}
				/>
			</ui.S2>
			{selectedThreshold == '0.0' ? (
				<ui.Alert level={ui.Level.Info}>
					{
						'A selected threshold of 0 makes your model predict the same class for every input.'
					}
				</ui.Alert>
			) : selectedThreshold == '1.0' ? (
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
							textColorA={baselineTextColor}
							textColorB={selectedThresholdTextColor}
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
							textColorA={baselineTextColor}
							textColorB={selectedThresholdTextColor}
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
							textColorA={baselineTextColor}
							textColorB={selectedThresholdTextColor}
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
							textColorA={baselineTextColor}
							textColorB={selectedThresholdTextColor}
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
					classLabel={props.classes[selectedClassIndex]}
					colorA={baselineColor}
					colorB={selectedThresholdColor}
					textColorA={baselineTextColor}
					textColorB={selectedThresholdTextColor}
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
