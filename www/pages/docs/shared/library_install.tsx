import { h, ui } from 'deps'
import { Language } from 'layouts/language'

export function LibraryInstall() {
	return <ui.CodeSelect hideLineNumbers={true} languages={codeForLanguage} />
}

let codeForLanguage = {
	[Language.JavaScript]: `$ npm install @tangramhq/tangram`,
	[Language.Python]: `$ pip install tangram`,
	[Language.Ruby]: `$ gem install tangram`,
	[Language.Go]: `$ go get -u github.com/tangram-hq/tangram/languages/go `,
}
