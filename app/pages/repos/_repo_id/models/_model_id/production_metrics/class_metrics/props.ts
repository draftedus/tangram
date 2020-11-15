import { DateWindow, DateWindowInterval } from "common/time"
import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	class: string
	classMetrics: Array<{
		className: string
		intervals: Array<{
			f1Score: {
				production: number | null
				training: number
			}
			label: string
			precision: {
				production: number | null
				training: number
			}
			recall: {
				production: number | null
				training: number
			}
		}>
	}>
	classes: string[]

	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	id: string
	modelLayoutInfo: ModelLayoutInfo
	overall: {
		classMetrics: OverallClassMetrics[]
		label: string
	}

	title: string
}

export type OverallClassMetrics = {
	className: string
	comparison: {
		falseNegativeFraction: {
			production: number | null
			training: number
		}
		falsePositiveFraction: {
			production: number | null
			training: number
		}
		trueNegativeFraction: {
			production: number | null
			training: number
		}
		truePositiveFraction: {
			production: number | null
			training: number
		}
	}
	confusionMatrix: {
		falseNegatives: number | null
		falsePositives: number | null
		trueNegatives: number | null
		truePositives: number | null
	}
	f1Score: {
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
}
