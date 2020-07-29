import { h, ui } from 'deps'
import { Language } from 'layouts/language'

export function LogPrediction() {
	return <ui.CodeSelect languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.JavaScript]: `model.logPrediction({
	identifier: 'John Doe,
	options: { threshold: 0.5 }
	input,
	output,
})
`,
	[Language.Python]: `model.log_prediction(
	identifier='John Doe',
	options={ 'threshold': 0.5 },
	input=input,
	output=output,
)`,
	[Language.Ruby]: `model.log_prediction(
	identifier: 'John Doe',
	options: { threshold: 0.5 },
	input: input,
	output: output,
)`,
	[Language.Go]: `predictOptions := tangram.PredictOptions{
	Threshold: 0.5,
}
predictionEvent := tangram.LogPredictionOptions{
	Identifier: "John Doe",
	Options:    predictOptions,
	Input:      input,
	Output:     output,
}
err := model.LogPrediction(predictionEvent)`,
}
