import { Children, Fragment, h, useEffect } from './deps'
import { Language, SyntaxColors, grammars, highlight } from './syntax/mod'
import { times } from './util'

let syntaxColors: SyntaxColors = {
	background: 'var(--header-color)',
	builtin: 'var(--indigo)',
	comment: 'var(--muted-text-color)',
	foreground: 'var(--text-color)',
	function: 'var(--blue)',
	keyword: 'var(--purple)',
	lineNumbers: 'var(--muted-text-color)',
	literalBool: 'var(--orange)',
	literalNumber: 'var(--yellow)',
	literalString: 'var(--green)',
}

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

export function Code(props: CodeProps) {
	return (
		<div class="code">
			<div class="code-inner-wrapper" style={{ display: 'grid' }}>
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

export function CodeSelect(props: CodeSelectProps) {
	let name = Math.random().toString()
	useEffect(() => {
		bootCodeSelect()
	})
	return (
		<div class="code code-grid">
			{props.languages && (
				<>
					<CodeOption
						checked={true}
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
				</>
			)}
		</div>
	)
}

export function bootCodeSelect() {
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
}

type CodeOptionProps = {
	checked?: boolean
	code: string
	hideLineNumbers?: boolean
	language: Language
	name: string
}

function CodeOption(props: CodeOptionProps) {
	let optionId = Math.random().toString()
	return (
		<>
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
		</>
	)
}

type LineNumbersProps = {
	count: number
}

function LineNumbers(props: LineNumbersProps) {
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

export function InlineCode(props: InlineCodeProps) {
	return <span class="inline-code-wrapper">{props.children}</span>
}

function countLines(text: string) {
	// trim trailing newline
	let trimmedText =
		text[text.length - 1] === '\n' ? text.substr(0, text.length - 1) : text
	return trimmedText.split('\n').length
}
