import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	aucRoc: number
	class: string

	modelId: string
	modelLayoutInfo: ModelLayoutInfo

	rocCurveData: Array<{
		falsePositiveRate: number
		truePositiveRate: number
	}>
	title: string
}
