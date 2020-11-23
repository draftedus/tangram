import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	class: string
	classes: string[]
	f1Score: number
	falseNegatives: number
	falsePositives: number
	id: string
	modelLayoutInfo: ModelLayoutInfo
	precision: number
	recall: number
	trueNegatives: number
	truePositives: number
}
