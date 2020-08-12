import { ui } from 'deps'

ui.hydrateBarChart('probabilities')
if (document.getElementById('regression_shap')) {
	ui.hydrateShapChart('regression_shap')
}
if (document.getElementById('classification_shap')) {
	ui.hydrateShapChart('classification_shap')
}
