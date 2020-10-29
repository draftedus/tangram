import tangram
import os

# Get the path to the .tangram file.
model_path = os.path.join('heart_disease.tangram')
# Load the model from the file.
model = tangram.Model.from_file(model_path)