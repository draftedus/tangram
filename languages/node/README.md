# Tangram + Node.js

- [Watch the Video](https://www.tangramhq.com)
- [Read the Docs](https://www.tangramhq.com/docs)

The Tangram Node.js library makes it easy to make predictions with your Tangram machine learning model from Node.js.

## Usage

```
$ npm install @tangramhq/tangram-node
```

```javascript
let fs = require('fs')
let tangram = require('@tangramhq/tangram-node')

let modelData = fs.readFileSync('./heart-disease.tangram')
let model = new tangram.Model(modelData)

let input = {
	age: 63,
	gender: 'male',
	// ...
}

let output = model.predictSync(input)

console.log(output)
```

For more information, [read the docs](https://www.tangramhq.com/docs).

## Platform Support

Tangram for Node is currently supported for Linux, macOS, and Windows on AMD64 CPUs. Are you interested in another platform? Get in touch via email at help@tangramhq.com.

## Examples

This folder contains three examples, [examples/basic]([examples/basic]), [examples/advanced](examples/advanced), and [examples/typescript](examples/typescript).

### Basic

This example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ cd examples/basic
$ npm install
$ npm start
```

### Advanced

This example demonstrates logging predictions and true values to the Tangram reporting and monitoring web app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart-disease.tangram` to it.

Then you can run the example:

```
$ cd examples/advanced
$ npm install
$ npm start
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values being reported.

For more information on reporting and monitoring, [read the docs](https://www.tangramhq.com/docs).

### Typescript

This example shows how to make predictions with your Tangram machine learning model in an app built with [Node.js](https://nodejs.org) and [TypeScript](https://www.typescriptlang.org). It demonstrates how to provide types for the input and output of a model.

To build and run the example:

```
$ cd examples/typescript
$ npm install
$ npm run build
$ npm start
```
