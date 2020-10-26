import * as ui from '@tangramhq/ui'
import { Language } from 'layouts/language'
import { h } from 'preact'

export function Prediction() {
	return <ui.CodeSelect languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.Go]: `import "github.com/tangram-hq/tangram/languages/go"

path := "./heart_disease.tangram"
model, _ := tangram.LoadModelFromFile(path, nil)
defer model.Destroy()

input := tangram.Input{
	"age":    63,
	"gender": "male",
	// ...
}

output := model.PredictOne(input, nil)`,
	[Language.JavaScript]: `const fs = require('fs')
const tangram = require('@tangramhq/tangram')

const modelData = fs.readFileSync('./heart_disease.tangram')
const model = new tangram.Model(modelData)

const output = model.predictSync({
	age: 63,
	gender: 'male',
	// ...
})
`,
	[Language.Python]: `import tangram

model = tangram.Model.from_file('./census.tangram')

output = model.predict({
	'age': 63,
	'gender': 'male',
	# ...
})`,
	[Language.Ruby]: `require 'tangram'

model = Tangram::Model.from_file('./heart_disease.tangram')

output = model.predict({
	age: 63,
	gender: 'male',
	# ...
})`,
}
