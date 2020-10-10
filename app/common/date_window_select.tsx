import { DateWindow } from './time'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

type DateWindowSelectFieldProps = {
	dateWindow: DateWindow
}

export function DateWindowSelectField(props: DateWindowSelectFieldProps) {
	return (
		<div>
			<ui.Form>
				<ui.SelectField
					id="date-window-select"
					label="Date Window"
					name="date_window"
					options={Object.values(DateWindow)}
					value={props.dateWindow}
				/>
				<noscript>
					<ui.Button>{'Submit'}</ui.Button>
				</noscript>
			</ui.Form>
		</div>
	)
}

export function bootDateWindowSelectField() {
	let selectElements = document.querySelectorAll('#date-window-select')
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
