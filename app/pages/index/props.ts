import { PinwheelInfo } from '@tangramhq/pinwheel'

export type Props = {
	pinwheelInfo: PinwheelInfo
	repos: Array<{
		createdAt: string
		id: string
		ownerName: string | null
		title: string
	}>
}
