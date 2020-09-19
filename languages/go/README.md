# Tangram for Go

- [Watch the Video](https://www.tangramhq.com)
- [Read the Docs](https://www.tangramhq.com/docs)

The Tangram Go library makes it easy to make predictions with your Tangram machine learning model from Go.

## Usage

```
$ go get -u github.com/tangram-hq/tangram/languages/go
```

```go
import "github.com/tangram-hq/tangram/languages/go"

model, _ := tangram.LoadModelFromFile("./heart-disease.tangram", nil)
defer model.Destroy()

input := tangram.Input{
	"age":    63,
	"gender": "male",
	// ...
}

output := model.PredictOne(input, nil)

fmt.Println("Output:", output.ClassName)
}
```

For more information, [read the docs](https://www.tangramhq.com/docs).

## Platform Support

Tangram for Go is currently supported for `$GOOS` values of `darwin`, `linux`, and `windows` and `$GOARCH` values of `amd64` and `arm64`. Are you interested in another platform? Get in touch via email at help@tangramhq.com.

Tangram for Go links to the [libtangram](https://github.com/tangram-hq/libtangram) C library, so cgo is required. libtangram will be linked statically into your executable, so when you run `go build` you will still get a statically linked executable you can move around without having to worry about dynamic linking errors.

## Examples

This repo contains two examples, [examples/predict](examples/predict) and [examples/monitor](examples/monitor).

### Basic

This example demonstrates loading a model from a `.tangram` file and making a prediction.

```
$ cd examples
$ go run basic.go
```

### Advanced

This example demonstrates logging predictions and true values to the Tangram reporting and monitoring app. Before running the example, upload the file `examples/heart-disease.tangram` to https://app.tangramhq.com.

Then you can run the example:

```
$ cd examples
$ go run advanced.go
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values being reported.

For more information on reporting and monitoring, [read the docs](https://www.tangramhq.com/docs).
