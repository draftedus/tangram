import './styles.css'
import { PricingCards } from '../../../../../www/pages/pricing/pricing_cards'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AppLayout } from 'layouts/app_layout'
import { h } from 'preact'

export type Props = {
	card: {
		brand: string
		country: string
		expMonth: number
		expYear: number
		last4: string
		name: string
	} | null
	id: string
	members: Array<{
		email: string
		id: string
		isAdmin: boolean
	}>
	name: string
	pinwheelInfo: PinwheelInfo
	plan: Plan
	repos: Array<{
		id: string
		title: string
	}>
	stripePublishableKey: string
	userId: string
}

export enum Plan {
	Trial = 'trial',
	Free = 'free',
	Startup = 'startup',
	Team = 'team',
	Enterprise = 'enterprise',
}

export default function OrganizationIndexPage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<div
				data-stripe-publishable-key={props.stripePublishableKey}
				id="stripe-publishable-key"
				style="display: none"
			></div>
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
							<ui.P>{'Organization does not have any repos.'}</ui.P>
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
													name="memberId"
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
					<ui.H2>{'Plan'}</ui.H2>
					<ui.P>
						{'You are currently on the '}
						{props.plan}
						{' plan.'}
					</ui.P>
					<PricingCards
						enterpriseCta={
							<ui.Form post={true}>
								<input name="action" type="hidden" value="change_plan" />
								<input name="plan" type="hidden" value={Plan.Enterprise} />
								<ui.Button
									color="var(--blue)"
									disabled={props.plan === Plan.Enterprise}
									type="submit"
								>
									{props.plan === Plan.Enterprise ? 'Selected' : 'Select Plan'}
								</ui.Button>
							</ui.Form>
						}
						enterpriseSelected={props.plan === Plan.Enterprise}
						freeCta={
							<ui.Form post={true}>
								<input name="action" type="hidden" value="change_plan" />
								<input name="plan" type="hidden" value={Plan.Free} />
								<ui.Button
									color="var(--indigo)"
									disabled={props.plan === Plan.Free}
									type="submit"
								>
									{props.plan === Plan.Free ? 'Selected' : 'Select Plan'}
								</ui.Button>
							</ui.Form>
						}
						freeSelected={props.plan === Plan.Free}
						startupCta={
							<ui.Form post={true}>
								<input name="action" type="hidden" value="change_plan" />
								<input name="plan" type="hidden" value={Plan.Startup} />
								<ui.Button
									color="var(--teal)"
									disabled={props.plan === Plan.Startup}
									type="submit"
								>
									{props.plan === Plan.Startup ? 'Selected' : 'Select Plan'}
								</ui.Button>
							</ui.Form>
						}
						startupSelected={props.plan === Plan.Startup}
						teamCta={
							<ui.Form post={true}>
								<input name="action" type="hidden" value="change_plan" />
								<input name="plan" type="hidden" value={Plan.Team} />
								<ui.Button
									color="var(--green)"
									disabled={props.plan === Plan.Team}
									type="submit"
								>
									{props.plan === Plan.Team ? 'Selected' : 'Select Plan'}
								</ui.Button>
							</ui.Form>
						}
						teamSelected={props.plan === Plan.Team}
					/>
				</ui.S2>
				<ui.S2>
					<ui.SpaceBetween>
						<ui.H2>{'Billing'}</ui.H2>
						<ui.Button color="var(--blue)" id="set-payment-method-button">
							{'Set Payment Method'}
						</ui.Button>
					</ui.SpaceBetween>
					{props.card ? (
						<ui.Table width="100%">
							<ui.TableBody>
								<ui.TableRow>
									<ui.TableCell>{'Name'}</ui.TableCell>
									<ui.TableCell>{props.card.name}</ui.TableCell>
								</ui.TableRow>
								<ui.TableRow>
									<ui.TableCell>{'Brand'}</ui.TableCell>
									<ui.TableCell>{props.card.brand}</ui.TableCell>
								</ui.TableRow>
								<ui.TableRow>
									<ui.TableCell>{'Last 4 Digits'}</ui.TableCell>
									<ui.TableCell>{props.card.last4}</ui.TableCell>
								</ui.TableRow>
								<ui.TableRow>
									<ui.TableCell>{'Expiration Date'}</ui.TableCell>
									<ui.TableCell>
										{props.card.expMonth}
										{' / '}
										{props.card.expYear}
									</ui.TableCell>
								</ui.TableRow>
							</ui.TableBody>
						</ui.Table>
					) : (
						<ui.Card>
							<ui.P>{'You do not have a payment method.'}</ui.P>
						</ui.Card>
					)}
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
