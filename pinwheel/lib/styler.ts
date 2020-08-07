// import { useLayoutEffect } from './react'
// import * as CSS from 'csstype'

// type Styler = {
// 	init(): void
// 	insertedRules: Set<string>
// 	reset(): void
// 	styleSheet: CSSStyleSheet | null
// 	styleTag: HTMLStyleElement | null
// }

// export let styler: Styler = {
// 	init,
// 	insertedRules: new Set<string>(),
// 	reset,
// 	styleSheet: null,
// 	styleTag: null,
// }

// let isBrowser = typeof document !== 'undefined'

// function init() {
// 	if (isBrowser && !styler.styleSheet) {
// 		let existingStyleTag = document.getElementById('styler')
// 		if (
// 			existingStyleTag instanceof HTMLStyleElement &&
// 			existingStyleTag.sheet &&
// 			existingStyleTag.sheet instanceof CSSStyleSheet
// 		) {
// 			styler.styleTag = existingStyleTag
// 		} else {
// 			styler.styleTag = document.createElement('style')
// 			styler.styleTag.id = 'styler'
// 			document.head.appendChild(styler.styleTag)
// 		}
// 		if (
// 			!(styler.styleTag.sheet && styler.styleTag.sheet instanceof CSSStyleSheet)
// 		) {
// 			throw Error()
// 		}
// 		styler.styleSheet = styler.styleTag.sheet
// 	}
// }

// function reset() {
// 	styler.insertedRules.clear()
// }

export function cx(
	...classes: Array<string | null | undefined | false>
): string {
	return classes.filter(c => c).join(' ')
}

// type CSSObject = { [key: string]: CSSObject | CSS.Properties }

// export function css(object: CSSObject): string[] {
// 	return renderCssObject(object)
// }

// function renderCssObject(object: CSSObject): string[] {
// 	return Object.entries(object).map(([key, value]) => {
// 		let children = isProperties(value)
// 			? renderCssProperties(value)
// 			: renderCssObject(value)
// 		return `${key} { ${children} }`
// 	})
// }

// function renderCssProperties(properties: CSS.Properties): string {
// 	return Object.entries(properties)
// 		.map(([key, value]) => renderCssProperty(key, value))
// 		.join(' ')
// }

// function renderCssProperty(key: string, value: string) {
// 	let transformedKey = camelCaseToKebabCase(key)
// 	if (key.startsWith('Moz') || key.startsWith('Webkit')) {
// 		transformedKey = '-' + transformedKey
// 	}
// 	return `${transformedKey}: ${value};`
// }

// function camelCaseToKebabCase(input: string) {
// 	return input.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase()
// }

// function isProperties(
// 	value: CSSObject | CSS.Properties,
// ): value is CSS.Properties {
// 	return typeof Object.entries(value)[0][1] !== 'object'
// }

// export function useCss(...rules: string[][]) {
// 	rules.forEach(rules => rules.forEach(rule => styler.insertedRules.add(rule)))
// 	if (isBrowser) {
// 		// eslint-disable-next-line react-hooks/rules-of-hooks
// 		useLayoutEffect(() => {
// 			rules.forEach(rules =>
// 				rules.forEach(rule => {
// 					styler.init()
// 					if (!styler.styleSheet) {
// 						throw Error()
// 					}
// 					try {
// 						styler.styleSheet.insertRule(
// 							rule,
// 							styler.styleSheet.cssRules.length,
// 						)
// 					} catch {}
// 				}),
// 			)
// 		})
// 	}
// }
