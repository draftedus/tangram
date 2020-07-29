import { Scope, SyntaxGrammar } from '../types'

export let ruby: SyntaxGrammar = [
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
