module.exports = require('babel-plugin-macros').createMacro(
	({ references, state }) => {
		for (const reference of references.default) {
			const filePath = require('path').dirname(state.file.opts.filename)
			const args = reference.parentPath.get('arguments')
			if (!Array.isArray(args)) {
				throw Error()
			}
			const arg = args[0]
			if (arg === undefined) {
				throw Error()
			}
			const relativePath = arg.evaluate().value
			if (relativePath === undefined) {
				throw Error()
			}
			const absolutePath = require.resolve(relativePath, { paths: [filePath] })
			const bytes = require('fs').readFileSync(absolutePath)
			const bytesEncoded = bytes.toString('base64')
			reference.parentPath.replaceWithSourceString(
				`Uint8Array.from(atob('${bytesEncoded}'), c => c.charCodeAt(0))`,
			)
		}
	},
)
