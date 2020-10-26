# Tangram for Ruby

- [Watch the Video](https://www.tangramhq.com)
- [Read the Docs](https://www.tangramhq.com/docs)

The Tangram Ruby library makes it easy to make predictions with your Tangram machine learning model from Ruby.

## Usage

```
$ gem install tangram
```

```ruby
require 'tangram'

model = Tangram::Model.from_file('./heart_disease.tangram')

input = {
	age: 63,
	gender: 'male',
	# ...
}

output = model.predict(input)

puts(output)
```

For more information, [read the docs](https://www.tangramhq.com/docs).

## Platform Support

Tangram for Ruby is currently supported for Linux, macOS, and Windows on x86-64/amd64/x64 CPUs. Are you interested in another platform? Get in touch via email at help@tangramhq.com.

## Examples

This folder contains two examples, [examples/predict]([examples/predict]) and [examples/monitor](examples/monitor).

### Basic

This example demonstrates loading a model from a `.tangram` file and making a prediction.

To run the example:

```
$ ruby examples/basic.rb
```

### Advanced

This example demonstrates logging predictions and true values to the Tangram reporting and monitoring web app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `examples/heart_disease.tangram` to it.

Then you can run the example:

```
$ ruby examples/monitor/main.rb
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values being reported.

For more information on reporting and monitoring, [read the docs](https://www.tangramhq.com/docs).
