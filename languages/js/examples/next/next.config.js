module.exports = {
	webpack(config) {
		config.module.rules.push({
			test: /\.tangram$/,
			loaders: ['@tangramhq/model-loader'],
		})
		return config
	},
}
