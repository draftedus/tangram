export * from './document'
export * from './client'
export * from './types'

export function cx(
	...classes: Array<string | null | undefined | false>
): string {
	return classes.filter(c => c).join(' ')
}
