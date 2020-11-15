import { DateWindow, DateWindowInterval } from "common/time"
import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	columnName: string

	dateWindow: DateWindow
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
}

export type Inner =
	| {
			type: Type.Number
			value: NumberProps
	  }
	| {
			type: Type.Enum
			value: EnumProps
	  }
	| {
			type: Type.Text
			value: TextProps
	  }

export enum Type {
	Enum = "enum",
	Number = "number",
	Text = "text",
}

export type EnumProps = {
	absentCount: number
	alert: string | null
	columnName: string
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	invalidCount: number
	overallChartData: Array<
		[
			string,
			{
				productionCount: number
				productionFraction: number
				trainingCount: number
				trainingFraction: number
			},
		]
	>
	overallInvalidChartData: Array<[string, number]> | null
	rowCount: number
}

export type NumberProps = {
	absentCount: number
	alert: string | null
	columnName: string
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	intervalBoxChartData: Array<{
		label: string
		stats: {
			max: number
			min: number
			p25: number
			p50: number
			p75: number
		} | null
	}>
	invalidCount: number
	maxComparison: {
		production: number | null
		training: number
	}
	meanComparison: {
		production: number | null
		training: number
	}
	minComparison: {
		production: number | null
		training: number
	}
	overallBoxChartData: {
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
	rowCount: number
	stdComparison: {
		production: number | null
		training: number
	}
}

export type TextProps = {
	absentCount: number
	alert: string | null
	columnName: string
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	invalidCount: number
	overallTokenHistogram: Array<[string, number]>
	rowCount: number
}
