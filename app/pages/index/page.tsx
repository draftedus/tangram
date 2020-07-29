import { h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export type Props = {
	repos: Array<{
		createdAt: string
		id: string
		mainModelId: string
		ownerName: string
		title: string
	}>
}

export default function HomePage(props: Props) {
	return (
		<AppLayout>
			<ui.S1>
				<ui.SpaceBetween>
					<ui.H1>Repositories</ui.H1>
					<ui.Button href="/repos/new">Create Repo</ui.Button>
				</ui.SpaceBetween>
				{props.repos.length !== 0 ? (
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell expand>Name</ui.TableHeaderCell>
								<ui.TableHeaderCell>Owner</ui.TableHeaderCell>
								<ui.TableHeaderCell>Created</ui.TableHeaderCell>
								<ui.TableHeaderCell></ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.repos.map(repoItem => (
								<ui.TableRow key={repoItem.id}>
									<ui.TableCell>
										<ui.Link href={`/models/${repoItem.mainModelId}/`}>
											{repoItem.ownerName}/{repoItem.title}
										</ui.Link>
									</ui.TableCell>
									<ui.TableCell>{repoItem.ownerName}</ui.TableCell>
									<ui.TableCell>{repoItem.createdAt}</ui.TableCell>
									<ui.TableCell>
										<ui.Form>
											<ui.Button color={ui.colors.red}>Delete</ui.Button>
										</ui.Form>
									</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.TableBody>
					</ui.Table>
				) : (
					<ui.Card>
						<ui.P>You do not have any repositories.</ui.P>
					</ui.Card>
				)}
			</ui.S1>
		</AppLayout>
	)
}
