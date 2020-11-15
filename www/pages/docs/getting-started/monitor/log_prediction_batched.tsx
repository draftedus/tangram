import * as ui from "@tangramhq/ui"
import { Language } from "layouts/language"
import { h } from "preact"

export function LogPredictionBatched() {
	return <ui.CodeSelect languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.JavaScript]: `for (const input of inputs) {
	const output = model.predictSync(input)
	model.enqueueLogPrediction({
		input,
		output,
	})
}

// Do this at a later point, perhaps on a timer.
model.flushLogQueue()`,
	[Language.Python]: `for input in inputs:
	output = model.predict_sync(input)
	model.enqueue_log_prediction(
		input=input,
		output=output,
	)

// Do this at a later point, perhaps on a timer.
model.flush_log_queue()`,
	[Language.Ruby]: `for input in inputs do
	output = model.predict_sync(input)
	model.enqueue_log_prediction(
		input: input,
		output: output,
	)
end

// Do this at a later point, perhaps on a timer.
model.flush_log_queue()`,
	[Language.Go]: `for i, input := range inputs {
	output := model.PredictOne(input, nil)
	predictionEvent := tangram.LogPredictionOptions{
		Input:      input,
		Output:     output,
	}
	model.EnqueLogPrediction(predictionEvent)
}

// Do this at a later point, perhaps on a timer.
err := model.FlushLogQueue()`,
}
