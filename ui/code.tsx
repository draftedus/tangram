import {
	Children,
	Fragment,
	css,
	cssClass,
	cx,
	h,
	useCss,
	useEffect,
} from './deps'
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

let codeClass = cssClass()
let codeCss = css({
	[`.${codeClass}`]: {
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
		<div class={codeClass}>
			<div class={codeInnerWrapperClass} style={{ display: 'block' }}>
				{!props.hideLineNumbers && (
					<LineNumbers count={countLines(props.code)} />
				)}
				<div
					class={codeInnerValueClass}
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

let codeGridClass = cssClass()
let codeGridCss = css({
	[`.${codeGridClass}`]: {
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
			radioElement.addEventListener('click', (event: any) => {
				let lang = event.target.dataset.lang
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
		<div class={cx(codeClass, codeGridClass)}>
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

let codeInnerWrapperClass = cssClass()
let codeInnerWrapperCss = css({
	[`.${codeInnerWrapperClass}`]: {
		backgroundColor: syntaxColors.background,
		display: 'none',
		grid: 'auto / auto minmax(0, 1fr)',
		gridArea: 'code',
		overflow: 'auto',
		padding: '0.5rem',
		width: '100%',
	},
})

let codeRadioInputClass = cssClass()
let codeRadioInputCss = css({
	[`.${codeRadioInputClass}`]: {
		height: '0',
		margin: '0',
		opacity: '0',
		position: 'absolute',
		right: '0',
		top: '0',
		width: '0',
	},
	[`.${codeRadioInputClass}[data-lang="go"]:checked ~ label[data-lang="go"]`]: {
		color: variables.colors.accent,
	},
	[`.${codeRadioInputClass}[data-lang="javascript"]:checked ~ label[data-lang="javascript"]`]: {
		color: variables.colors.accent,
	},
	[`.${codeRadioInputClass}[data-lang="python"]:checked ~ label[data-lang="python"]`]: {
		color: variables.colors.accent,
	},
	[`.${codeRadioInputClass}[data-lang="ruby"]:checked ~ label[data-lang="ruby"]`]: {
		color: variables.colors.accent,
	},
	[`.${codeRadioInputClass}[data-lang="go"]:checked ~ .${codeInnerWrapperClass}[data-lang="go"]`]: {
		display: 'grid',
	},
	[`.${codeRadioInputClass}[data-lang="javascript"]:checked ~ .${codeInnerWrapperClass}[data-lang="javascript"]`]: {
		display: 'grid',
	},
	[`.${codeRadioInputClass}[data-lang="python"]:checked ~ .${codeInnerWrapperClass}[data-lang="python"]`]: {
		display: 'grid',
	},
	[`.${codeRadioInputClass}[data-lang="ruby"]:checked ~ .${codeInnerWrapperClass}[data-lang="ruby"]`]: {
		display: 'grid',
	},
})

let codeRadioLabelClass = cssClass()
let codeRadioLabelCss = css({
	[`.${codeRadioLabelClass}`]: {
		cursor: 'pointer',
	},
	[`.${codeRadioLabelClass}:first-of-type`]: {
		paddingLeft: '0.5rem',
	},
})

let codeInnerValueClass = cssClass()
let codeInnerValueCss = css({
	[`.${codeInnerValueClass}`]: {
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
				class={codeRadioInputClass}
				data-lang={props.language}
				id={optionId}
				name={props.name}
				type="radio"
				value={props.language}
			/>
			<label
				class={codeRadioLabelClass}
				data-lang={props.language}
				for={optionId}
				style={{
					gridArea: props.language,
				}}
			>
				{props.language}
			</label>
			<div class={codeInnerWrapperClass} data-lang={props.language}>
				{!props.hideLineNumbers && (
					<LineNumbers count={countLines(props.code)} />
				)}
				<div
					class={codeInnerValueClass}
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

let wrapperClass = cssClass()
let wrapperCss = css({
	[`.${wrapperClass}`]: {
		marginRight: '0.5rem',
		textAlign: 'right',
		userSelect: 'none',
	},
})

let lineNumberClass = cssClass()
let lineNumberCss = css({
	[`.${lineNumberClass}`]: {
		color: syntaxColors.lineNumbers,
		fontSize: '1rem',
		lineHeight: '1.5',
	},
})

function LineNumbers(props: LineNumbersProps) {
	useCss(wrapperCss)
	useCss(lineNumberCss)
	return (
		<div class={wrapperClass}>
			{times(props.count, index => (
				<div class={lineNumberClass} key={index}>
					{index + 1}
				</div>
			))}
		</div>
	)
}

type InlineCodeProps = { children?: Children }

let containerClass = cssClass()
let containerCss = css({
	[`.${containerClass}`]: {
		backgroundColor: syntaxColors.background,
		borderRadius: variables.border.radius,
		color: syntaxColors.foreground,
		fontSize: '0.8rem',
		padding: '0.1rem 0.2rem',
	},
})

export function InlineCode(props: InlineCodeProps) {
	useCss(containerCss)
	return <span class={containerClass}>{props.children}</span>
}

function countLines(text: string) {
	// trim trailing newline
	let trimmedText =
		text[text.length - 1] === '\n' ? text.substr(0, text.length - 1) : text
	return trimmedText.split('\n').length
}
