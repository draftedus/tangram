import { Children, css, cssClass, h, useCss } from '../deps'

type FormTitleProps = { children?: Children }

let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: { textAlign: 'center' },
})

export function FormTitle(props: FormTitleProps) {
	useCss(titleCss)
	return <legend class={titleClass}>{props.children}</legend>
}

type FormProps = {
	action?: string
	children?: Children
	directive?: string
	encType?: string
	onSubmit?: (event: Event) => void
	post?: boolean
}

let formClass = cssClass()
let formCss = css({
	[`.${formClass}`]: {
		display: 'grid',
		gridRowGap: '1rem',
		margin: '0',
	},
})

export function Form(props: FormProps) {
	useCss(formCss)
	return (
		<form
			action={props.action}
			class={formClass}
			encType={props.encType}
			method={props.post ? 'post' : undefined}
			onSubmit={props.onSubmit}
		>
			{props.directive && (
				<input name="action" style="display: none" value={props.directive} />
			)}
			{props.children}
		</form>
	)
}

export type FormFieldProps<T> = {
	disabled?: boolean
	id?: string
	label?: string
	name?: string
	onChange?: (newValue: T | null) => void
	value: T | null
}
