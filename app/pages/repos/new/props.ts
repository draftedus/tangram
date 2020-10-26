import { AppLayoutInfo } from 'layouts/app_layout'

export type Props = {
	appLayoutInfo: AppLayoutInfo
	error?: string
	owner?: string
	owners?: Owner[]
	title?: string
}

type Owner = {
	title: string
	value: string
}
