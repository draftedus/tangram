import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	columnCount: number
	columnStats: Array<{
		columnType?: ColumnType
		invalidCount?: number
		max: number | null
		mean: number | null
		min: number | null
		name: string
		std: number | null
		uniqueCount: number | null
		variance: number | null
	}>

	modelId: string
	modelLayoutInfo: ModelLayoutInfo

	rowCount: number
	targetColumnStats: {
		columnType?: ColumnType
		invalidCount?: number
		max: number | null
		mean: number | null
		min: number | null
		name: string
		std: number | null
		uniqueCount: number | null
		variance: number | null
	}
}

export enum ColumnType {
	Unknown = "unknown",
	Number = "number",
	Enum = "enum",
	Text = "text",
}
