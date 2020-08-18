import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export type Props = {
	email: string
	organizations: Array<{
		id: string
		name: string
	}>
	pinwheelInfo: PinwheelInfo
	repos: Array<{
		id: string
		title: string
	}>
}

export default function UserPage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.SpaceBetween>
					<ui.H1>{'User'}</ui.H1>
					<ui.Form post={true}>
						<input name="action" type="hidden" value="logout" />
						<ui.Button color="var(--red)">{'Logout'}</ui.Button>
					</ui.Form>
				</ui.SpaceBetween>
				<ui.S2>
					<ui.Form>
						<ui.TextField label="Email" readOnly={true} value={props.email} />
					</ui.Form>
				</ui.S2>
				<ui.S2>
					<ui.SpaceBetween>
						<ui.H2>{'User Repos'}</ui.H2>
						<ui.Button href="/repos/new">{'Create New Repo'}</ui.Button>
					</ui.SpaceBetween>
					{props.repos.length > 0 ? (
						<ui.Table width="100%">
							<ui.TableHeader>
								<ui.TableRow>
									<ui.TableHeaderCell>{'Repo Title'}</ui.TableHeaderCell>
								</ui.TableRow>
							</ui.TableHeader>
							<ui.TableBody>
								{props.repos.map(repo => (
									<ui.TableRow key={repo.id}>
										<ui.TableCell>
											<ui.Link href={`/repos/${repo.id}/`}>
												{repo.title}
											</ui.Link>
										</ui.TableCell>
									</ui.TableRow>
								))}
							</ui.TableBody>
						</ui.Table>
					) : (
						<ui.Card>
							<ui.P>{'You do not have any repos.'}</ui.P>
						</ui.Card>
					)}
				</ui.S2>
				<ui.S2>
					<ui.SpaceBetween>
						<ui.H2>{'Organizations'}</ui.H2>
						<ui.Button href="/organizations/new">
							{'Create New Organization'}
						</ui.Button>
					</ui.SpaceBetween>
					{props.organizations.length > 0 ? (
						<ui.Table width="100%">
							<ui.TableHeader>
								<ui.TableRow>
									<ui.TableHeaderCell>{'Organization Name'}</ui.TableHeaderCell>
								</ui.TableRow>
							</ui.TableHeader>
							<ui.TableBody>
								{props.organizations.map(organization => (
									<ui.TableRow key={organization.id}>
										<ui.TableCell>
											<ui.Link href={`/organizations/${organization.id}/`}>
												{organization.name}
											</ui.Link>
										</ui.TableCell>
									</ui.TableRow>
								))}
							</ui.TableBody>
						</ui.Table>
					) : (
						<ui.Card>
							<ui.P>{'You do not have any organizations.'}</ui.P>
						</ui.Card>
					)}
				</ui.S2>
			</ui.S1>
		</AppLayout>,
	)
}
