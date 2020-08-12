import { PinwheelInfo, h, renderPage, ui } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { Fragment } from 'preact'

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
			value: string | null
	  }
	| {
			max: number
			min: number
			name: string
			p25: number
			p50: number
			p75: number
			type: ColumnType.Number
			value: string | null
	  }
	| {
			histogram: Array<[string, number]>
			name: string
			options: string[]
			type: ColumnType.Enum
			value: string | null
	  }
	| {
			name: string
			type: ColumnType.Text
			value: string | null
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
	className: string
	classes: string[]
	probabilities: Array<[string, number]>
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
}

export default function PredictPage(props: Props) {
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.Prediction}
		>
			{props.prediction ? (
				<PredictionOutputInner {...props} />
			) : (
				<PredictionInputInner {...props} />
			)}
		</ModelLayout>,
	)
}

function PredictionInputInner(props: Props) {
	return (
		<ui.S1>
			<ui.H1>{'Prediction'}</ui.H1>
			<ui.Form autoComplete="off">
				<div class="predict-form-items-wrapper">
					{props.columns.map(column => {
						let name = column.name
						switch (column.type) {
							case ColumnType.Unknown:
								return (
									<Fragment>
										<ui.TextField
											key={name}
											label={column.name}
											name={name}
											value={column.value}
										/>
										<div />
									</Fragment>
								)
							case ColumnType.Number:
								return (
									<Fragment>
										<ui.TextField
											key={name}
											label={column.name}
											name={name}
											value={column.value}
										/>
										<div>
											<ui.BoxChart
												class="column-chart"
												data={[
													{
														color: ui.colors.blue,
														data: [
															{
																x: 0,
																y: {
																	max: column.max,
																	min: column.min,
																	p25: column.p25,
																	p50: column.p50,
																	p75: column.p75,
																},
															},
														],
														title: 'quartiles',
													},
												]}
												hideLegend={true}
												id={column.name}
											/>
										</div>
									</Fragment>
								)
							case ColumnType.Enum:
								return (
									<Fragment>
										<ui.SelectField
											key={name}
											label={column.name}
											name={name}
											options={column.options}
											value={column.value ?? undefined}
										/>
										<div>
											<ui.BarChart
												class="column-chart"
												data={[
													{
														color: ui.colors.blue,
														data: column.histogram.map(([_, value], i) => ({
															x: i,
															y: value,
														})),
														title: 'histogram',
													},
												]}
												hideLegend={true}
												id={column.name}
											/>
										</div>
									</Fragment>
								)
							case ColumnType.Text:
								return (
									<Fragment>
										<ui.TextField
											key={name}
											label={column.name}
											name={name}
											value={column.value ?? undefined}
										/>
										<div />
									</Fragment>
								)
						}
					})}
				</div>
				<div class="predict-form-buttons-wrapper">
					<ui.Button type="submit">{'Predict'}</ui.Button>
					<ui.Button color={ui.colors.yellow} type="reset">
						{'Reset Defaults'}
					</ui.Button>
				</div>
			</ui.Form>
		</ui.S1>
	)
}

function PredictionOutputInner(props: Props) {
	return (
		<ui.S1>
			<ui.H1>{'Prediction'}</ui.H1>
			<div class="predict-output-items-wrapper">
				{props.columns.map(column => (
					<div key={column.name}>
						<span style="color: var(--muted-text-color)">{column.name}</span>
						{': '}
						<span style="color: var(--text-color)">{column.value}</span>
					</div>
				))}
			</div>
			{props.prediction &&
				(props.prediction.type === PredictionType.Classification ? (
					<ClassificationPredictionOutput {...props.prediction.value} />
				) : props.prediction.type === PredictionType.Regression ? (
					<RegressionPredictionOutput {...props.prediction.value} />
				) : null)}
		</ui.S1>
	)
}

type RegressionPredictionOutputProps = RegressionPrediction

function RegressionPredictionOutput(props: RegressionPredictionOutputProps) {
	return (
		<ui.S2>
			<ui.H2>{'Prediction Output'}</ui.H2>
			<ui.Card>
				<ui.NumberChart title="Predicted" value={props.value.toString()} />
			</ui.Card>
			<ui.H2>{'Explanation'}</ui.H2>
			<ui.P>
				{
					"Each class has a baseline likelihood of being predicted equal to that class's proportion in the training dataset. The learned model adjust its output based on what features it sees in the example. The chart below shows which features were most influential in making the model's decision."
				}
			</ui.P>
			<ui.Card>
				<ui.ShapChart
					data={props.shapChartData}
					id="regression_shap"
					includeXAxisTitle={true}
					includeYAxisLabels={false}
					includeYAxisTitle={false}
					negativeColor={ui.colors.red}
					positiveColor={ui.colors.green}
				/>
			</ui.Card>
		</ui.S2>
	)
}

type ClassificationPredictionOutputProps = ClassificationPrediction

function ClassificationPredictionOutput(
	props: ClassificationPredictionOutputProps,
) {
	let probabilityData = [
		{
			color: ui.colors.blue,
			data: props.probabilities.map(([_className, probability], i) => ({
				x: i,
				y: probability,
			})),
			title: 'Probabilities',
		},
	]
	return (
		<ui.S2>
			<ui.H2>{'Prediction Output'}</ui.H2>
			<ui.Card>
				<ui.NumberChart title="Predicted Class" value={props.className} />
			</ui.Card>
			<ui.Card>
				<ui.NumberChart
					title="Probability"
					value={ui.formatPercent(props.probability, 2)}
				/>
			</ui.Card>
			{props.probabilities.length > 2 && (
				<ui.BarChart
					data={probabilityData}
					id="probabilities"
					title="Predicted Probabilities"
					xAxisLabelFormatter={i => props.probabilities[i][0]}
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
					data={props.shapChartData}
					id="classification_shap"
					includeXAxisTitle={true}
					includeYAxisLabels={true}
					includeYAxisTitle={true}
					negativeColor={ui.colors.red}
					positiveColor={ui.colors.green}
				/>
			</ui.Card>
		</ui.S2>
	)
}
