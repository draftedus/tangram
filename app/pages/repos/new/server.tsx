import { Props } from './props'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AppLayout } from 'layouts/app_layout'
import { h } from 'preact'

export default (pageInfo: PageInfo, props: Props) => {
	return renderPage(
		<AppLayout info={props.appLayoutInfo} pageInfo={pageInfo}>
			<ui.S1>
				<ui.H1>{'Create New Repo'}</ui.H1>
				<ui.Form post={true}>
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
					<ui.Button type="submit">{'Submit'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
