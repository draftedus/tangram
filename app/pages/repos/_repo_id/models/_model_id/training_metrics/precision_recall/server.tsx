import { ClassSelect } from 'common/class_select'
import * as definitions from 'common/definitions'
import { PinwheelInfo, h, renderPage, ui } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'

export type Props = {
	class: string
	classes: string[]
	modelId: string
	modelLayoutInfo: ModelLayoutInfo
	nonParametricPrecisionRecallCurveData: Array<{
		precision: number
		recall: number
		threshold: number
	}>
	parametricPrecisionRecallCurveData: Array<{
		precision: number
		recall: number
	}>
	pinwheelInfo: PinwheelInfo
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
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.TrainingMetrics}
		>
			<ui.S1>
				<ui.H1>{'Training Metrics'}</ui.H1>
				<ui.TabBar>
					<ui.TabLink href="./">{'Overview'}</ui.TabLink>
					<ui.TabLink href="class_metrics">{'Class Metrics'}</ui.TabLink>
					<ui.TabLink href="precision_recall" selected={true}>
						{'PR Curve'}
					</ui.TabLink>
					<ui.TabLink href="roc">{'ROC Curve'}</ui.TabLink>
				</ui.TabBar>
				<ClassSelect class={props.class} classes={props.classes} />
				<ui.S2>
					<ui.H2>{'Precision Recall Curve'}</ui.H2>
					<ui.P>{definitions.precisionRecall}</ui.P>
					<ui.Card>
						<ui.LineChart
							data={data}
							id="parametric_pr"
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
					<ui.H2>{'Non-Parametric Precision Recall Curve'}</ui.H2>
					<ui.P>{definitions.precisionRecall}</ui.P>
					<ui.Card>
						<ui.LineChart
							data={nonParametricData}
							id="non_parametric_pr"
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
		</ModelLayout>,
	)
}
