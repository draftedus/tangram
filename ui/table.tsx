import './table.css'
import { cx } from './util'
import { ComponentChildren, h } from 'preact'

export type TableProps = {
	children?: ComponentChildren
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
	children?: ComponentChildren
}

export function TableHeader(props: TableHeaderProps) {
	return <thead class="table-header">{props.children}</thead>
}

export type TableBodyProps = {
	children?: ComponentChildren
}

export function TableBody(props: TableBodyProps) {
	return <tbody>{props.children}</tbody>
}

export type TableRowProps = {
	children?: ComponentChildren
}

export function TableRow(props: TableRowProps) {
	return <tr>{props.children}</tr>
}

export type TableCellProps = {
	children?: ComponentChildren
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
