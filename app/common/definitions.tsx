export let indicatorFeature =
	"An indicator feature is one where the value is 1 if the feature is present in the example and 0 otherwise. For instance, an indicator feature group representing a customer's payment plan with two distinct values: 'paid' and 'free' has two indicator features: 'paid' and 'free'. These features can be thought of as 'is_paid' and 'is_free' where the value of 'is_paid' is 1 for a training example if and only if the example is for a customer on a 'paid' plan."
export let normalization =
	'Normalization scales numeric features by the standard deviation and mean. This allows the algorithm to learn better in cases where numeric features are very different scales. For instance, if one feature is number of bed rooms and another is square footage, square footage is on the order of 1000s whereas bedrooms is on the order of single digits. normalized value = (value - mean) / std'
export let numberFeature =
	'A numeric feature is one where the value has been normalized to have mean 0 and a standard deviation of 1'
export let regularization =
	'Regularization helps prevent overfitting in training by adding a term to the loss function that penalizes high weights.'
export let batchGradientDescent =
	'In batch gradient descent, the gradient of the loss function is evaluated for the whole dataset before making an update to the weights in the model.'
export let epoch =
	'An epoch is a full pass over the training dataset. If we have 100 rows, with mini-batch gradient descent of size 20 examples, we would have 5 batches for a single epoch. In stochastic gradient descent with 1 example per batch, we would have 100 batches for 1 epoch and for full batch gradient descent we would have 1 batch per epoch.'
export let logLoss =
	'Logarithmic loss is the loss function used in logistic regression.'
export let crossEntropyLoss =
	'Cross Entropy Loss is the loss function used in multiclass classification.'
export let logisticRegression =
	'A logistic regression classifier is a linear binary classifier. The model takes the form: `y = Sigmoid(mx + b)` where y ranges from 0 to 1 due to the sigmoid function.'
export let miniBatchGradientDescent =
	'In mini-batch gradient descent, the gradient of the loss function is evaluated by averaging the gradient of a batch of training examples.'
export let stochasticGradientDescent =
	'In stochastic gradient descent, the gradient of the loss function and update of the model weights is performed for each training example.'
export let accuracy =
	'Accuracy is the percentage of instances in the training dataset that were correctly classified.'
export let aucRoc =
	"The area under the receiver operating characteric curve is the probability that a randomly chosen positive example's predicted score is higher than a randomly selected negative example's score"
export let confusionMatrix =
	'The confusion matrix shows the distribution of how your model predicted classes versus what the actual class labels are.'
export let precisionRecall =
	'Precision is the percentage of examples that were labeled as positive that are actually positive. Recall is the percentage of positive examples that were labeled as positive.'
export let precision =
	'Precision is the percentage of examples that were labeled as positive that are actually positive.'
export let recall =
	'Recall is the percentage of positive examples that were labeled as positive.'
export let receiverOperatingCharacteristic =
	'The Receiver Operating Characteristic Curve shows the True Positive Rate v. False Positive Rate at various thresholds in binary classification.'
