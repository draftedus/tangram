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
				<ui.H1>{props.name}</ui.H1>
				<ui.S2>
					<ui.SpaceBetween>
						<ui.H2>{'Details'}</ui.H2>
						<ui.Button
							color="var(--gray)"
							href={`/organizations/${props.id}/edit`}
						>
							{'Edit'}
						</ui.Button>
					</ui.SpaceBetween>
					<ui.Form>
						<ui.TextField
							label="Organization Name"
							readOnly={true}
							value={props.name}
						/>
					</ui.Form>
				</ui.S2>
				<ui.S2>
					<ui.SpaceBetween>
						<ui.H2>{'Repos'}</ui.H2>
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
							<ui.P>{'This organization does not have any repos.'}</ui.P>
						</ui.Card>
					)}
				</ui.S2>
				<ui.S2>
					<ui.SpaceBetween>
						<ui.H2>{'Members'}</ui.H2>
						<ui.Button href={`/organizations/${props.id}/members/new`}>
							{'Invite Team Member'}
						</ui.Button>
					</ui.SpaceBetween>
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell>{'Email'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Role'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Remove'}</ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.members.map(member => (
								<ui.TableRow key={member.id}>
									<ui.TableCell expand={true}>{member.email}</ui.TableCell>
									<ui.TableCell>
										{member.isAdmin ? 'Admin' : 'Member'}
									</ui.TableCell>
									<ui.TableCell>
										{props.userId != member.id ? (
											<ui.Form post={true}>
												<input
													name="action"
													type="hidden"
													value="delete_member"
												/>
												<input
													name="member_id"
													type="hidden"
													value={member.id}
												/>
												<ui.Button color="var(--red)" type="submit">
													{'Remove'}
												</ui.Button>
											</ui.Form>
										) : null}
									</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.TableBody>
					</ui.Table>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Danger Zone'}</ui.H2>
					<ui.Form post={true}>
						<input name="action" type="hidden" value="delete_organization" />
						<ui.Button color="var(--red)">{'Delete Organization'}</ui.Button>
					</ui.Form>
				</ui.S2>
			</ui.S1>
		</AppLayout>,
	)
}
