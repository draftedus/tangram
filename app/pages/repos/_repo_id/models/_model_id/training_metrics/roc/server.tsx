import { LineChart, LineStyle, PointStyle } from '@tangramhq/charts'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { ClassSelectField } from 'common/class_select_field'
import * as definitions from 'common/definitions'
import { renderPage } from 'common/render'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { h } from 'preact'

export type Props = {
	aucRoc: number[]
	class: string
	classes: string[]
	modelId: string
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	rocCurveData: Array<
		Array<{
			falsePositiveRate: number
			truePositiveRate: number
		}>
	>
	title: string
}

export default function TrainingMetricsIndexPage(props: Props) {
	let selectedClassIndex = props.classes.indexOf(props.class)
	let rocData = props.rocCurveData[selectedClassIndex].map(
		({ falsePositiveRate, truePositiveRate }) => ({
			x: falsePositiveRate,
			y: truePositiveRate,
		}),
	)
	let aucRoc = props.aucRoc[selectedClassIndex]
	let data = [
		{
			color: ui.colors.blue,
			data: rocData,
			title: 'ROC',
		},
		{
			color: ui.colors.gray,
			data: [
				{ x: 0, y: 0 },
				{ x: 1, y: 1 },
			],
			lineStyle: LineStyle.Dashed,
			pointStyle: PointStyle.Hidden,
			title: 'Reference',
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
					<ui.TabLink href="precision_recall">{'PR Curve'}</ui.TabLink>
					<ui.TabLink href="roc" selected={true}>
						{'ROC Curve'}
					</ui.TabLink>
				</ui.TabBar>
				<ui.Form>
					<ClassSelectField class={props.class} classes={props.classes} />
					<noscript>
						<ui.Button>{'Submit'}</ui.Button>
					</noscript>
				</ui.Form>
				<ui.S2>
					<ui.H2>{'Area Under the Receiver Operating Characteristic'}</ui.H2>
					<ui.P>{definitions.aucRoc}</ui.P>
					<ui.Card>
						<ui.NumberChart
							key="auc"
							title="AUC"
							value={ui.formatNumber(aucRoc)}
						/>
					</ui.Card>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Receiver Operating Characteristic Curve'}</ui.H2>
					<ui.P>{definitions.receiverOperatingCharacteristic}</ui.P>
					<ui.Card>
						<LineChart
							data={data}
							hideLegend={false}
							id="roc"
							title="Receiver Operating Characteristic Curve"
							xAxisTitle="False Positive Rate"
							xMax={1}
							xMin={0}
							yAxisTitle="True Positive Rate"
							yMax={1}
							yMin={0}
						/>
					</ui.Card>
				</ui.S2>
			</ui.S1>
		</ModelLayout>,
	)
}
