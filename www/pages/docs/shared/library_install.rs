import * as ui from "@tangramhq/ui"
import { Language } from "layouts/language"
import { h } from "preact"

export function LibraryInstall() {
	return <ui.CodeSelect hideLineNumbers={true} languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.JavaScript]: `$ npm install @tangramhq/tangram`,
	[Language.Python]: `$ pip install tangram`,
	[Language.Ruby]: `$ gem install tangram`,
	[Language.Go]: `$ go get -u github.com/tangram-hq/tangram/languages/go `,
}
