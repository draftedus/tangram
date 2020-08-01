import { Children, Fragment, css, h, useCss, useEffect } from './deps'
import { Language, grammars, highlight } from './syntax/mod'
import { syntaxColors, variables } from './theme'
import { times } from './util'

type CodeForLanguage = {
	[Language.JavaScript]: string
	[Language.Ruby]: string
	[Language.Python]: string
	[Language.Go]: string
}

type CodeProps = {
	code: string
	hideLineNumbers?: boolean
	language?: Language
}

let codeCss = css({
	[`.code`]: {
		MozTabSize: '2',
		backgroundColor: variables.colors.header,
		borderRadius: variables.border.radius,
		overflow: 'hidden',
		tabSize: '2',
	},
})

export function Code(props: CodeProps) {
	useCss(codeCss, codeInnerWrapperCss, codeInnerValueCss)
	return (
		<div class="code">
			<div class="code-inner-wrapper" style={{ display: 'block' }}>
				{!props.hideLineNumbers && (
					<LineNumbers count={countLines(props.code)} />
				)}
				<div
					class="code-inner-code"
					dangerouslySetInnerHTML={{
						__html: props.language
							? highlight(props.code, grammars[props.language], syntaxColors)
							: props.code,
					}}
				/>
			</div>
		</div>
	)
}

type CodeSelectProps = {
	hideLineNumbers?: boolean
	languages?: CodeForLanguage
}

let codeGridCss = css({
	[`.code-grid`]: {
		display: 'grid',
		gridColumnGap: '1rem',
		gridRowGap: '0.5rem',
		gridTemplateAreas: "'go javascript python ruby' 'code code code code'",
		gridTemplateColumns: 'auto auto auto 1fr',
		gridTemplateRows: 'auto',
		justifyContent: 'start',
		justifyItems: 'start',
		paddingTop: '0.5rem',
		position: 'relative',
	},
})

export function CodeSelect(props: CodeSelectProps) {
	let name = Math.random().toString()
	useCss(codeCss, codeGridCss)
	useEffect(() => {
		let radioElements = document.querySelectorAll('input[type=radio]')
		radioElements.forEach(radioElement => {
			radioElement.addEventListener('click', event => {
				if (!(event.currentTarget instanceof HTMLInputElement)) throw Error()
				let lang = event.currentTarget.dataset.lang
				let langElements = document.querySelectorAll(
					`input[type=radio][data-lang=${lang}]`,
				)
				langElements.forEach(langElement => {
					if (!(langElement instanceof HTMLInputElement)) throw Error()
					langElement.checked = true
				})
			})
		})
	})
	return (
		<div class="code code-grid">
			{props.languages && (
				<Fragment>
					<CodeOption
						checked
						code={props.languages[Language.Go]}
						language={Language.Go}
						name={name}
					/>
					<CodeOption
						code={props.languages[Language.JavaScript]}
						language={Language.JavaScript}
						name={name}
					/>
					<CodeOption
						code={props.languages[Language.Python]}
						language={Language.Python}
						name={name}
					/>
					<CodeOption
						code={props.languages[Language.Ruby]}
						language={Language.Ruby}
						name={name}
					/>
				</Fragment>
			)}
		</div>
	)
}

type CodeOptionProps = {
	checked?: boolean
	code: string
	hideLineNumbers?: boolean
	language: Language
	name: string
}

let codeInnerWrapperCss = css({
	[`.code-inner-wrapper`]: {
		backgroundColor: syntaxColors.background,
		display: 'none',
		grid: 'auto / auto minmax(0, 1fr)',
		gridArea: 'code',
		overflow: 'auto',
		padding: '0.5rem',
		width: '100%',
	},
})

