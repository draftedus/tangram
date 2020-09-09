import { JSX } from 'preact'
import { renderToString } from 'preact-render-to-string'

export function renderPage(element: JSX.Element): string {
	return '<!doctype html>' + renderToString(element)
}
