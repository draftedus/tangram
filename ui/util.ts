/** Merge CSS class names together. */
export function cx(
	...classes: Array<string | null | undefined | false>
): string {
	return classes.filter(c => c).join(' ')
}

/**
 * Format a value between 0 and 1 as a percent, rounded to the nearest percent.
 * For example, 0.75105 will format as "75%".
 * @param value should be between 0 and 1
 */
export function formatPercent(value: number, precision?: number) {
	if (value === 1.0) {
		return '100%'
	} else {
		let v = value * 100
		let p = precision !== undefined ? precision + 2 : 2
		return v.toPrecision(p) + '%'
	}
}

export function formatNumber(
	value?: number | null,
	maxDigits?: number,
): string {
	if (value === undefined || value === null) {
		return ''
	}
	let result = value.toPrecision(maxDigits ?? 6)
	// Remove trailing zeros including the decimal point, for example 12345.000.
	result = result.replace(/\.(0*)$/, '')
	// Remove trailing zeros excluding the decimal point, for example .01234500.
	result = result.replace(/\.([0-9]*)([1-9])(0*)$/, '.$1$2')
	return result
}

export function range(a: number, b?: number) {
	let start = b !== undefined ? a : 0
	let end = b !== undefined ? b : a
	let result: number[] = []
	for (let i = start; i < end; i++) {
		result.push(i)
	}
	return result
}

export let times = <T>(n: number, fn: (index: number) => T): T[] => {
	let result = []
	for (let i = 0; i < n; i++) {
		result.push(fn(i))
	}
	return result
}

export function randomInt(max: number) {
	return Math.floor(Math.random() * Math.floor(max))
}

export let zip = <T1, T2>(a: T1[], b: T2[]): Array<[T1, T2]> => {
	let result: Array<[T1, T2]> = []
	let length = Math.min(a.length, b.length)
	for (let i = 0; i < length; i++) {
		result.push([a[i], b[i]])
	}
	return result
}

export function getCssVariableValue(variable: string): String {
	let root = document.documentElement
	return window.getComputedStyle(root).getPropertyValue(variable)
}
