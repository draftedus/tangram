import { InputTable, Prediction } from "common/prediction_result"
import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	identifier: String
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
}

export enum InnerType {
	Found = "found",
	NotFound = "not_found",
}

export type Inner =
	| {
			type: InnerType.Found
			value: Found
	  }
	| {
			type: InnerType.NotFound
			value: null
	  }

export type Found = {
	date: String
	inputTable: InputTable
	prediction: Prediction
}
