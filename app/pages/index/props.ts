export type Props = {
	repos: Array<{
		createdAt: string
		id: string
		ownerName: string | null
		title: string
	}>
}