let codeRadioInputCss = css({
	[`.code-radio-input`]: {
		height: '0',
		margin: '0',
		opacity: '0',
		position: 'absolute',
		right: '0',
		top: '0',
		width: '0',
	},
	[`.code-radio-input[data-lang="go"]:checked ~ label[data-lang="go"]`]: {
		color: variables.colors.accent,
	},
	[`.code-radio-input[data-lang="javascript"]:checked ~ label[data-lang="javascript"]`]: {
		color: variables.colors.accent,
	},
	[`.code-radio-input[data-lang="python"]:checked ~ label[data-lang="python"]`]: {
		color: variables.colors.accent,
	},
	[`.code-radio-input[data-lang="ruby"]:checked ~ label[data-lang="ruby"]`]: {
		color: variables.colors.accent,
	},
	[`.code-radio-input[data-lang="go"]:checked ~ .code-inner-wrapper[data-lang="go"]`]: {
		display: 'grid',
	},
	[`.code-radio-input[data-lang="javascript"]:checked ~ .code-inner-wrapper[data-lang="javascript"]`]: {
		display: 'grid',
	},
	[`.code-radio-input[data-lang="python"]:checked ~ .code-inner-wrapper[data-lang="python"]`]: {
		display: 'grid',
	},
	[`.code-radio-input[data-lang="ruby"]:checked ~ .code-inner-wrapper[data-lang="ruby"]`]: {
		display: 'grid',
	},
})

let codeRadioLabelCss = css({
	[`.code-radio-label`]: {
		cursor: 'pointer',
	},
	[`.code-radio-label:first-of-type`]: {
		paddingLeft: '0.5rem',
	},
})

let codeInnerValueCss = css({
	[`.code-inner-code`]: {
		MozTabSize: '2',
		boxSizing: 'border-box',
		caretColor: 'black',
		color: variables.colors.text,
		fontSize: '1rem',
		lineHeight: '1.5',
		tabSize: '2',
		whiteSpace: 'pre',
	},
})

function CodeOption(props: CodeOptionProps) {
	let optionId = Math.random().toString()
	useCss(
		codeInnerWrapperCss,
		codeInnerValueCss,
		codeRadioInputCss,
		codeRadioLabelCss,
	)
	return (
		<Fragment>
			<input
				checked={props.checked ?? undefined}
				class="code-radio-input"
				data-lang={props.language}
				id={optionId}
				name={props.name}
				type="radio"
				value={props.language}
			/>
			<label
				class="code-radio-label"
				data-lang={props.language}
				for={optionId}
				style={{
					gridArea: props.language,
				}}
			>
				{props.language}
			</label>
			<div class="code-inner-wrapper" data-lang={props.language}>
				{!props.hideLineNumbers && (
					<LineNumbers count={countLines(props.code)} />
				)}
				<div
					class="code-inner-code"
					dangerouslySetInnerHTML={{
						__html: props.language
							? highlight(props.code, grammars[props.language], syntaxColors)
							: props.code,
					}}
				/>
			</div>
		</Fragment>
	)
}

type LineNumbersProps = {
	count: number
}

let wrapperCss = css({
	[`.code-line-numbers-wrapper`]: {
		marginRight: '0.5rem',
		textAlign: 'right',
		userSelect: 'none',
	},
})

let lineNumberCss = css({
	[`.code-line-numbers`]: {
		color: syntaxColors.lineNumbers,
		fontSize: '1rem',
		lineHeight: '1.5',
	},
})

function LineNumbers(props: LineNumbersProps) {
	useCss(wrapperCss, lineNumberCss)
	return (
		<div class="code-line-numbers-wrapper">
			{times(props.count, index => (
				<div class="code-line-numbers" key={index}>
					{index + 1}
				</div>
			))}
		</div>
	)
}

type InlineCodeProps = { children?: Children }

let containerCss = css({
	[`.inline-code-wrapper`]: {
		backgroundColor: syntaxColors.background,
		borderRadius: variables.border.radius,
		color: syntaxColors.foreground,
		fontSize: '0.8rem',
		padding: '0.1rem 0.2rem',
	},
})

export function InlineCode(props: InlineCodeProps) {
	useCss(containerCss)
	return <span class="inline-code-wrapper">{props.children}</span>
}

function countLines(text: string) {
	// trim trailing newline
	let trimmedText =
		text[text.length - 1] === '\n' ? text.substr(0, text.length - 1) : text
	return trimmedText.split('\n').length
}
