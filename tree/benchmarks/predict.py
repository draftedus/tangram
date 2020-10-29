import tangram
import os
from time import time

start = time()
# Get the path to the .tangram file.
model_path = os.path.join('data/heart-disease.tangram')
# Load the model from the file.
model = tangram.Model.from_file(model_path)
print(time() - start)