import { CodeFix } from './code_fix'
import { LogModelInit } from './log_model_init'
import { LogPrediction } from './log_prediction'
import { LogPredictionBatched } from './log_prediction_batched'
import { LogTrueValue } from './log_true_value'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { DocsLayout, DocsPage } from 'layouts/docs_layout'
import { h } from 'preact'

import featureImportances from './screenshots/feature_importances.png'
import generateToken from './screenshots/generate_token.png'
import generateTokenAlert from './screenshots/generate_token_alert.png'
import productionColumnStatsUniqueValues from './screenshots/production_column_stats_unique_values.png'
import productionColumnStatsUnknownValues from './screenshots/production_column_stats_unknown_values.png'
import productionMetricsOverview from './screenshots/production_metrics_overview.png'
import productionStatsColumnStats from './screenshots/production_stats_column_stats.png'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default (props: Props) =>
	renderPage(
		<DocsLayout
			pinwheelInfo={props.pinwheelInfo}
			selectedPage={DocsPage.Monitor}
		>
			<ui.S1>
				<ui.H1>{'Monitor'}</ui.H1>
				<ui.S2>
					<ui.P>
						{
							'Once our model is deployed, we want to make sure that it performs as well in production as it did in training. Tangram makes this easy by providing an API to log predictions and true values so you can view production stats and metrics.'
						}
					</ui.P>
					<ui.P>
						{
							'First, we need to get an API token from the reporting and monitoring app. In the topbar, click '
						}
						<b>{'Tokens'}</b>
						{'.'}
					</ui.P>
					<ui.P>{'Then, follow the steps to generate a new token.'}</ui.P>
					<ui.Img alt="generate token" src={generateToken} />
					<ui.P>{'Copy the token value and keep it somewhere safe.'}</ui.P>
					<ui.Img alt="generate token alert" src={generateTokenAlert} />
					<ui.P>{'Pass the API token when you initialize your model.'}</ui.P>
					<LogModelInit />
					<ui.P>{'Log each prediction after you make it.'}</ui.P>
					<LogPrediction />
					<ui.P>
						{
							'Later on, we may get official diagnoses for patients. In our code, we can log true values.'
						}
					</ui.P>
					<LogTrueValue />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Batching'}</ui.H2>
					<ui.P>
						{
							'For simplicity, the default logging functions make a network request each time they are called. In most applications, you will want to batch these calls to reduce latency along your critical path. To do so, you can call the batched logging functions with the same arguments as their non-batched alternatives. When your application is ready to send all logs to the server in a single request, you can flush the queue.'
						}
					</ui.P>
					<LogPredictionBatched />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Production Stats'}</ui.H2>
					<ui.P>
						{
							'Under **Production Stats**, we can compare production stats with training stats.'
						}
					</ui.P>
					<ui.Img alt="column stats" src={productionStatsColumnStats} />
					<ui.P>
						{'Here, we see that the '}
						<ui.InlineCode>{'chest_pain'}</ui.InlineCode>
						{
							" column has an alert and a high unknown values count. Let's select it."
						}
					</ui.P>
					<ui.Img
						alt="unknown values"
						src={productionColumnStatsUnknownValues}
					/>
					<ui.P>
						{
							'In the unknown values table, we see that there are a large number of inputs with the unknown value '
						}
						<ui.InlineCode>{'asx'}</ui.InlineCode>
						{'.'}
					</ui.P>
					<ui.Img alt="unique values" src={productionColumnStatsUniqueValues} />
					<ui.P>
						<ui.InlineCode>{'asx'}</ui.InlineCode>
						{' is a common medical abbreviation for '}
						<ui.InlineCode>{'asymptomatic'}</ui.InlineCode>
						{' so it looks like'}
						<ui.InlineCode>{'asymptomatic'}</ui.InlineCode>
						{' is mislabeled as '}
						<ui.InlineCode>{'asx'}</ui.InlineCode>
						{' in production. We can fix this by mapping '}
						<ui.InlineCode>{'asx'}</ui.InlineCode>
						{' to '}
						<ui.InlineCode>{'asymptomatic'}</ui.InlineCode>
						{' in our code. That fix could look something like this:'}
					</ui.P>
					<CodeFix />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Production Metrics'}</ui.H2>
					<ui.P>
						{'Under '}
						<b>{'Production Metrics'}</b>
						{
							'  we can ensure our model is as accurate in production as it was in training. We can view accuracy, precision, recall and compare each of these metrics with their values in training.'
						}
					</ui.P>
					<ui.Img
						alt="production metrics overview"
						src={productionMetricsOverview}
					/>
					<ui.P>
						{
							'In this case, it looks like our production accuracy is a bit lower than we expected. This could be due to the mislabeling of  '
						}
						<ui.InlineCode>{'asymptomatic'}</ui.InlineCode>
						{' as '}
						<ui.InlineCode>{'asx'}</ui.InlineCode>
						{
							". To support this theory, let's see how important this feature was to our model."
						}
					</ui.P>
					<ui.P>
						{'Under '}
						<b>{'Model'}</b>
						{', we can view feature and column importances. The'}
						<ui.InlineCode>{'chest_pain=asymptomatic'}</ui.InlineCode>
						{
							'  feature is by far the most important to our model, so it makes sense that mislabeling '
						}
						<ui.InlineCode>{'asymptomatic'}</ui.InlineCode>
						{' as '}
						<ui.InlineCode>{'asx'}</ui.InlineCode>
						{'  would cause a drop in accuracy.'}
					</ui.P>
					<ui.Img alt="feature importances" src={featureImportances} />
					<ui.P>
						{
							"We're done! We just trained and deployed a custom machine learning model using the tangram cli and language libraries. Then, we learned more about our model and debugged an issue in production using the reporting and monitoring app."
						}
					</ui.P>
					<ui.P>
						{
							"You're now ready to train and deploy your first model. If you run into any issues or want some hands-on help getting set up, send us an email at help@tangramhq.com or use the chat bubble on this page."
						}
					</ui.P>
				</ui.S2>
			</ui.S1>
		</DocsLayout>,
	)
