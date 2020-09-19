# Tangram for Python

- [Watch the Video](https://www.tangramhq.com)
- [Read the Docs](https://www.tangramhq.com/docs)

The Tangram Python library makes it easy to make predictions with your Tangram machine learning model from Python.

## Usage

```
$ pip install tangram
```

```python
import tangram

model = tangram.Model.from_file('./heart-disease.tangram')

input = {
	'age': 63,
	'gender': 'male',
	# ...
}

output = model.predict(input)

print(output)
```

For more information, [read the docs](https://www.tangramhq.com/docs).

## Platform Support

Tangram for Python is currently supported for Linux, macOS, and Windows on x86-64/amd64/x64 CPUs. Are you interested in another platform? Get in touch via email at help@tangramhq.com.

## Examples

This repo contains two examples, [examples/basic.py]([examples/basic.py]) and [examples/advanced.py](examples/advanced.py).

### Basic

This example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ python3 examples/basic.py
```

### Advanced

This example demonstrates logging predictions and true values to the Tangram reporting and monitoring app. Before running the example, upload the file `examples/heart-disease.tangram` to either https://app.tangramhq.com or your on-prem deployment.

Then you can run the example:

```
$ python3 examples/advanced.py
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values being reported.

For more information on reporting and monitoring, [read the docs](https://www.tangramhq.com/docs).
