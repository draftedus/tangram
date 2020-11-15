import { DateWindow, DateWindowInterval } from "common/time"
import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	modelId: string
	modelLayoutInfo: ModelLayoutInfo
	overallColumnStatsTable: Array<{
		absentCount: number
		alert: string | null
		columnType: ColumnType
		invalidCount: number
		name: string
	}>
	predictionCountChart: Array<{
		count: number
		label: string
	}>
	predictionStatsChart: PredictionStatsChart
	predictionStatsIntervalChart: PredictionStatsIntervalChart
}

export type PredictionStatsChart =
	| {
			data: RegressionChartEntry
			type: Task.Regression
	  }
	| {
			data: BinaryClassificationChartEntry
			type: Task.BinaryClassification
	  }
	| {
			data: MulticlassClassificationChartEntry
			type: Task.MulticlassClassification
	  }

export type PredictionStatsIntervalChart =
	| {
			data: RegressionChartEntry[]
			type: Task.Regression
	  }
	| {
			data: BinaryClassificationChartEntry[]
			type: Task.BinaryClassification
	  }
	| {
			data: MulticlassClassificationChartEntry[]
			type: Task.MulticlassClassification
	  }

export enum Task {
	Regression = "regression",
	BinaryClassification = "binary_classification",
	MulticlassClassification = "multiclass_classification",
}

export type RegressionChartEntry = {
	label: string
	quantiles: {
		production: {
			max: number
			min: number
			p25: number
			p50: number
			p75: number
		} | null
		training: {
			max: number
			min: number
			p25: number
			p50: number
			p75: number
		}
	}
}

export type BinaryClassificationChartEntry = {
	histogram: {
		production: Array<[string, number]>
		training: Array<[string, number]>
	}
	label: string
}

export type MulticlassClassificationChartEntry = {
	histogram: {
		production: Array<[string, number]>
		training: Array<[string, number]>
	}
	label: string
}

export enum ColumnType {
	Number = "number",
	Enum = "enum",
	Text = "text",
}
