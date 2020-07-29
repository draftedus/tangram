# Tangram + JavaScript

- [Watch the Video](https://www.tangramhq.com)
- [Read the Docs](https://www.tangramhq.com/docs)

The Tangram JavaScript library makes it easy to make predictions with your Tangram machine learning model from JavaScript.

## Usage

The Tangram JavaScript library is compatible with both Node.js and web browsers. Click the link below for more information for the environment you are using.

### Node.js

- [Node.js](examples/node)
- [Node.js + TypeScript](examples/node-typescript)

### Browser

- [Create React App](examples/create-react-app)
- [Next.js](examples/next)
- [Parcel](examples/parcel)

## Platform Support

`tangram-js` is compiled to pure javascript so it will run in Node.js on any platform, IE 10+, and any version of Edge, Chrome, Safari, or Firefox released in the last 10 years. Note that it is distributed as a commonjs module, so you will need to use a bundler. The examples folder contains example integrations with a number of bundlers. Are you looking to use `tangram-js` in an environment we don't support? Get in touch via email at help@tangramhq.com.

### Basic Usage

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
