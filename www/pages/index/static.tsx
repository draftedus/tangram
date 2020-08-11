import { Explanations } from './explanations'
import explanations from './explanations.svg'
import { Introspection } from './introspection'
import introspection from './introspection.svg'
import { Prediction } from './prediction'
import prediction from './prediction.svg'
import { ProductionMetrics } from './production_metrics'
import productionMetrics from './production_metrics.svg'
import { ProductionStats } from './production_stats'
import productionStats from './production_stats.svg'
import { Training } from './training'
import training from './training.svg'
import { TrainingMetrics } from './training_metrics'
import trainingMetrics from './training_metrics.svg'
import { TrainingStats } from './training_stats'
import trainingStats from './training_stats.svg'
import { Tuning } from './tuning'
import tuning from './tuning.svg'
import { Client, PinwheelInfo, h, renderPage, ui } from 'deps'
import { PageLayout } from 'layouts/page_layout'

type Props = {
	info: PinwheelInfo
}

export default function Home(props: Props) {
	return renderPage(
		<PageLayout background={true} info={props.info}>
			<ui.S1>
				<Hero />
				<Features />
			</ui.S1>
		</PageLayout>,
	)
}

function Hero() {
	return (
		<div class="index-wrapper">
			<div class="index-inner">
				<h1 class="index-title">{'Machine Learning for Developers'}</h1>
				<div class="index-subtitle">
					{
						'Train a model on the command line and make predictions in any programming language. Understand how your model works, tune it, and monitor it in production.'
					}
				</div>
				<Video />
			</div>
		</div>
	)
}

function Video() {
	let videoStyle = {
		border: 'none',
		bottom: '0',
		height: '100%',
		left: '0',
		outline: 'none',
		position: 'absolute',
		right: '0',
		top: '0',
		width: '100%',
	}
	return (
		<div class="index-wrapper">
			<div class="index-placeholder">
				<iframe
					allowFullScreen={true}
					src="https://player.vimeo.com/video/385352664"
					style={videoStyle}
					title="tangram video"
				/>
			</div>
		</div>
	)
}

