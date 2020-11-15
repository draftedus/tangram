import { DateWindow, DateWindowInterval } from "common/time"

export type RegressorProps = {
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	mseChart: {
		data: Array<{
			label: string
			mse: number | null
		}>
		trainingMse: number
	}
	overall: {
		mse: {
			production: number | null
			training: number
		}
		rmse: {
			production: number | null
			training: number
		}
		trueValuesCount: number
	}
	true_values_count_chart: Array<{ count: number; label: string }>
}

export type BinaryClassifierProps = {
	accuracyChart: {
		data: Array<{
			accuracy: number | null
			label: string
		}>
		trainingAccuracy: number
	}
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	id: string
	overall: {
		accuracy: {
			production: number | null
			training: number
		}
		precision: {
			production: number | null
			training: number
		}
		recall: {
			production: number | null
			training: number
		}
		trueValuesCount: number
	}
	true_values_count_chart: Array<{ count: number; label: string }>
}

export type MulticlassClassifierProps = {
	accuracyChart: {
		data: Array<{
			accuracy: number | null
			label: string
		}>
		trainingAccuracy: number
	}
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	id: string
	overall: {
		accuracy: {
			production: number | null
			training: number
		}
		classMetricsTable: Array<{
			className: string
			precision: {
				production: number | null
				training: number
			}
			recall: {
				production: number | null
				training: number
			}
		}>
		trueValuesCount: number
	}
	true_values_count_chart: Array<{ count: number; label: string }>
}
