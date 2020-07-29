# Tangram + Create React App

This example shows how to make predictions with your Tangram machine learning model in an app built with [Create React App](https://github.com/facebook/create-react-app).

## Run

To run the example, run `yarn install` to install dependencies and `yarn start` to run the development server.

## Guide

To use your model with Create React App, you need to install the following npm packages:

- [@tangramhq/tangram](https://npmjs.com/@tangramhq/tangram)
- [@tangramhq/inline-model.macro](https://npmjs.com/@tangramhq/inline-model.macro)

[@tangramhq/inline-model.macro] is a [babel macro](https://github.com/kentcdodds/babel-plugin-macros) that inlines your model file's data into your Webpack bundle.

Now you can import your model and make predictions:

```javascript
import * as tangram from '@tangramhq/tangram'
import inlineModel from '@tangramhq/inline-model.macro'

const modelData = inlineModel('./heart-disease.tangram')
const model = new tangram.Model(modelData)

const input = {
	age: 63,
	gender: 'male',
	// ...
}

const output = model.predictSync(input)
```
