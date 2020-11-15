export type Cookie = {
	key: string
	maxAge?: number
	path?: string
	sameSite?: "none" | "lax" | "strict"
	secure?: boolean
	value: string
}

export function get(key: string): Cookie | undefined {
	return getAll().get(key)
}

export function set(cookie: Cookie) {
	let path = cookie.path ? `;path=${cookie.path}` : ""
	let maxAge = cookie.maxAge ? `;max-age=${cookie.maxAge}` : ""
	let sameSite = cookie.sameSite ? `;samesite=${cookie.sameSite}` : ""
	let secure = cookie.secure ? `;secure` : ""
	document.cookie = `${cookie.key}=${cookie.value}${path}${maxAge}${sameSite}${secure}`
}

export function remove(key: string) {
	document.cookie = `${key}=;path=/;max-age=0`
}

export function getAll(): Map<string, Cookie> {
	return new Map(
		document.cookie.split(";").map(cookie => {
			let [key, value] = cookie.trim().split("=")
			return [key.trim(), { key, value }]
		}),
	)
}

export function removeAll() {
	for (let key of getAll().keys()) {
		remove(key)
	}
}
