import { h, ui } from 'deps'
import { Language } from 'layouts/language'

export function LogTrueValue() {
	return <ui.CodeSelect languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.JavaScript]: `model.logTrueValue({
	identifier: 'John Doe',
	trueValue: 'Positive',
})`,
	[Language.Python]: `model.log_true_value(
	identifier='John Doe',
	true_value='Positive',
)`,
	[Language.Ruby]: `model.log_true_value(
	identifier: 'John Doe',
	true_value: 'Positive',
)`,
	[Language.Go]: `trueValueEvent := tangram.LogTrueValueOptions{
	Identifier: "John Doe",
	TrueValue:  "Positive",
}
err := model.LogTrueValue(trueValueEvent)
`,
}
