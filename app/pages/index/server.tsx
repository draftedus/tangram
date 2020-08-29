import { Fragment, PinwheelInfo, h, renderPage, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export type Props = {
	pinwheelInfo: PinwheelInfo
	repos: Array<{
		createdAt: string
		id: string
		ownerName: string | null
		title: string
	}>
}

export default function HomePage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.SpaceBetween>
					<ui.H1>{'Repositories'}</ui.H1>
					<ui.Button href="/repos/new">{'Create Repo'}</ui.Button>
				</ui.SpaceBetween>
				{props.repos.length !== 0 ? (
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell expand={true}>{'Name'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Owner'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Created'}</ui.TableHeaderCell>
								<ui.TableHeaderCell></ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.repos.map(repo => (
								<ui.TableRow key={repo.id}>
									<ui.TableCell>
										<ui.Link href={`/repos/${repo.id}/`}>
											{repo.ownerName && (
												<>
													{repo.ownerName}
													{'/'}
												</>
											)}
											{repo.title}
										</ui.Link>
									</ui.TableCell>
									<ui.TableCell>{repo.ownerName}</ui.TableCell>
									<ui.TableCell>{repo.createdAt}</ui.TableCell>
									<ui.TableCell>
										<form method="post">
											<input name="action" type="hidden" value="delete_repo" />
											<input name="repoId" type="hidden" value={repo.id} />
											<ui.Button color="var(--red)">{'Delete'}</ui.Button>
										</form>
									</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.TableBody>
					</ui.Table>
				) : (
					<ui.Card>
						<ui.P>{'You do not have any repositories.'}</ui.P>
					</ui.Card>
				)}
			</ui.S1>
		</AppLayout>,
	)
}
