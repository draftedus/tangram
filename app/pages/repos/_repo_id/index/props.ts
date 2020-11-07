import { AppLayoutInfo } from 'layouts/app_layout'

export type Props = {
	appLayoutInfo: AppLayoutInfo
	models: Array<{
		createdAt: string
		id: string
	}>
	title: string
}
