/* eslint-disable */

import { buffy } from 'deps'

export type Event =
	| { id: 0; value: [buffy.WireType, ArrayBuffer] }
	| { id: 1; value: PageViewEvent }

export let Event = {
	PageView: 1 as const,
	...buffy.createEnum<Event>([1, () => PageViewEvent, 'pageView']),
}

export type PageViewEvent = {
	browserInfo?: BrowserInfo
	url?: string
	referrer?: string
	sessionId?: string
}

export let PageViewEvent = buffy.createStruct<PageViewEvent>(
	[1, () => BrowserInfo, 'browserInfo'],
	[2, () => buffy.Str, 'url'],
	[3, () => buffy.Str, 'referrer'],
	[4, () => buffy.Str, 'sessionId'],
)

export type BrowserInfo = {
	userAgent?: string
	screenWidth?: number
	screenHeight?: number
}

export let BrowserInfo = buffy.createStruct<BrowserInfo>(
	[1, () => buffy.Str, 'userAgent'],
	[2, () => buffy.U32, 'screenWidth'],
	[3, () => buffy.U32, 'screenHeight'],
)
