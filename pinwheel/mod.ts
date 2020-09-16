export { Head, Body, Document } from './document'
export { Client, hydrateComponent } from './client'

export type PinwheelInfo = {
	clientJsSrc?: string
	preloadJsSrcs?: string[]
}

export function cx(
	...classes: Array<string | null | undefined | false>
): string {
	return classes.filter(c => c).join(' ')
}
