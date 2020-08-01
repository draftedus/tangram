import { Children, css, cx, h, useCss } from './deps'
import { border, variables } from './theme'

export type TableProps = {
	children?: Children
	width?: string
}

let tableWrapperCss = css({
	[`.table-wrapper`]: { overflow: 'auto' },
})

let tableCss = css({
	[`.table`]: {
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
		<div class="table-wrapper">
			<table class="table" style={{ width: props.width ?? 'auto' }}>
				{props.children}
			</table>
		</div>
	)
}

export type TableHeaderProps = {
	children?: Children
}

let tableHeaderCss = css({
	[`.table-header`]: {
		borderTopLeftRadius: variables.border.radius,
		borderTopRightRadius: variables.border.radius,
	},
})

export function TableHeader(props: TableHeaderProps) {
	useCss(tableHeaderCss)
	return <thead class="table-header">{props.children}</thead>
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

let tableHeaderCellCss = css({
	[`.table-header-cell`]: {
		backgroundColor: variables.colors.header,
		borderBottom: border,
		color: variables.colors.text,
		fontWeight: 'normal',
		padding: '0.5rem 1rem',
		whiteSpace: 'nowrap',
	},
})

let alignLeftCss = css({
	[`.table-align-left`]: { textAlign: 'left' },
})

let alignRightCss = css({
	[`.table-align-right`]: { textAlign: 'right' },
})

let alignCenterCss = css({
	[`.table-align-center`]: { textAlign: 'center' },
})

let expandCss = css({
	[`.table-expand`]: { width: '100%' },
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
			alignClass = 'table-align-left'
			break
		}
		case 'right': {
			alignClass = 'table-align-right'
			break
		}
		case 'center': {
			alignClass = 'table-align-center'
			break
		}
	}
	return (
		<th
			class={cx(
				'table-header-cell',
				props.textAlign ? alignClass : 'table-align-left',
				props.expand && 'table-expand',
			)}
		>
			{props.children}
		</th>
	)
}

let tableCellCss = css({
	[`.table-cell`]: {
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
		<td class="table-cell" style={style}>
			{props.children}
		</td>
	)
}
