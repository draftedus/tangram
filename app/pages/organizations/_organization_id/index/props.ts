import { AppLayoutInfo } from "layouts/app_layout"

export type Props = {
	appLayoutInfo: AppLayoutInfo
	id: string
	members: Array<{
		email: string
		id: string
		isAdmin: boolean
	}>
	name: string
	repos: Array<{
		id: string
		title: string
	}>
	userId: string
}
