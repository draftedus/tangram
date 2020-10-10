import { LineChart } from '@tangramhq/charts'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { ClassSelectField } from 'common/class_select'
import * as definitions from 'common/definitions'
import { renderPage } from 'common/render'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { h } from 'preact'

export type Props = {
	class: string
	classes: string[]
	data: Array<{
		precision: number
		recall: number
		threshold: number
	}>
	modelId: string
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
}

export default function TrainingMetricsIndexPage(props: Props) {
	let prData = ui
		.zip(
			props.data.map(threshold => threshold.recall),
			props.data.map(threshold => threshold.precision),
		)
		.map(([recall, precision]) => ({ x: recall, y: precision }))
		.filter(v => v.x !== null && v.y !== null)
	let precisionData = ui
		.zip(
			props.data.map(threshold => threshold.threshold),
			props.data.map(threshold => threshold.precision),
		)
		.map(([threshold, precision]) => ({ x: threshold, y: precision }))
		.filter(v => v.x !== null && v.y !== null)
	let recallData = ui
		.zip(
			props.data.map(threshold => threshold.threshold),
			props.data.map(threshold => threshold.recall),
		)
		.map(([threshold, recall]) => ({ x: threshold, y: recall }))
		.filter(v => v.x !== null && v.y !== null)
	let parametricData = [
		{
			color: ui.colors.blue,
			data: prData,
			title: 'PR',
		},
	]
	let nonParametricData = [
		{
			color: ui.colors.blue,
			data: precisionData,
			title: 'Precision',
		},
		{
			color: ui.colors.green,
			data: recallData,
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
				<ui.Form>
					<ClassSelectField class={props.class} classes={props.classes} />
					<noscript>
						<ui.Button>{'Submit'}</ui.Button>
					</noscript>
				</ui.Form>
				<ui.S2>
					<ui.H2>{'Parametric Precision Recall Curve'}</ui.H2>
					<ui.P>{definitions.precisionRecall}</ui.P>
					<ui.Card>
						<LineChart
							data={parametricData}
							hideLegend={true}
							id="parametric_pr"
							title="Parametric Precision Recall Curve"
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
						<LineChart
							data={nonParametricData}
							hideLegend={false}
							id="non_parametric_pr"
							title="Non-Parametric Precision Recall Curve"
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
