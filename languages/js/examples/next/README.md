# Tangram + Next

This example shows how to make predictions with your Tangram machine learning model in an app built with [Next.js](https://nextjs.org).

## Run

To run the example, run `yarn install` to install dependencies and `yarn dev` to run the development server. Then, open your browser to `http://localhost:3000`.

## Guide

To use your model with Next.js, you need to install the following npm packages:

- [@tangramhq/tangram](https://npmjs.com/@tangramhq/tangram)
- [@tangramhq/model-loader](https://npmjs.com/@tangramhq/model-loader)

Then, you need to update your `next.config.js` js to specify the Tangram model loader for `.tangram` files, like this:

```javascript
module.exports = {
	webpack(config) {
		config.module.rules.push({
			test: /\.tangram$/,
			loaders: ['@tangramhq/model-loader'],
		})
		return config
	},
}
```

The Tangram model loader inlines your `.tangram` model file's data into your Webpack bundle.

Now you can import your model and make predictions:

```javascript
import modelData from '../heart-disease.tangram'
import * as tangram from '@tangramhq/tangram'

const model = new tangram.Model(modelData)

const input = {
	age: 63,
	gender: 'male',
	// ...
}

const output = model.predictSync(input)
```
