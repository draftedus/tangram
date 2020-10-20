import { Props } from './props'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AppLayout } from 'layouts/app_layout'
import { h } from 'preact'

export default function RepoCreatePage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Create New Repo'}</ui.H1>
				<ui.Form encType="multipart/form-data" post={true}>
					{props.error && (
						<ui.Alert level={ui.Level.Danger}>{props.error}</ui.Alert>
					)}
					<ui.TextField
						label="Title"
						name="title"
						required={true}
						value={props.title}
					/>
					{props.owners && (
						<ui.SelectField
							label="Owner"
							name="owner"
							required={true}
							value={props.owner}
						>
							{props.owners.map(({ title, value }) => (
								<option key={value} value={value}>
									{title}
								</option>
							))}
						</ui.SelectField>
					)}
					<ui.FileField label="File" name="file" required={true} />
					<ui.Button type="submit">{'Upload'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
