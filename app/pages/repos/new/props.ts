import { PinwheelInfo } from '@tangramhq/pinwheel'

export type Props = {
	error?: string
	owner?: string
	owners?: Owner[]
	pinwheelInfo: PinwheelInfo
	title?: string
}

type Owner = {
	title: string
	value: string
}
