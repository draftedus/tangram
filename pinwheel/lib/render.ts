import { ComponentType, h, renderToString } from './react'
import { DocumentProps } from './types'

export function renderPage<T>(
	documentComponent: ComponentType<DocumentProps>,
	pageComponent: ComponentType<T>,
	props: T,
	clientJsPath?: string,
): string {
	// render the html for the page
	let html = renderToString(h(pageComponent, props))
	// render the document with the html for the page
	html = renderToString(
		h(documentComponent, {
			clientJsPath,
			html,
		}),
	)
	html = '<!doctype html>' + html
	return html
}
