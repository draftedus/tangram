import {
	EnumColumnToken,
	NumberColumnToken,
	TextColumnToken,
	UnknownColumnToken,
} from './tokens'
import {
	BarChart,
	FeatureContributionsChart,
	FeatureContributionsChartData,
} from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export enum PredictionType {
	Regression = 'regression',
	BinaryClassification = 'binary_classification',
	MulticlassClassification = 'multiclass_classification',
}

export type Prediction =
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

export type RegressionPrediction = {
	featureContributionsChartData: FeatureContributionsChartData
	value: number
}

export type BinaryClassificationPrediction = {
	className: string
	classes: string[]
	featureContributionsChartData: FeatureContributionsChartData
	probabilities: Array<[string, number]>
	probability: number
}

export type MulticlassClassificationPrediction = {
	className: string
	classes: string[]
	featureContributionsChartData: FeatureContributionsChartData
	probabilities: Array<[string, number]>
	probability: number
}

export type PredictionResultProps = {
	inputTable: InputTable
	prediction: Prediction
}

export type InputTable = {
	rows: Array<{ columnName: string; columnType: ColumnType; value: string }>
}

export enum ColumnType {
	Unknown = 'unknown',
	Number = 'number',
	Text = 'text',
	Enum = 'enum',
}

export function PredictionResult(props: PredictionResultProps) {
	let inner
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
		<div class="predict-output-items-wrapper">
			<ui.H2>{'Input'}</ui.H2>
			<ui.Table width="100%">
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{'Column Name'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Column Type'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Value'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{props.inputTable.rows.map(inputTableRow => (
						<ui.TableRow key={inputTableRow.columnName}>
							<ui.TableCell>{inputTableRow.columnName}</ui.TableCell>
							<ui.TableCell>
								{inputTableRow.columnType === ColumnType.Number ? (
									<NumberColumnToken />
								) : inputTableRow.columnType === ColumnType.Enum ? (
									<EnumColumnToken />
								) : inputTableRow.columnType === ColumnType.Text ? (
									<TextColumnToken />
								) : inputTableRow.columnType === ColumnType.Unknown ? (
									<UnknownColumnToken />
								) : null}
							</ui.TableCell>
							<ui.TableCell>{inputTableRow.value}</ui.TableCell>
						</ui.TableRow>
					))}
				</ui.TableBody>
			</ui.Table>
			{inner}
		</div>
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
					title="Prediction"
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
