import { Children, css, cssClass, cx, h, useCss } from './deps'
import { border, variables } from './theme'

export type TableProps = {
	children?: Children
	width?: string
}

let tableWrapperClass = cssClass()
let tableWrapperCss = css({
	[`.${tableWrapperClass}`]: { overflow: 'auto' },
})

let tableClass = cssClass()
let tableCss = css({
	[`.${tableClass}`]: {
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		borderSpacing: '0',
		margin: '0',
	},
})

export function Table(props: TableProps) {
	useCss(tableWrapperCss)
	useCss(tableCss)
	return (
		<div class={tableWrapperClass}>
			<table class={tableClass} style={{ width: props.width ?? 'auto' }}>
				{props.children}
			</table>
		</div>
	)
}

export type TableHeaderProps = {
	children?: Children
}

let tableHeaderClass = cssClass()
let tableHeaderCss = css({
	[`.${tableHeaderClass}`]: {
		borderTopLeftRadius: variables.border.radius,
		borderTopRightRadius: variables.border.radius,
	},
})

export function TableHeader(props: TableHeaderProps) {
	useCss(tableHeaderCss)
	return <thead class={tableHeaderClass}>{props.children}</thead>
}

export type TableBodyProps = {
	children?: Children
}

export function TableBody(props: TableBodyProps) {
	return <tbody>{props.children}</tbody>
}

export type TableRowProps = {
	children?: Children
}

export function TableRow(props: TableRowProps) {
	return <tr>{props.children}</tr>
}

export type TableCellProps = {
	align?: 'left' | 'center' | 'right' | null
	children?: Children
	colSpan?: string
	color?: string
	expand?: boolean
}

let tableHeaderCellClass = cssClass()
let tableHeaderCellCss = css({
	[`.${tableHeaderCellClass}`]: {
		backgroundColor: variables.colors.header,
		borderBottom: border,
		color: variables.colors.text,
		fontWeight: 'normal',
		padding: '0.5rem 1rem',
		whiteSpace: 'nowrap',
	},
})

let alignClass = cssClass()
let alignCss = css({
	[`.${alignClass}`]: { textAlign: 'left' },
})

let expandClass = cssClass()
let expandCss = css({
	[`.${expandClass}`]: { width: '100%' },
})

export function TableHeaderCell(props: TableCellProps) {
	useCss(tableHeaderCellCss)
	useCss(alignCss)
	useCss(expandCss)
	return (
		<th
			class={cx(
				tableHeaderCellClass,
				props.align ? alignClass : undefined,
				props.expand && expandClass,
			)}
		>
			{props.children}
		</th>
	)
}

let tableCellClass = cssClass()
let tableCellCss = css({
	[`.${tableCellClass}`]: {
		padding: '1rem 1rem',
		whiteSpace: 'nowrap',
		width: undefined,
	},
})

export function TableCell(props: TableCellProps) {
	useCss(tableCellCss)
	let style = {
		backgroundColor: props.color as any,
	}
	return (
		<td class={tableCellClass} style={style}>
			{props.children}
		</td>
	)
}
