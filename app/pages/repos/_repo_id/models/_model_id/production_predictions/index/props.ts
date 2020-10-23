import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Props = {
	modelLayoutInfo: ModelLayoutInfo
	pagination: {
		after: string | null
		before: string | null
	}
	pinwheelInfo: PinwheelInfo
	predictionTable: { rows: PredictionTableRow[] }
}

type PredictionTableRow = {
	date: string
	identifier: string
	output: string
}
