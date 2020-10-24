import { PinwheelInfo } from '@tangramhq/pinwheel'
import { InputTable, Prediction } from 'common/prediction_result'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Props = {
	date: String
	identifier: String
	inputTable: InputTable
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	prediction: Prediction
}
