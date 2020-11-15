import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	modelLayoutInfo: ModelLayoutInfo

	tuning: TuningProps | null
}

export type TuningProps = {
	baselineThreshold: number
	class: string
	metrics: Array<{
		accuracy: number
		f1Score: number
		falseNegatives: number
		falsePositives: number
		precision: number
		recall: number
		threshold: number
		trueNegatives: number
		truePositives: number
	}>
}
