import { h, ui } from 'deps'
import { DocsLayout } from 'layouts/docs_layout'

export default function OnPrem() {
	return (
		<DocsLayout>
			<ui.S1>
				<ui.H1>On-Prem Install Guide</ui.H1>
				<ui.S2>
					<ui.P>
						Thanks for trying Tangram! In this tutorial we will show you how to
						install the Tangram reporting and monitoring app on your own server.
					</ui.P>
					<ui.P>
						Tangram can run on any machine that supports Docker. After
						installing Docker, log in to the Tangram container registry using
						the credentials provided to you by the Tangram team.
					</ui.P>
					<ui.Code
						code="$ docker login registry.tangramhq.com"
						hideLineNumbers
					></ui.Code>
					<ui.P>
						Create a Docker network for the services to communicate with each
						other:
					</ui.P>
					<ui.Code
						code="$ docker network create tangram"
						hideLineNumbers
					></ui.Code>
					<ui.P>
						The reporting and monitoring app is backed by a postgres database.
						You can use one you already have, or create a basic one that runs in
						Docker with the command below.
					</ui.P>
					<ui.Code code={postgresCode} hideLineNumbers></ui.Code>
					<ui.P>
						Next, run the `app` and `api` containers. If you are running
						postgres elsewhere, replace the `DATABASE_URL` environment variable
						accordingly. If you want to connect to your database with SSL, you
						can pass the certificate to the `api` using the `DATABASE_CERT`
						environment variable.
					</ui.P>
					<ui.Code code={apiAppCode} hideLineNumbers></ui.Code>
					<ui.P>
						That's it! You should now be able to access the reporting and
						monitoring app at http://localhost:8000. To keep your data secure,
						make sure to run the application behind a VPN.
					</ui.P>
					<ui.P>
						If you have any questions, get in touch with our team at{' '}
						<ui.Link href="mailto:help@tangramhq.com">
							help@tangramhq.com
						</ui.Link>
						.
					</ui.P>
				</ui.S2>
			</ui.S1>{' '}
		</DocsLayout>
	)
}

let postgresCode = `docker run \\
  -d \\
  --name tangram-db \\
  --network tangram \\
  postgres`

let apiAppCode = `docker run \\
  -d \\
  --name tangram-api \\
  --restart unless-stopped \\
  --network tangram \\
  -e DATABASE_URL="postgres://postgres@tangram-db:5432/postgres" \\
  -e PORT="8001" \\
  registry.tangramhq.com/api:latest
docker run \\
  -d \\
  --name tangram-app \\
  --restart unless-stopped \\
  --network tangram \\
  --publish 8000:8000 \\
  -e API_URL="http://tangram-api:8001" \\
  -e PORT="8000" \\
  registry.tangramhq.com/app:latest`
