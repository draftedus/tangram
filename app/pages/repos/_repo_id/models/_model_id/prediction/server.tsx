import { PinwheelInfo, h, renderPage, ui } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'

export type Props = {
	columns: Column[]
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	prediction: Prediction | null
}

export type Column =
	| {
			name: string
			type: ColumnType.Unknown
	  }
	| {
			max: number
			min: number
			name: string
			type: ColumnType.Number
	  }
	| {
			name: string
			options: string[]
			type: ColumnType.Enum
	  }
	| {
			name: string
			type: ColumnType.Text
	  }

export enum ColumnType {
	Unknown = 'unknown',
	Number = 'number',
	Enum = 'enum',
	Text = 'text',
}

enum PredictionType {
	Regression = 'regression',
	Classification = 'classification',
}

type Prediction =
	| {
			type: PredictionType.Regression
			value: RegressionPrediction
	  }
	| {
			type: PredictionType.Classification
			value: ClassificationPrediction
	  }
	| null

type RegressionPrediction = {
	shapChartData: Array<{
		baseline: number
		baselineLabel: string
		label: string
		output: number
		outputLabel: string
		values: Array<{
			feature: string
			value: number
		}>
	}>
	value: number
}

type ClassificationPrediction = {
	chartData: Array<{ x: number; y: number }>
	className: string
	classes: string[]
	probability: number
	shapChartData: Array<{
		baseline: number
		baselineLabel: string
		label: string
		output: number
		outputLabel: string
		values: Array<{
			feature: string
			value: number
		}>
	}>
	xAxisLabelFormatter: (i: number) => string
}

export default function PredictPage(props: Props) {
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.Prediction}
		>
			<ui.S1>
				<ui.H1>{'Predict'}</ui.H1>
				<ui.Form>
					{props.columns.map(column => {
						let name = column.name
						switch (column.type) {
							case ColumnType.Unknown:
								return <ui.TextField key={name} label={column.name} />
							case ColumnType.Number:
								return <ui.TextField key={name} label={column.name} />
							case ColumnType.Enum:
								return (
									<ui.SelectField
										key={name}
										label={column.name}
										options={column.options}
									/>
								)
							case ColumnType.Text:
								return <ui.TextField key={name} label={column.name} />
						}
					})}
					<input name="action" type="hidden" value="predict" />
					<ui.Button type="submit">{'Predict'}</ui.Button>
				</ui.Form>
				{props.prediction &&
				props.prediction.type === PredictionType.Classification ? (
					<ui.S2>
						<ui.H2>{'Prediction Output'}</ui.H2>
						<ui.Card>
							<ui.NumberChart
								title="Predicted Class"
								value={props.prediction.value.className}
							/>
						</ui.Card>
						<ui.Card>
							<ui.NumberChart
								title="Probability"
								value={ui.formatPercent(props.prediction.value.probability)}
							/>
						</ui.Card>
						{props.prediction.value.classes.length > 2 && (
							<ui.BarChart
								data={[
									{
										color: ui.colors.blue,
										data: props.prediction.value.chartData,
										title: 'probabilities',
									},
								]}
								id="probabilities"
								title="Predicted Probabilities"
								xAxisLabelFormatter={props.prediction.value.xAxisLabelFormatter}
							/>
						)}
						<ui.H2>{'Explanation'}</ui.H2>
						<ui.P>
							{
								"The baseline value is the mean value observed in the training dataset. Each feature in the example influences the model's output, either positively or negatively."
							}
						</ui.P>
						<ui.Card>
							<ui.ShapChart
								data={props.prediction.value.shapChartData}
								id="classification_shap"
								includeXAxisTitle={true}
								includeYAxisLabels={true}
								includeYAxisTitle={true}
								negativeColor={ui.colors.red}
								positiveColor={ui.colors.green}
							/>
						</ui.Card>
					</ui.S2>
				) : props.prediction &&
				  props.prediction.type === PredictionType.Regression ? (
					<ui.S2>
						<ui.H2>{'Prediction Output'}</ui.H2>
						<ui.Card>
							<ui.NumberChart
								title="Predicted"
								value={props.prediction.value.value.toString()}
							/>
						</ui.Card>
						<ui.H2>{'Explanation'}</ui.H2>
						<ui.P>
							{
								"Each class has a baseline likelihood of being predicted equal to that class's proportion in the training dataset. The learned model adjust its output based on what features it sees in the example. The chart below shows which features were most influential in making the model's decision."
							}
						</ui.P>
						<ui.Card>
							<ui.ShapChart
								data={props.prediction.value.shapChartData}
								id="regression_shap"
								includeXAxisTitle={true}
								includeYAxisLabels={false}
								includeYAxisTitle={false}
								negativeColor={ui.colors.red}
								positiveColor={ui.colors.green}
							/>
						</ui.Card>
					</ui.S2>
				) : null}
			</ui.S1>
		</ModelLayout>,
	)
}
