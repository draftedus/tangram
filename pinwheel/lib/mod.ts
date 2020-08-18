export * from './document'
export * from './client'
export * from './types'
export * from './link'
export * from './react'

import { VNode, renderToString } from './react'

export function renderPage(element: VNode): string {
	return '<!doctype html>' + renderToString(element)
}

export function cx(
	...classes: Array<string | null | undefined | false>
): string {
	return classes.filter(c => c).join(' ')
}
