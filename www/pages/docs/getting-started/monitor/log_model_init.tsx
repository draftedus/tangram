import * as ui from "@tangramhq/ui"
import { Language } from "layouts/language"
import { h } from "preact"

export function LogModelInit() {
	return <ui.CodeSelect languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.JavaScript]: `const model = new tangram.Model(modelData, {
	base_url: process.env.TANGRAM_URL || 'https://api.tangramhq.com'
})`,
	[Language.Python]: `base_url = os.getenv('TANGRAM_URL', default='https://api.tangramhq.com')
model = tangram.Model.from_file(model_path, base_url=base_url)`,
	[Language.Ruby]: `base_url = ENV['TANGRAM_URL'] || 'https://api.tangramhq.com'
model = Tangram::Model.from_file(model_path, base_url: base_url)`,
	[Language.Go]: `baseURL, present := os.LookupEnv("TANGRAM_URL")
if !present {
	baseURL = "https://api.tangramhq.com"
}
options := tangram.ModelOptions{
	BaseURL: baseURL,
}
model, err := tangram.LoadModelFromFile("./heart_disease.tangram", &options)
`,
}
