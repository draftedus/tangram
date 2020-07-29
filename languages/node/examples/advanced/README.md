# Tangram + Node.js

This example shows how to make predictions with your Tangram machine learning model in an app built with [Node.js](https://nodejs.org).

```
$ npm install --save @tangramhq/tangram
```

```javascript
const fs = require('fs')
const tangram = require('@tangramhq/tangram')

const modelData = fs.readFileSync('./heart-disease.tangram')
const model = new tangram.Model(modelData)

const input = {
	age: 63,
	gender: 'male',
	// ...
}

const output = model.predictSync(input)

console.log(output)
```

For more information, [read the docs](https://www.tangramhq.com/docs).

## Examples

This folder contains two examples, [predict]([predict]) and [monitor](monitor).

### Predict

This example demonstrates loading a model from a `.tangram` file and making a prediction.

### Monitor

This example demonstrates logging predictions and true values to the Tangram reporting and monitoring app. Before running the example, upload the `heart-disease.tangram` file in [monitor/heart-disease.tangram](monitor/heart-disease.tangram) to either https://app.tangramhq.com or your on-prem deployment.

Then you can run the example:

```
$ node monitor/main.js
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values being reported.

For more information on reporting and monitoring, [read the docs](https://www.tangramhq.com/docs).
