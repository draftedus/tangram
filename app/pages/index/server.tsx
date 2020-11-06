import { Props } from './props'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AppLayout } from 'layouts/app_layout'
import { Fragment, h } from 'preact'

export default (pageInfo: PageInfo, props: Props) => {
	if (Math.random() < 0.0) {
		throw Error('hello world')
	}
	return renderPage(
		<AppLayout info={props.appLayoutInfo} pageInfo={pageInfo}>
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
								<ui.TableHeaderCell>{'Created'}</ui.TableHeaderCell>
								<ui.TableHeaderCell></ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.Tablehyper::Body>
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
									<ui.TableCell>{repo.createdAt}</ui.TableCell>
									<ui.TableCell>
										<form method="post">
											<input name="action" type="hidden" value="delete_repo" />
											<input name="repo_id" type="hidden" value={repo.id} />
											<ui.Button color="var(--red)">{'Delete'}</ui.Button>
										</form>
									</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.Tablehyper::Body>
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
