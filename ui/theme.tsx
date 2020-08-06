import { SyntaxColors } from './syntax/mod'

export let desktop = '@media (min-width: 1200px)'
export let mobile = '@media (max-width: 1199px)'

export type Variables = {
	border: {
		radius: string
		width: string
	}
	colors: {
		accent: string
		background: string
		border: string
		funText: string
		header: string
		hover: string
		mutedText: string
		surface: string
		text: string
	}
	width: {
		max: string
		min: string
	}
}

export let variables: Variables = {
	border: {
		radius: '4px',
		width: '2px',
	},
	colors: {
		accent: 'var(--accent)',
		background: 'var(--background)',
		border: 'var(--border-color)',
		funText: 'var(--fun-text)',
		header: 'var(--header)',
		hover: 'var(--hover)',
		mutedText: 'var(--muted-text)',
		surface: 'var(--surface)',
		text: 'var(--text)',
	},
	width: {
		max: '1200px',
		min: '256px',
	},
}

export let border = `${variables.border.width} solid ${variables.colors.border}`

export let colors = {
	blue: '#0A84FF',
	gray: '#8E8E93',
	green: '#30D158',
	indigo: '#5e5ce6',
	orange: '#FF9F0A',
	pink: '#FF375F',
	purple: '#BF5AF2',
	red: '#FF453A',
	teal: '#4DD0E1',
	yellow: '#FFD60A',
}

export let eightColors = {
	black: '#000000',
	blue: '#0A84FF',
	cyan: '#4DD0E1',
	green: '#30D158',
	magenta: '#FF375F',
	red: '#FF453A',
	white: '#FFFFFF',
	yellow: '#FFD60A',
}

export let terminalColors = {
	background: variables.colors.header,
	mutedText: variables.colors.mutedText,
	text: variables.colors.text,
}

export let syntaxColors: SyntaxColors = {
	background: terminalColors.background,
	builtin: colors.indigo,
	comment: terminalColors.mutedText,
	foreground: terminalColors.text,
	function: colors.blue,
	keyword: colors.purple,
	lineNumbers: terminalColors.mutedText,
	literalBool: colors.orange,
	literalNumber: colors.yellow,
	literalString: colors.green,
}