function Features() {
	return (
		<div class="index-features-grid">
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img alt="training icon" class="index-step-icon" src={training} />
					<div class="index-step-title">{'Training'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'Train a model using the Tangram command line tool. Just pass a CSV file with your data and the name of the column you want to predict. Tangram automatically converts text, numeric and categorical data in into features machine learning models understand, trains linear and gradient boosted decision trees across a large hyperparameter grid and finally chooses the model with the best performance.'
						}
					</ui.P>
					<ui.P>
						<b>
							{
								'Your data stays secure because all training happens on your laptop or your own server.'
							}
						</b>
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Training} id="training" props={{}} />
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img alt="prediction icon" class="index-step-icon" src={prediction} />
					<div class="index-step-title">{'Prediction'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'Make predictions from your favorite programming language using the '
						}
						<ui.InlineCode>{'.tangram'}</ui.InlineCode>
						{' file generated by the command line tool.'}
					</ui.P>
					<ui.P>
						<b>
							{
								"Prediction happens locally, as opposed to calling an external API, so it's super fast and your data stays secure."
							}
						</b>
					</ui.P>
					<ui.P>
						{
							'Libraries are available for Go, JavaScript, Python, and Ruby. Support for C/C++, C#, Java, PHP, and Rust is coming soon.'
						}
					</ui.P>
				</ui.S2>
			</div>
			<div>
				<Prediction />
			</div>
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img
						alt="training stats icon"
						class="index-step-icon"
						src={trainingStats}
					/>
					<div class="index-step-title">{'Training Stats'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'Want to learn more about your model? Upload it to the Tangram reporting and monitoring app.'
						}
					</ui.P>
					<ui.P>
						{
							'View box plots showing the distribution of the numeric columns in your dataset, bar charts for categorical columns and word token histograms for text columns.'
						}
					</ui.P>
					<ui.P>
						{
							'Dirty data? Tangram calculates the number of missing values in each column so you can easily spot issues in your dataset.'
						}
					</ui.P>
					<ui.P>
						<b>
							{
								'The reporting and monitoring app can be run by us in the cloud or by you on your own servers.'
							}
						</b>
					</ui.P>
				</ui.S2>
			</div>
			<Client component={TrainingStats} id="training-stats" props={{}} />
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img
						alt="training metrics icon"
						class="index-step-icon"
						src={trainingMetrics}
					/>
					<div class="index-step-title">{'Training Metrics'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{'View metrics showing how your model performed on the test set.'}
					</ui.P>
					<ui.P>
						{
							'Tangram takes care of splitting your csv into a train set used to train your model and a test set used to evaluate your model.'
						}
					</ui.P>
					<ui.P>
						{
							'View common evaluation metrics like accuracy, precision, f1 scores and pr and roc curves.'
						}
					</ui.P>
					<ui.P>
						{
							'Find out where your model is making mistakes by viewing per-class confusion matrices.'
						}
					</ui.P>
				</ui.S2>
			</div>
			<Client component={TrainingMetrics} id="training-metrics" props={{}} />
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img
						alt="introspection icon"
						class="index-step-icon"
						src={introspection}
					/>
					<div class="index-step-title">{'Introspection'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'Learn more about how your model is making predictions by viewing the features that were most important to your model.'
						}
					</ui.P>
					<ui.P>
						{
							'View feature importance scores for gradient boosted decision tree models and raw model weights for linear models.'
						}
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Introspection} id="introspection" props={{}} />
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img
						alt="explainability icon"
						class="index-step-icon"
						src={explanations}
					/>
					<div class="index-step-title">{'Explanations'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'Figure out why your model made a particular prediction. Features with green/positive contributions push the probability of the class higher while features with red/negative contributions push the probability of the class lower.'
						}
					</ui.P>
					<ui.P>
						{
							'In this example, the baseline probability of being "positive" for heart disease is 51%. The patient\'s high cholesterol and low exercise max heart rate increased the probability that they have heart disease while their normal thallium stress test decreased the probability of heart disease lower.'
						}
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Explanations} id="explanations" props={{}} />
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img alt="tuning icon" class="index-step-icon" src={tuning} />
					<div class="index-step-title">{'Tuning'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'Tune your model to your preferred tradeoff between precision and recall.'
						}
					</ui.P>
					<ui.P>
						{
							'For example, if your model is a fraud detector, you probably want high recall to minimize false negatives. On the other hand, if your model is an email spam detector, you probably want high precision to minimize false positives.'
						}
					</ui.P>
					<ui.P>
						{
							'To use your selected threshold, just update the predict call in your code.'
						}
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Tuning} id="tuning" props={{}} />
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img
						alt="production stats icon"
						class="index-step-icon"
						src={productionStats}
					/>
					<div class="index-step-title">{'Production Stats'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'When your model is in production, you want to make sure that the data coming in is similar to what you trained on. You can opt in to logging by calling '
						}
						<ui.InlineCode>{'logPrediction'}</ui.InlineCode>
						{' in your code.'}
					</ui.P>
					<ui.P>
						{
							'Then you can compare production stats with training stats to make sure the distributions of each of the columns in production is similar to what you trained on.'
						}
					</ui.P>
					<ui.P>
						{
							'Tangram automatically alerts you when it detects issues in your production data.'
						}
					</ui.P>
				</ui.S2>
			</div>
			<Client component={ProductionStats} id="production-stats" props={{}} />
			<div class="index-step">
				<div class="index-step-title-wrapper">
					<img
						alt="production metrics icon"
						class="index-step-icon"
						src={productionMetrics}
					/>
					<div class="index-step-title">{'Production Metrics'}</div>
				</div>
				<ui.S2>
					<ui.P>
						{
							'Later on, you may find out the true value for what you were trying to predict. In your code, you can call '
						}
						<ui.InlineCode>{'logTrueValue'}</ui.InlineCode>
						{'.'}
					</ui.P>
					<ui.P>
						{
							'Then you can view production metrics to ensure your model is as accurate in production as it was in training.'
						}
					</ui.P>
					<ui.P>
						{
							'Sometimes datasets change over time. View metrics over time to ensure your model is performing as well as when you first trained it.'
						}
					</ui.P>
				</ui.S2>
			</div>
			<Client
				component={ProductionMetrics}
				id="production-metrics"
				props={{}}
			/>
		</div>
	)
}
