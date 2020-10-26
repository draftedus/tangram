import { AppLayoutInfo } from 'layouts/app_layout'

export type Props = {
	appLayoutInfo: AppLayoutInfo
	card: {
		brand: string
		country: string
		expMonth: number
		expYear: number
		last4: string
		name: string
	} | null
	id: string
	members: Array<{
		email: string
		id: string
		isAdmin: boolean
	}>
	name: string
	plan: Plan
	repos: Array<{
		id: string
		title: string
	}>
	stripePublishableKey: string
	userId: string
}

export enum Plan {
	Trial = 'trial',
	Free = 'free',
	Startup = 'startup',
	Team = 'team',
	Enterprise = 'enterprise',
}
