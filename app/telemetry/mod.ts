import { Event } from './generated/mod'
import { buffy } from 'deps'

let sessionId = [...crypto.getRandomValues(new Uint8Array(16))]
	.map(bit => (bit < 0x10 ? '0' : '') + bit.toString(16))
	.join('')

function track(event: Event) {
	let telemetryBaseUrlElement = document.head.querySelector(
		'meta[name=tangram-telemetry-base-url]',
	)
	if (!telemetryBaseUrlElement) {
		return
	}
	let baseUrl = telemetryBaseUrlElement.getAttribute('content')
	if (!baseUrl) {
		return
	}
	let writer = new buffy.Writer()
	Event.serialize(writer, event)
	let data = writer.bytes()
	let blob = new Blob([data], { type: 'text/plain' })
	navigator.sendBeacon(baseUrl + '/track/app', blob)
}

type TrackPageViewOptions = {
	referrer: string
}

export function trackPageView(options: TrackPageViewOptions) {
	track({
		id: Event.PageView,
		value: {
			browserInfo: {
				screenHeight: window.screen.height,
				screenWidth: window.screen.width,
				userAgent: navigator.userAgent,
			},
			referrer: options.referrer,
			sessionId,
			url: window.location.href,
		},
	})
}
