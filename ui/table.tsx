import { Children, cx, h } from './deps'

export type TableProps = {
	children?: Children
	width?: string
}

export function Table(props: TableProps) {
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

export function TableHeader(props: TableHeaderProps) {
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

export function TableHeaderCell(props: TableCellProps) {
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

export function TableCell(props: TableCellProps) {
	let style = {
		backgroundColor: props.color as any,
	}
	return (
		<td class="table-cell" style={style}>
			{props.children}
		</td>
	)
}
