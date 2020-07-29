# Tangram + Parcel

This example shows how to make predictions with your Tangram machine learning model in an app built with [Parcel](https://parceljs.org).

## Run

To run the example, run `yarn install` to install dependencies and `yarn dev` to run the development server. Then, open your browser to `http://localhost:8080`.

## Guide

To use your model with Parcel, you need to install the following npm packages:

- [@tangramhq/tangram](https://npmjs.com/@tangramhq/tangram)

Now you can import your model and make predictions:

```javascript
import * as tangram from '@tangramhq/tangram'
import { readFileSync } from 'fs'
import { Buffer } from 'buffer'

const modelData = readFileSync('src/heart-disease.tangram')
const model = new tangram.Model(modelData)

const input = {
	age: 63,
	gender: 'male',
	// ...
}

const output = model.predictSync(input)
```
