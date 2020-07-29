import { h, ui } from 'deps'
import { Language } from 'layouts/language'

export function CodeFix() {
	return <ui.CodeSelect languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.JavaScript]: `chestPain = chestPain === 'asx' ? 'asymptomatic' : chestPain`,
	[Language.Python]: `chest_pain = 'asymptomatic' if chest_pain == 'asx' else chest_pain`,
	[Language.Ruby]: `chest_pain = chest_pain == 'asx' ? 'asymptomatic' : chest_pain`,
	[Language.Go]: `if chest_pain == 'asx' {
	chest_pain = 'asymptomatic'
}`,
}
