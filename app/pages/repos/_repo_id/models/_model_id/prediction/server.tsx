import './styles.css'
import {
	BarChart,
	BoxChart,
	FeatureContributionsChart,
	FeatureContributionsChartData,
} from '@tangramhq/charts'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import {
	EnumColumnToken,
	NumberColumnToken,
	TextColumnToken,
	UnknownColumnToken,
} from 'common/tokens'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { Fragment, h } from 'preact'

export type Props = {
	columns: Column[]
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	prediction: Prediction | null
}

export type Column = UnknownColumn | NumberColumn | EnumColumn | TextColumn

export enum ColumnType {
	Unknown = 'unknown',
	Number = 'number',
	Enum = 'enum',
	Text = 'text',
}

type UnknownColumn = {
	name: string
	type: ColumnType.Unknown
	value: string | null
}

type NumberColumn = {
	max: number
	min: number
	name: string
	p25: number
	p50: number
	p75: number
	type: ColumnType.Number
	value: string | null
}

type EnumColumn = {
	histogram: Array<[string, number]>
	name: string
	options: string[]
	type: ColumnType.Enum
	value: string | null
}

type TextColumn = {
	name: string
	type: ColumnType.Text
	value: string | null
}

enum PredictionType {
	Regression = 'regression',
	BinaryClassification = 'binary_classification',
	MulticlassClassification = 'multiclass_classification',
}

type Prediction =
	| {
			type: PredictionType.Regression
			value: RegressionPrediction
	  }
	| {
			type: PredictionType.BinaryClassification
			value: BinaryClassificationPrediction
	  }
	| {
			type: PredictionType.MulticlassClassification
			value: MulticlassClassificationPrediction
	  }

type RegressionPrediction = {
	featureContributionsChartData: FeatureContributionsChartData
	value: number
}

type BinaryClassificationPrediction = {
	className: string
	classes: string[]
	featureContributionsChartData: FeatureContributionsChartData
	probabilities: Array<[string, number]>
	probability: number
}

type MulticlassClassificationPrediction = {
	className: string
	classes: string[]
	featureContributionsChartData: FeatureContributionsChartData
	probabilities: Array<[string, number]>
	probability: number
}

export default function PredictPage(props: Props) {
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.Prediction}
		>
			{props.prediction === null ? (
				<PredictionForm columns={props.columns} />
			) : (
				<PredictionResult
					columns={props.columns}
					prediction={props.prediction}
				/>
			)}
		</ModelLayout>,
	)
}

type PredictionFormProps = {
	columns: Column[]
}

