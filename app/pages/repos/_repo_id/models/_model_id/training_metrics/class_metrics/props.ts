import { ModelLayoutInfo } from 'layouts/model_layout'

export type Props = {
	class: string
	classMetrics: {
		f1Score: number
		falseNegatives: number
		falsePositives: number
		precision: number
		recall: number
		trueNegatives: number
		truePositives: number
	}
	classes: string[]

	id: string
	modelLayoutInfo: ModelLayoutInfo
}
