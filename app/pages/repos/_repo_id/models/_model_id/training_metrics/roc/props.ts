import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Props = {
	aucRoc: number
	class: string
	modelId: string
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	rocCurveData: Array<{
		falsePositiveRate: number
		truePositiveRate: number
	}>
	title: string
}
