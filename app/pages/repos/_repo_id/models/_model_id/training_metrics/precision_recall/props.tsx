import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	class: string

	modelId: string
	modelLayoutInfo: ModelLayoutInfo
	precisionRecallCurveData: Array<{
		precision: number
		recall: number
		threshold: number
	}>
}
