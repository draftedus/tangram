import { h, ui } from 'deps'

type ClassSelectProps = {
	class: string
	classes: string[]
}

export function ClassSelect(props: ClassSelectProps) {
	return (
		<div>
			<ui.Form>
				<ui.SelectField
					id="class-select"
					label="Select Class"
					name="class"
					options={props.classes}
					value={props.class}
				/>
				<noscript>
					<ui.Button>Submit</ui.Button>
				</noscript>
			</ui.Form>
		</div>
	)
}

export function bootClassSelectField() {
	let selectElements = document.querySelectorAll('#class-select')
	selectElements.forEach(selectElement => {
		if (!(selectElement instanceof HTMLSelectElement)) throw Error()
		selectElement.addEventListener('change', event => {
			if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
			let form = event.currentTarget.closest('form')
			if (!(form instanceof HTMLFormElement)) throw Error()
			form.submit()
		})
	})
}
