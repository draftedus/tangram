export enum Language {
	Go = "go",
	JavaScript = "javascript",
	Python = "python",
	Ruby = "ruby",
	Text = "text",
}

export function highlight(
	text: string,
	grammar: SyntaxGrammar,
	colors: SyntaxColors,
): string {
	// Replacements stores the range and scope for each part of the text to highlight.
	let replacements: Replacement[] = []
	// Loop over each rule in the grammar.
	let match
	for (let { regex, scope } of grammar) {
		// Loop over each match of the rule in the text.
		while ((match = regex.exec(text)) !== null) {
			// This is the range of the match in the text.
			let range = {
				end: match.index + match[0].length,
				start: match.index,
			}
			// Only use this match if it doesn't overlap with any of the existing replacements.
			let foundOverlap = false
			for (let { range: existingRange } of replacements) {
				if (
					(range.start <= existingRange.start &&
						range.end >= existingRange.start) ||
					(existingRange.start <= range.start &&
						existingRange.end >= range.start)
				) {
					foundOverlap = true
					break
				}
			}
			if (!foundOverlap) {
				replacements.push({
					range,
					scope,
				})
			}
		}
	}
	// Sort the replacements by start index.
	replacements.sort((a, b) => (a.range.start <= b.range.start ? -1 : 1))
	// Apply the replacements to the text.
	let html = text
	let offset = 0
	for (let { range, scope } of replacements) {
		let replacementText = text.substring(range.start, range.end)
		let replacementHtml = `<span style="color: ${colors[scope]}">${replacementText}</span>`
		html =
			html.substring(0, offset + range.start) +
			replacementHtml +
			html.substring(offset + range.end)
		offset += replacementHtml.length - (range.end - range.start)
	}
	return html
}

export type SyntaxColors = {
	background: string
	foreground: string
	lineNumbers: string
} & { [S in Scope]: string }

enum Scope {
	BuiltIn = "builtin",
	Comment = "comment",
	Function = "function",
	Keyword = "keyword",
	LiteralBool = "literalBool",
	LiteralNumber = "literalNumber",
	LiteralString = "literalString",
}

type SyntaxGrammar = SyntaxGrammarRule[]

type SyntaxGrammarRule = {
	regex: RegExp
	scope: Scope
}

type Replacement = {
	range: Range
	scope: Scope
}

type Range = {
	end: number
	start: number
}

let go: SyntaxGrammar = [
	{
		regex: /(\/\/.*$)|(\/\*.*\*\/)/gm,
		scope: Scope.Comment,
	},
	{
		regex: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/g,
		scope: Scope.LiteralString,
	},
	{
		regex: /\b[0-9]+(\.[0-9]*)?\b/g,
		scope: Scope.LiteralNumber,
	},
	{
		regex: /\b(?:_|iota|nil|true|false)\b/g,
		scope: Scope.LiteralBool,
	},
	{
		regex: /\b(?:bool|byte|complex(?:64|128)|error|float(?:32|64)|rune|string|u?int(?:8|16|32|64)?|uintptr|append|cap|close|complex|copy|delete|imag|len|make|new|panic|print(?:ln)?|real|recover)\b/g,
		scope: Scope.BuiltIn,
	},
	{
		regex: /\b(?:break|case|chan|const|continue|default|defer|else|fallthrough|for|func|go(?:to)?|if|import|interface|map|package|range|return|select|struct|switch|type|var)\b/g,
		scope: Scope.Keyword,
	},
]

let javascript: SyntaxGrammar = [
	{
		regex: /(\/\/.*$)|(\/\*.*\*\/)/gm,
		scope: Scope.Comment,
	},
	{
		regex: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/g,
		scope: Scope.LiteralString,
	},
	{
		regex: /\b[0-9]+(\.[0-9]*)?\b/g,
		scope: Scope.LiteralNumber,
	},
	{
		regex: /\b(break|case|catch|class|const|continue|debugger|default|delete|do|else|export|extends|finally|for|function|if|import|in|instanceof|new|return|super|switch|this|throw|try|typeof|var|void|while|with|yield|enum|implements|interface|let|package|private|protected|public|static|await)\b/g,
		scope: Scope.Keyword,
	},
]

let python: SyntaxGrammar = [
	{
		regex: /#.*$/gm,
		scope: Scope.Comment,
	},
	{
		regex: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/g,
		scope: Scope.LiteralString,
	},
	{
		regex: /\b[0-9]+(\.[0-9]*)?\b/g,
		scope: Scope.LiteralNumber,
	},
	{
		regex: /\b(and|as|assert|async|await|break|class|continue|def|del|elif|else|except|exec|finally|for|from|global|if|import|in|is|lambda|nonlocal|not|or|pass|print|raise|return|try|while|with|yield)\b/g,
		scope: Scope.Keyword,
	},
]

let ruby: SyntaxGrammar = [
	{
		regex: /#.*$/gm,
		scope: Scope.Comment,
	},
	{
		regex: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/g,
		scope: Scope.LiteralString,
	},
	{
		regex: /\b[0-9]+(\.[0-9]*)?\b/g,
		scope: Scope.LiteralNumber,
	},
	{
		regex: /\b(?:Array|Bignum|Binding|Class|Continuation|Dir|Exception|FalseClass|File|Stat|Fixnum|Float|Hash|Integer|IO|MatchData|Method|Module|NilClass|Numeric|Object|Proc|Range|Regexp|String|Struct|TMS|Symbol|ThreadGroup|Thread|Time|TrueClass)\b/g,
		scope: Scope.BuiltIn,
	},
	{
		regex: /\b(?:alias|and|BEGIN|begin|break|case|class|def|define_method|defined|do|each|else|elsif|END|end|ensure|extend|for|if|in|include|module|new|next|nil|not|or|prepend|protected|private|public|raise|redo|require|rescue|retry|return|self|super|then|throw|undef|unless|until|when|while|yield)\b/g,
		scope: Scope.Keyword,
	},
]

export let grammars = {
	go,
	javascript,
	python,
	ruby,
	text: [],
}
