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
					label="Select Class"
					name="class"
					options={props.classes}
					value={props.class}
				/>
			</ui.Form>
		</div>
	)
}
