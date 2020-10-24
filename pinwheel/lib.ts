export { Head, Body, Document } from './document'
export { Client, hydrateComponent } from './client'

export type PinwheelInfo = {
	clientJsSrc?: string
	cssSrcs?: string[]
	preloadJsSrcs?: string[]
}
