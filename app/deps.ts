export * from '../pinwheel/lib/mod'
export * as buffy from '../buffy/buffy-js/mod'
export * as ui from '../ui/mod'

export function assert(
	condition: unknown,
	message?: string,
): asserts condition {
	if (!condition) {
		throw new Error(message)
	}
}

export function r<T>(t: T | null | undefined, message?: string): T {
	if (t === null || t === undefined) {
		throw Error(message)
	}
	return t
}
