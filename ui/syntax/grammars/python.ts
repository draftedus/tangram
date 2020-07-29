import { Scope, SyntaxGrammar } from '../types'

export let python: SyntaxGrammar = [
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
