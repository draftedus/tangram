let chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
let lookup = new Uint8Array(256)
for (var i = 0; i < chars.length; i++) {
	lookup[chars.charCodeAt(i)] = i
}

export let base64Encode = (data: ArrayBuffer): string => {
	let bytes = new Uint8Array(data)
	let result = ''
	for (let i = 0; i < bytes.length; i += 3) {
		result += chars[bytes[i] >> 2]
		result += chars[((bytes[i] & 3) << 4) | (bytes[i + 1] >> 4)]
		result += chars[((bytes[i + 1] & 15) << 2) | (bytes[i + 2] >> 6)]
		result += chars[bytes[i + 2] & 63]
	}
	if (bytes.length % 3 === 2) {
		result = result.substring(0, result.length - 1) + '='
	} else if (bytes.length % 3 === 1) {
		result = result.substring(0, result.length - 2) + '=='
	}
	return result
}

export let base64Decode = (string: string): ArrayBuffer => {
	let resultLength = string.length * 0.75
	if (string[string.length - 1] === '=') {
		resultLength--
		if (string[string.length - 2] === '=') {
			resultLength--
		}
	}
	let result = new Uint8Array(resultLength)
	let j = 0
	let i = 0
	while (i < string.length) {
		let encoded1 = lookup[string.charCodeAt(i)]
		let encoded2 = lookup[string.charCodeAt(i + 1)]
		let encoded3 = lookup[string.charCodeAt(i + 2)]
		let encoded4 = lookup[string.charCodeAt(i + 3)]
		result[j++] = (encoded1 << 2) | (encoded2 >> 4)
		result[j++] = ((encoded2 & 15) << 4) | (encoded3 >> 2)
		result[j++] = ((encoded3 & 3) << 6) | (encoded4 & 63)
		i += 4
	}
	return result.buffer
}
