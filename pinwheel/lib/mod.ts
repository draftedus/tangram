export * from './document'
export * from './client'
export * from './types'
export * from './styler'
export * from './link'
export * from './react'

import { VNode, renderToString } from './react'

export function renderPage(element: VNode): string {
	return '<!doctype html>' + renderToString(element)
}
