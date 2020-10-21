import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Props = {
	class: string
	modelId: string
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	precisionRecallCurveData: Array<{
		precision: number
		recall: number
		threshold: number
	}>
}
