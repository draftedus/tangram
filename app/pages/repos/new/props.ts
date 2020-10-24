export type Props = {
	error?: string
	owner?: string
	owners?: Owner[]

	title?: string
}

type Owner = {
	title: string
	value: string
}
