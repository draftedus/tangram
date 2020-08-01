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
				<noscript>
					<ui.Button>Submit</ui.Button>
				</noscript>
			</ui.Form>
		</div>
	)
}
