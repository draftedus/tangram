import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export type Props = {
	inner: Inner
	pinwheelInfo: PinwheelInfo
}

enum InnerType {
	Auth = 'auth',
	NoAuth = 'no_auth',
}

type Inner =
	| {
			type: InnerType.Auth
			value: AuthProps
	  }
	| {
			type: InnerType.NoAuth
			value: NoAuthProps
	  }

type NoAuthProps = {
	repos: Array<{
		id: string
		title: string
	}>
}

type AuthProps = {
	email: string
	organizations: Array<{
		id: string
		name: string
	}>
	repos: Array<{
		id: string
		title: string
	}>
}

export default function UserPage(props: Props) {
	if (props.inner.type == InnerType.NoAuth) {
		return renderPage(
			<AppLayout pinwheelInfo={props.pinwheelInfo}>
				<ui.S1>
					<ui.H1>{'Root User'}</ui.H1>
					<ui.P>
						{
							'You are using the free version of tangram that does not support user accounts or organizations. Check out the different plans that allow you to collaborate with your team.'
						}
					</ui.P>
				</ui.S1>
			</AppLayout>,
		)
	} else if (props.inner.type === InnerType.Auth) {
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
							<ui.TextField
								label="Email"
								readOnly={true}
								value={props.inner.value.email}
							/>
						</ui.Form>
					</ui.S2>
					<ui.S2>
						<ui.SpaceBetween>
							<ui.H2>{'User Repos'}</ui.H2>
							<ui.Button href="/repos/new">{'Create New Repo'}</ui.Button>
						</ui.SpaceBetween>
						{props.inner.value.repos.length > 0 ? (
							<ui.Table width="100%">
								<ui.TableHeader>
									<ui.TableRow>
										<ui.TableHeaderCell>{'Repo Title'}</ui.TableHeaderCell>
									</ui.TableRow>
								</ui.TableHeader>
								<ui.TableBody>
									{props.inner.value.repos.map(repo => (
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
						{props.inner.value.organizations.length > 0 ? (
							<ui.Table width="100%">
								<ui.TableHeader>
									<ui.TableRow>
										<ui.TableHeaderCell>
											{'Organization Name'}
										</ui.TableHeaderCell>
									</ui.TableRow>
								</ui.TableHeader>
								<ui.TableBody>
									{props.inner.value.organizations.map(organization => (
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
}
