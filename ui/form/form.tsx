import { Children, css, h, useCss } from '../deps'

type FormTitleProps = { children?: Children }

let titleCss = css({
	[`.form-title`]: { textAlign: 'center' },
})

export function FormTitle(props: FormTitleProps) {
	useCss(titleCss)
	return <legend class="form-title">{props.children}</legend>
}

type FormProps = {
	action?: string
	children?: Children
	encType?: string
	onSubmit?: (event: Event) => void
	post?: boolean
}

let formCss = css({
	[`.form`]: {
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
			class="form"
			encType={props.encType}
			method={props.post ? 'post' : undefined}
			onSubmit={props.onSubmit}
		>
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
