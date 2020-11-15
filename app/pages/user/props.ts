import { AppLayoutInfo } from "layouts/app_layout"

export type Props = {
	appLayoutInfo: AppLayoutInfo
	inner: Inner
}

export enum InnerType {
	NoAuth = "no_auth",
	Auth = "auth",
}

export type Inner =
	| {
			type: InnerType.NoAuth
			value: NoAuthProps
	  }
	| {
			type: InnerType.Auth
			value: AuthProps
	  }

export type NoAuthProps = {}

export type AuthProps = {
	email: string
	organizations: Array<{
		id: string
		name: string
	}>
	repos: Array<{
		id: string
		title: string
	}>
}
