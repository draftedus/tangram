module.exports = function (content) {
	if (this.cacheable) {
		this.cacheable()
	}
	return `
		var data = '${content.toString('base64')}'
		var bytes = typeof window !== 'undefined' ? window.atob(data) : Buffer.from(data, 'base64').toString('binary')
		var array = new Uint8Array(bytes.length)
		for (var i = 0; i < bytes.length; i++) {
			array[i] = bytes.charCodeAt(i)
		}
		module.exports = array.buffer
	`
}

module.exports.raw = true
