export type SyntaxColors = {
	background: string
	foreground: string
	lineNumbers: string
} & { [S in Scope]: string }

export enum Scope {
	BuiltIn = 'builtin',
	Comment = 'comment',
	Function = 'function',
	Keyword = 'keyword',
	LiteralBool = 'literalBool',
	LiteralNumber = 'literalNumber',
	LiteralString = 'literalString',
}

export type SyntaxGrammar = SyntaxGrammarRule[]

type SyntaxGrammarRule = {
	regex: RegExp
	scope: Scope
}

export enum Language {
	Go = 'go',
	JavaScript = 'javascript',
	Python = 'python',
	Ruby = 'ruby',
	Text = 'text',
}

export type Replacement = {
	range: Range
	scope: Scope
}

export type Range = {
	end: number
	start: number
}
