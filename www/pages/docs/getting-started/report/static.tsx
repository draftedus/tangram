import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { DocsLayout, DocsPage } from 'layouts/docs_layout'
import { h } from 'preact'

import overviewImageURL from './screenshots/overview.png'
import trainingMetricsConfusionMatrixImageURL from './screenshots/training_metrics_confusion_matrix.png'
import trainingMetricsOverviewImageURL from './screenshots/training_metrics_overview.png'
import trainingMetricsPrCurveImageURL from './screenshots/training_metrics_pr_curve.png'
import trainingStatsAgeImageURL from './screenshots/training_stats_age.png'
import trainingStatsChestPainImageURL from './screenshots/training_stats_chest_pain.png'
import trainingStatsHistogramImageURL from './screenshots/training_stats_histogram.png'
import trainingStatsOverviewImageURL from './screenshots/training_stats_overview.png'
import uploadImageURL from './screenshots/upload.png'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default (props: Props) =>
	renderPage(
		<DocsLayout
			pinwheelInfo={props.pinwheelInfo}
			selectedPage={DocsPage.Report}
		>
			<ui.S1>
				<ui.H1>{'Getting Started'}</ui.H1>
				<ui.S2>
					<ui.H2>{'Report'}</ui.H2>
					<ui.P>
						{"Let's learn more about our model by uploading the "}
						<ui.InlineCode>{'heart-disease.tangram'}</ui.InlineCode>
						{
							' file to the reporting and monitoring app. The app can be run by us in the cloud or by you on your own servers. To try the cloud hosted version, go to '
						}
						<ui.Link href="https://app.tangramhq.com/login">
							{'https://app.tangramhq.com/login'}
						</ui.Link>
						{
							', create an account and log in. Then click "Upload Model", select the model file, and upload it.'
						}
					</ui.P>
					<ui.Img alt="upload" src={uploadImageURL} />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Overview'}</ui.H2>
					<ui.P>
						{
							"The overview shows that Tangram used 80% of the data to train our model and reserved 20% of the data to test it. The model's accuracy is 80.33%, which means it made the correct prediction on 80.33% of the test set."
						}
					</ui.P>
					<ui.Img alt="overview" src={overviewImageURL} />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Training Stats'}</ui.H2>
					<ui.P>
						{'Under '}
						<b>{'Training Stats'}</b>
						{' we can learn more about our dataset.'}
					</ui.P>
					<ui.P>
						{'For '}
						<ui.InlineCode>{'Enum'}</ui.InlineCode>
						{
							' columns, we see how many unique values were found, how many rows had null values and a histogram showing the distribution of each enum value.'
						}
					</ui.P>
					<ui.Img
						alt="chest pain column"
						src={trainingStatsChestPainImageURL}
					/>
					<ui.P>
						{'For '}
						<ui.InlineCode>{'Number'}</ui.InlineCode>
						{
							' columns, we see statistics including min, max, mean, and standard deviation. We can also get a sense of the distribution of values by viewing a box plot showing the quantiles.'
						}
					</ui.P>
					<ui.Img alt="age column" src={trainingStatsAgeImageURL} />
					<ui.P>
						{"Let's select the "}
						<ui.InlineCode>{'exercise_max_heart_rate'}</ui.InlineCode>
						{
							' column and look at the histogram. It is a good idea to check the dataset to make sure the distribution of the data matches our expectations.'
						}
					</ui.P>
					<ui.Img alt="overview" src={trainingStatsOverviewImageURL} />
					<ui.Img alt="histogram" src={trainingStatsHistogramImageURL} />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Training Metrics'}</ui.H2>
					<ui.P>
						{'Under '}
						<b>{'Training Metrics'}</b>
						{
							', we can see how the model performed on the test set. We can review per-class metrics like precision and recall, confusion matrices and PR and ROC curves.'
						}
					</ui.P>
					<ui.Img
						alt="metrics overview"
						src={trainingMetricsOverviewImageURL}
					/>
					<ui.Img
						alt="precision_recall curve"
						src={trainingMetricsPrCurveImageURL}
					/>
					<ui.P>
						{
							'Select the "Class Metrics" tab and scroll down to view the confusion matrix.'
						}
					</ui.P>
					<ui.Img
						alt="confusion matrix"
						src={trainingMetricsConfusionMatrixImageURL}
					/>
					<ui.P>
						{
							"It looks like false negatives are a bit high. This means we are predicting people are healthy when they actually aren't. It would be better if the model had fewer false negatives, even if it means more false positives. Let's do that by going to the "
						}
						<b>{'Tuning'}</b>
						{' page.'}
					</ui.P>
				</ui.S2>
			</ui.S1>
		</DocsLayout>,
	)
