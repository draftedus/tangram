import { ComponentChildren, h } from 'preact'

type FormTitleProps = { children?: ComponentChildren }

export function FormTitle(props: FormTitleProps) {
	return <legend class="form-title">{props.children}</legend>
}

type FormProps = {
	action?: string
	autoComplete?: string
	children?: ComponentChildren
	encType?: string
	id?: string
	onSubmit?: (event: Event) => void
	post?: boolean
}

export function Form(props: FormProps) {
	return (
		<form
			action={props.action}
			autoComplete={props.autoComplete}
			class="form"
			encType={props.encType}
			id={props.id}
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
