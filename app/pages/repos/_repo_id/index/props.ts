import { PinwheelInfo } from '@tangramhq/pinwheel'

export type Props = {
	models: Array<{
		createdAt: string
		id: string
	}>
	pinwheelInfo: PinwheelInfo
}
