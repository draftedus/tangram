import { ClassSelect } from 'common/class_select'
import * as definitions from 'common/definitions'
import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	aucRoc: number
	class: string
	classes: string[]
	modelId: string
	modelLayoutProps: ModelLayoutProps
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
			lineStyle: ui.LineStyle.Dashed,
			pointStyle: ui.PointStyle.Hidden,
			title: 'Reference',
		},
	]
	return renderPage(
		<ModelLayout {...props.modelLayoutProps} pinwheelInfo={props.pinwheelInfo}>
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
				<ClassSelect class={props.class} classes={props.classes} />
				<ui.S2>
					<ui.H2>{'Area Under the Receiver Operating Characteristic'}</ui.H2>
					<ui.P>{definitions.aucRoc}</ui.P>
					<ui.Card>
						<ui.NumberChart
							key="auc"
							title="AUC"
							value={ui.formatNumber(props.aucRoc)}
						/>
					</ui.Card>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Receiver Operating Characteristic Curve'}</ui.H2>
					<ui.P>{definitions.receiverOperatingCharacteristic}</ui.P>
					<ui.Card>
						<ui.LineChart
							data={data}
							showLegend={false}
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
