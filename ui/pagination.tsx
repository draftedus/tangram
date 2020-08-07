import { Button } from './button'
import { Children, h } from './deps'
import { colors } from './theme'

export type PaginationProps = {
	children?: Children
	count?: number
	limit: number
	offset: number
	onNextPressed: () => void
	onPreviousPressed: () => void
}

export function Pagination(props: PaginationProps) {
	let pageString: string
	if (props.count === 0) {
		pageString = '0 of 0'
	} else if (!props.count) {
		let start = props.offset + 1
		let end = props.offset + props.limit
		pageString = `${start} - ${end}`
	} else {
		let start = props.offset + 1
		let end = Math.min(props.count, props.offset + props.limit)
		pageString = `${start} - ${end} of ${props.count}`
	}
	let prevButtonDisabled = props.offset === 0
	let nextButtonDisabled = props.count
		? props.offset + props.limit >= props.count
		: false
	return (
		<PaginationWrapper>
			<PaginationButton
				disabled={prevButtonDisabled}
				onClick={props.onPreviousPressed}
			>
				{'Previous'}
			</PaginationButton>
			<div>{pageString}</div>
			<PaginationButton
				disabled={nextButtonDisabled}
				onClick={props.onNextPressed}
			>
				{'Next'}
			</PaginationButton>
		</PaginationWrapper>
	)
}

type PaginationWrapperProps = { children?: Children }

function PaginationWrapper(props: PaginationWrapperProps) {
	return <div class="pagination">{props.children}</div>
}

type PaginationButtonProps = {
	children?: Children
	disabled?: boolean
	onClick: () => void
}

function PaginationButton(props: PaginationButtonProps) {
	return (
		<Button
			color={colors.blue}
			disabled={props.disabled}
			onClick={props.onClick}
		>
			<div class="pagination-button">{props.children}</div>
		</Button>
	)
}
