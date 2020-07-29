import { Scope, SyntaxGrammar } from '../types'

export let javascript: SyntaxGrammar = [
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
