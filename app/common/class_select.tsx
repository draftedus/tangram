import * as ui from '@tangramhq/ui'
import { h } from 'preact'

type ClassSelectProps = {
	class: string
	classes: string[]
}

export function ClassSelectField(props: ClassSelectProps) {
	return (
		<ui.SelectField
			id="class-select"
			label="Select Class"
			name="class"
			options={props.classes.map(className => ({
				text: className,
				value: className,
			}))}
			value={props.class}
		/>
	)
}
