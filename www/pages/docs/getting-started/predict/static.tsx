import { LibraryInstall } from '../../shared/library_install'
import { Predict } from './predict'
import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { DocsLayout, DocsPage } from 'layouts/docs_layout'

type Props = {
	pinwheelInfo: PinwheelInfo
}
export default (props: Props) =>
	renderPage(
		<DocsLayout
			pinwheelInfo={props.pinwheelInfo}
			selectedPage={DocsPage.Predict}
		>
			<ui.S1>
				<ui.H1>{'Predict'}</ui.H1>
				<ui.S2>
					<ui.P>
						{'After training a model using the '}
						<ui.InlineCode>{'tangram train'}</ui.InlineCode>
						{' command, a file called '}
						<ui.InlineCode>{'heart-disease.tangram'}</ui.InlineCode>
						{
							' is output in the current directory. We can use this file to make predictions in our code. Libraries are available for Go, JavaScript, Python, and Ruby. Support for C/C++, C#, Java, PHP, and Rust is coming soon.'
						}
					</ui.P>
					<ui.Code
						code="$ ls<br/>heart-disease.tangram<br/>heart-disease.csv"
						hideLineNumbers={true}
					/>
					<ui.P>
						{'Install the Tangram library for your programming language.'}
					</ui.P>
					<LibraryInstall />
					<ui.P>
						{'Create a source file and fill it with the following code:'}
					</ui.P>
					<Predict />
					<ui.P>
						{
							"That's it! Prediction happens locally, as opposed to calling an external API, so its super fast and patient data stays secure. In this tutorial we just hard coded the data for an example patient but in your application you will probably retrieve data from a database or user input."
						}
					</ui.P>
				</ui.S2>
			</ui.S1>
		</DocsLayout>,
	)
