import { AppLayoutInfo } from "layouts/app_layout"

export type Props = {
	appLayoutInfo: AppLayoutInfo
	repos: Array<{
		createdAt: string
		id: string
		ownerName: string | null
		title: string
	}>
}
