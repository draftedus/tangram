import { Install } from '../../shared/install'
import { DatasetPreview } from './dataset_preview'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { DocsLayout, DocsPage } from 'layouts/docs_layout'
import { h } from 'preact'

export default (pageInfo: PageInfo) => {
	return renderPage(
		<DocsLayout
			clientJsSrc={pageInfo.clientJsSrc}
			cssSrcs={pageInfo.cssSrcs}
			preloadJsSrcs={pageInfo.preloadJsSrcs}
			selectedPage={DocsPage.Train}
		>
			<ui.S1>
				<ui.H1>{'Getting Started'}</ui.H1>
				<ui.P>
					{
						'Thanks for trying Tangram! In this tutorial, we will train a model to predict whether cardiac patients have heart disease and make predictions in our code. Then we will tune our model to get the best performance and debug a number of issues in production.'
					}
				</ui.P>
				<ui.S2>
					<ui.H2>{'Install the Tangram CLI'}</ui.H2>
					<Install />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Get the data'}</ui.H2>
					<ui.P>
						<ui.Link href="https://tangram-datasets.nyc3.digitaloceanspaces.com/heart-disease.csv">
							{'Download heart-disease.csv'}
						</ui.Link>
					</ui.P>
					<ui.P>
						{
							'The heart disease sample dataset contains information from cardiac patients such as their age, cholesterol, and stress test results. Below are some example rows.'
						}
					</ui.P>
					<DatasetPreview />
					<ui.P>
						{'The last column, called '}
						<ui.InlineCode>{'diagnosis'}</ui.InlineCode>
						{', is either '}
						<ui.InlineCode>{'Positive'}</ui.InlineCode>
						{' if the patient has heart disease or '}
						<ui.InlineCode>{'Negative'}</ui.InlineCode>
						{" if they don't."}
					</ui.P>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Train'}</ui.H2>
					<ui.P>
						{'We can train a model to predict the '}
						<ui.InlineCode>{'diagnosis'}</ui.InlineCode>
						{' column using the '}
						<ui.InlineCode>{'tangram train'}</ui.InlineCode>
						{
							' command, passing in the path to the CSV file and the name of the column we want to predict, called the '
						}
						<ui.InlineCode>{'target'}</ui.InlineCode>
						{' column.'}
					</ui.P>
					<ui.Code
						code="$ tangram train --file heart-disease.csv --target diagnosis"
						hideLineNumbers={true}
					/>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Column Names'}</ui.H2>
					<ui.P>
						{
							'The first thing the CLI does is infer the names and types of the columns in your dataset. The column names are taken from the header row in your CSV file. A header is required.'
						}
					</ui.P>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Null Values'}</ui.H2>
					<ui.P>
						{'Values are considered to be '}
						<ui.InlineCode>{'null'}</ui.InlineCode>
						{
							' if they are equal to the empty string, whether or not they are wrapped in quotes. For example, in the following CSV, each value in '
						}
						<ui.InlineCode>{'column_b'}</ui.InlineCode>
						{' is considered to be '}
						<ui.InlineCode>{'null'}</ui.InlineCode>
						{'.'}
					</ui.P>
					<ui.Code
						code={`column_a,column_b,column_c\n1,"",3\n2,,4`}
						hideLineNumbers={true}
					/>
					<ui.P>
						{
							'It is possible your CSV data will have other ways of indicating a null value, such as '
						}
						<ui.InlineCode>{'NULL'}</ui.InlineCode>
						{' or '}
						<ui.InlineCode>{'?'}</ui.InlineCode>
						{
							'. If that is the case, you need to preprocess your CSV file before using the Tangram CLI.'
						}
					</ui.P>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Column Types'}</ui.H2>
					<ui.P>
						{
							'The CLI will assign each column one of three possible column types: '
						}
						<ui.InlineCode>{'Number'}</ui.InlineCode>
						{', '}
						<ui.InlineCode>{'Bool'}</ui.InlineCode>
						{', '}
						<ui.InlineCode>{'Enum'}</ui.InlineCode>
						{', or '}
						<ui.InlineCode>{'Text'}</ui.InlineCode>
						{'.'}
					</ui.P>
					<ui.List>
						<ui.ListItem>
							<ui.InlineCode>{'Number'}</ui.InlineCode>
							{': A column is a '}
							<ui.InlineCode>{'Number'}</ui.InlineCode>
							{
								' column if every non-null value in the column parses as a finite floating point number.'
							}
						</ui.ListItem>
						<ui.ListItem>
							<ui.InlineCode>{'Bool'}</ui.InlineCode>
							{': A column is a '}
							<ui.InlineCode>{'Bool'}</ui.InlineCode>
							{
								' column if every non-null value in the column is either true/false, yes/no, or 0/1.'
							}
						</ui.ListItem>
						<ui.ListItem>
							<ui.InlineCode>{'Enum'}</ui.InlineCode>
							{': A column is an '}
							<ui.InlineCode>{'Enum'}</ui.InlineCode>
							{' column if it is not a '}
							<ui.InlineCode>{'Number'}</ui.InlineCode>
							{
								' column and the count of unique non-null values is less than 100.'
							}
						</ui.ListItem>
						<ui.ListItem>
							<ui.InlineCode>{'Text'}</ui.InlineCode>
							{': A column is a '}
							<ui.InlineCode>{'Text'}</ui.InlineCode>
							{' column if it is neither a '}
							<ui.InlineCode>{'Number'}</ui.InlineCode>
							{' column nor an '}
							<ui.InlineCode>{'Enum'}</ui.InlineCode>
							{' column.'}
						</ui.ListItem>
					</ui.List>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Task Types'}</ui.H2>
					<ui.P>
						{'Tangram supports two distinct task types: '}
						<b>{'regression'}</b>
						{' and '}
						<b>{'classification'}</b>
						{
							'. A regression task predicts a continuous numeric value, for example the price of a house. A classification task predicts one of a discrete set of values, for example the species of an animal. Tangram trains a regression model when the inferred type of the target column is '
						}
						<ui.InlineCode>{'Number'}</ui.InlineCode>
						{
							' and a classification model when the inferred type of the target column is '
						}
						<ui.InlineCode>{'Enum'}</ui.InlineCode>
						{' or '}
						<ui.InlineCode>{'Bool'}</ui.InlineCode>
						{'.'}
					</ui.P>
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Model Types'}</ui.H2>
					<ui.P>
						{
							'Tangram trains linear models and gradient boosted decision tree models and optimizes their hyperparameter settings. In the case of regression, tangram will choose the model with the lowest root mean squared error. In the case of classification, tangram will choose the model with the highest accuracy.'
						}
					</ui.P>
				</ui.S2>
			</ui.S1>
		</DocsLayout>,
	)
}
