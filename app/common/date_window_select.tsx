import { DateWindow } from './time'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

type DateWindowSelectFieldProps = {
	dateWindow: DateWindow
}

let dateWindowSelectFieldOptions = [
	{ text: 'Today', value: 'today' },
	{ text: 'This Month', value: 'this_month' },
	{ text: 'This Year', value: 'this_year' },
]

export function DateWindowSelectField(props: DateWindowSelectFieldProps) {
	return (
		<ui.SelectField
			id="date-window-select-field"
			label="Date Window"
			name="date_window"
			options={dateWindowSelectFieldOptions}
			value={props.dateWindow}
		/>
	)
}
