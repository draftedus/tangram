import { ui } from 'deps'

if (document.getElementById('probabilities')) {
	ui.hydrateBarChart('probabilities')
}
if (document.getElementById('regression_shap')) {
	ui.hydrateShapChart('regression_shap')
}
if (document.getElementById('classification_shap')) {
	ui.hydrateShapChart('classification_shap')
}
