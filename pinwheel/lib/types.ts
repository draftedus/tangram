export type Router = {
	loading: boolean
	location: Location
	push: RouterPushFunction
	reload: () => Promise<void>
	subscribe: (callback: RouterSubscriptionCallback) => RouterUnsubscribeCallback
	subscriptions: RouterSubscriptionCallback[]
}

export type RouterPushFunction = (
	pagename: string,
	options?: {
		hash?: string
		pathParams?: Map<string, string> | { [key: string]: string }
		replace?: boolean
		searchParams?: URLSearchParams | { [key: string]: string }
	},
) => Promise<void>

export enum RouterEventType {
	LOADING,
	REDIRECT,
	ERROR,
	COMPLETE,
}

export type RouterEvent =
	| {
			location: Location
			type: RouterEventType.LOADING
	  }
	| {
			type: RouterEventType.REDIRECT
	  }
	| {
			type: RouterEventType.ERROR
	  }
	| {
			location: Location
			type: RouterEventType.COMPLETE
	  }

export type RouterSubscriptionCallback = (event: RouterEvent) => void

export type RouterUnsubscribeCallback = () => void

export type Location = {
	hash?: string
	pagename: string
	pathParams?: Map<string, string>
	pathname: string
	searchParams?: URLSearchParams
}

export type PageModule<T> = {
	default: any
	getServerProps: GetServerProps<T>
}

export type DocumentProps = {
	clientJsPath?: string
	dev?: boolean
	html?: string
	preloadJsPaths?: string[]
}

export type ErrorProps = {
	error: any
}

export type GetServerPropsContext = {
	headers: Headers
	pathParams?: Map<string, string>
	searchParams?: URLSearchParams
}

export type GetServerPropsResult<T> = {
	props?: T
}

export type GetServerProps<T> = (context: GetServerPropsContext) => Promise<T>

export class Redirect {
	constructor(public url: string) {}
}
