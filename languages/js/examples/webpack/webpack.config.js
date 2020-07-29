const HtmlWebpackPlugin = require('html-webpack-plugin')
module.exports = {
	module: {
		rules: [
			{
				test: /\.tangram$/,
				loaders: ['@tangramhq/model-loader'],
			},
		],
	},
	plugins: [new HtmlWebpackPlugin()],
}
