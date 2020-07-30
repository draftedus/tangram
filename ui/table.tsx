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
	useCss(tableWrapperCss, tableCss)
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
	children?: Children
	colSpan?: string
	color?: string
	expand?: boolean
	textAlign?: 'left' | 'center' | 'right' | null
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

let alignLeftClass = cssClass()
let alignLeftCss = css({
	[`.${alignLeftClass}`]: { textAlign: 'left' },
})

let alignRightClass = cssClass()
let alignRightCss = css({
	[`.${alignRightClass}`]: { textAlign: 'right' },
})

let alignCenterClass = cssClass()
let alignCenterCss = css({
	[`.${alignCenterClass}`]: { textAlign: 'center' },
})

let expandClass = cssClass()
let expandCss = css({
	[`.${expandClass}`]: { width: '100%' },
})

export function TableHeaderCell(props: TableCellProps) {
	useCss(
		tableHeaderCellCss,
		expandCss,
		expandCss,
		alignCenterCss,
		alignRightCss,
		alignLeftCss,
	)
	let alignClass
	switch (props.textAlign) {
		case 'left': {
			alignClass = alignLeftClass
			break
		}
		case 'right': {
			alignClass = alignRightClass
			break
		}
		case 'center': {
			alignClass = alignCenterClass
			break
		}
	}
	return (
		<th
			class={cx(
				tableHeaderCellClass,
				props.textAlign ? alignClass : alignLeftClass,
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
