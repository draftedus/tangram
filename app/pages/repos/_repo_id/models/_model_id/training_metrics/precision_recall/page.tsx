import * as definitions from 'common/definitions'
import { h, ui } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	classes: string[]
	modelId: string
	modelLayout: ModelLayoutProps
	nonParametricPrecisionRecallCurveData: Array<{
		precision: number
		recall: number
		threshold: number
	}>
	parametricPrecisionRecallCurveData: Array<{
		precision: number
		recall: number
	}>
	selectedClass: string
}

export default function TrainingMetricsIndexPage(props: Props) {
	let prData = ui
		.zip(
			props.parametricPrecisionRecallCurveData.map(
				threshold => threshold.recall,
			),
			props.parametricPrecisionRecallCurveData.map(
				threshold => threshold.precision,
			),
		)
		.map(([recall, precision]) => ({ x: recall, y: precision }))
		.filter(v => v.x !== null && v.y !== null)
	let ptData = ui
		.zip(
			props.nonParametricPrecisionRecallCurveData.map(
				threshold => threshold.threshold,
			),
			props.nonParametricPrecisionRecallCurveData.map(
				threshold => threshold.precision,
			),
		)
		.map(([recall, precision]) => ({ x: recall, y: precision }))
		.filter(v => v.x !== null && v.y !== null)
	let rtData = ui
		.zip(
			props.nonParametricPrecisionRecallCurveData.map(
				threshold => threshold.threshold,
			),
			props.nonParametricPrecisionRecallCurveData.map(
				threshold => threshold.recall,
			),
		)
		.map(([recall, precision]) => ({ x: recall, y: precision }))
		.filter(v => v.x !== null && v.y !== null)
	let data = [
		{
			color: ui.colors.blue,
			data: prData,
			title: 'PR',
		},
	]
	let nonParametricData = [
		{
			color: ui.colors.blue,
			data: ptData,
			title: 'Precision',
		},
		{
			color: ui.colors.green,
			data: rtData,
			title: 'Recall',
		},
	]
	return (
		<ModelLayout {...props.modelLayout}>
			<ui.S1>
				<ui.H1>Training Metrics</ui.H1>
				<ui.TabBar>
					<ui.TabLink href={`/models/${props.modelId}/training_metrics/`}>
						Overview
					</ui.TabLink>
					<ui.TabLink
						href={`/models/${props.modelId}/training_metrics/class_metrics`}
					>
						Class Metrics
					</ui.TabLink>
					<ui.TabLink
						href={`/models/${props.modelId}/training_metrics/precision_recall`}
					>
						PR Curve
					</ui.TabLink>
					<ui.TabLink href={`/models/${props.modelId}/training_metrics/roc`}>
						ROC Curve
					</ui.TabLink>
				</ui.TabBar>
				<ui.Form>
					<ui.SelectField
						label="Select Class"
						name="class"
						options={props.classes}
						value={props.selectedClass}
					/>
				</ui.Form>
				<ui.S2>
					<ui.H2>Precision Recall Curve</ui.H2>
					<ui.P>{definitions.precisionRecall}</ui.P>
					<ui.Card>
						<ui.LineChart
							data={data}
							showLegend={false}
							title="Precision Recall Curve"
							xAxisTitle="Recall"
							xMax={1}
							xMin={0}
							yAxisTitle="Precision"
							yMax={1}
							yMin={0}
						/>
					</ui.Card>
				</ui.S2>
				<ui.S2>
					<ui.H2>Non-Parametric Precision Recall Curve</ui.H2>
					<ui.P>{definitions.precisionRecall}</ui.P>
					<ui.Card>
						<ui.LineChart
							data={nonParametricData}
							showLegend={true}
							title="Precision Recall Curve"
							xAxisTitle="Threshold"
							xMax={1}
							xMin={0}
							yAxisTitle="Precision/Recall"
							yMax={1}
							yMin={0}
						/>
					</ui.Card>
				</ui.S2>
			</ui.S1>
		</ModelLayout>
	)
}