function PredictionForm(props: PredictionFormProps) {
	return (
		<ui.S1>
			<ui.H1>{'Prediction'}</ui.H1>
			<ui.Form>
				<div class="predict-form-items-wrapper">
					{props.columns.map(column => {
						switch (column.type) {
							case ColumnType.Unknown:
								return <UnknownField column={column} />
							case ColumnType.Number:
								return <NumberField column={column} />
							case ColumnType.Enum:
								return <EnumField column={column} />
							case ColumnType.Text:
								return <TextField column={column} />
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

type UnknownFieldProps = {
	column: Column
}

function UnknownField(props: UnknownFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<UnknownColumnToken />
				</div>
				<ui.TextField
					key={props.column.name}
					label={props.column.name}
					name={props.column.name}
					value={props.column.value}
				/>
			</div>
			<div></div>
		</>
	)
}

type NumberFieldProps = {
	column: NumberColumn
}

function NumberField(props: NumberFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<NumberColumnToken />
				</div>
				<ui.TextField
					key={props.column.name}
					label={props.column.name}
					name={props.column.name}
					value={props.column.value}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BoxChart
					class="column-chart"
					data={[
						{
							color: ui.colors.blue,
							data: [
								{
									label: props.column.name,
									x: 0,
									y: {
										max: props.column.max,
										min: props.column.min,
										p25: props.column.p25,
										p50: props.column.p50,
										p75: props.column.p75,
									},
								},
							],
							title: 'quartiles',
						},
					]}
					hideLegend={true}
					id={props.column.name}
					shouldDrawXAxisLabels={false}
				/>
			</div>
		</>
	)
}

type EnumFieldProps = {
	column: EnumColumn
}

function EnumField(props: EnumFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<EnumColumnToken />
				</div>
				<ui.SelectField
					key={props.column.name}
					label={props.column.name}
					name={props.column.name}
					options={props.column.options.map(option => ({
						text: option,
						value: option,
					}))}
					value={props.column.value ?? undefined}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BarChart
					class="column-chart"
					data={[
						{
							color: ui.colors.blue,
							data: props.column.histogram.map(([label, value], i) => ({
								label,
								x: i,
								y: value,
							})),
							title: 'histogram',
						},
					]}
					hideLegend={true}
					id={props.column.name}
					shouldDrawXAxisLabels={false}
				/>
			</div>
		</>
	)
}

type TextFieldProps = {
	column: TextColumn
}

function TextField(props: TextFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<TextColumnToken />
				</div>
				<ui.TextField
					key={props.column.name}
					label={props.column.name}
					name={props.column.name}
					value={props.column.value ?? undefined}
				/>
			</div>
			<div />
		</>
	)
}

export type PredictionResultProps = {
	columns: Column[]
	prediction: Prediction
}

function PredictionResult(props: PredictionResultProps) {
	let inner
	console.log(JSON.stringify(props, null, 2))
	switch (props.prediction.type) {
		case PredictionType.Regression:
			inner = <RegressionPredictionResult prediction={props.prediction.value} />
			break
		case PredictionType.BinaryClassification:
			inner = (
				<BinaryClassificationPredictionResult
					prediction={props.prediction.value}
				/>
			)
			break
		case PredictionType.MulticlassClassification:
			inner = (
				<MulticlassClassificationPredictionResult
					prediction={props.prediction.value}
				/>
			)
			break
	}
	return (
		<ui.S1>
			<ui.H1>{'Prediction'}</ui.H1>
			<div class="predict-output-items-wrapper">
				<ui.H2>{'Input'}</ui.H2>
				{props.columns.map(column => (
					<div key={column.name}>
						<span style="color: var(--muted-text-color)">{column.name}</span>
						{': '}
						<span style="color: var(--text-color)">{column.value}</span>
					</div>
				))}
				{inner}
			</div>
		</ui.S1>
	)
}

type RegressionPredictionResultProps = {
	prediction: RegressionPrediction
}

function RegressionPredictionResult(props: RegressionPredictionResultProps) {
	return (
		<ui.S2>
			<ui.H2>{'Output'}</ui.H2>
			<ui.Card>
				<ui.NumberChart
					title="Predicted"
					value={props.prediction.value.toString()}
				/>
			</ui.Card>
			<ui.H2>{'Explanation'}</ui.H2>
			<ui.P>
				{"This chart shows how the input values influenced the model's output."}
			</ui.P>
			<ui.Card>
				<FeatureContributionsChart
					data={props.prediction.featureContributionsChartData}
					id="regression_feature_contributions"
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

type BinaryClassificationPredictionResultProps = {
	prediction: BinaryClassificationPrediction
}

function BinaryClassificationPredictionResult(
	props: BinaryClassificationPredictionResultProps,
) {
	return (
		<ui.S2>
			<ui.H2>{'Output'}</ui.H2>
			<ui.Card>
				<ui.NumberChart title="Prediction" value={props.prediction.className} />
			</ui.Card>
			<ui.Card>
				<ui.NumberChart
					title="Probability"
					value={ui.formatPercent(props.prediction.probability, 2)}
				/>
			</ui.Card>
			<ui.H2>{'Explanation'}</ui.H2>
			<ui.P>
				{"This chart shows how the input values influenced the model's output."}
			</ui.P>
			<ui.Card>
				<FeatureContributionsChart
					data={props.prediction.featureContributionsChartData}
					id="binary_classification_feature_contributions"
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

type MulticlassClassificationPredictionResultProps = {
	prediction: MulticlassClassificationPrediction
}

function MulticlassClassificationPredictionResult(
	props: MulticlassClassificationPredictionResultProps,
) {
	let probabilityData = [
		{
			color: ui.colors.blue,
			data: props.prediction.probabilities.map(([label, probability], i) => ({
				label,
				x: i,
				y: probability,
			})),
			title: 'Probabilities',
		},
	]
	return (
		<ui.S2>
			<ui.H2>{'Output'}</ui.H2>
			<ui.Card>
				<ui.NumberChart title="Prediction" value={props.prediction.className} />
			</ui.Card>
			<ui.Card>
				<ui.NumberChart
					title="Probability"
					value={ui.formatPercent(props.prediction.probability, 2)}
				/>
			</ui.Card>
			<BarChart
				data={probabilityData}
				id="probabilities"
				title="Predicted Probabilities"
			/>
			<ui.H2>{'Explanation'}</ui.H2>
			<ui.P>
				{"This chart shows how the input values influenced the model's output."}
			</ui.P>
			<ui.Card>
				<FeatureContributionsChart
					data={props.prediction.featureContributionsChartData}
					id="multiclass_classification_feature_contributions"
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
