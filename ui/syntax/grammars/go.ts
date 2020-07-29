import { Scope, SyntaxGrammar } from '../types'

export let go: SyntaxGrammar = [
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
