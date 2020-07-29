import json
import requests
from tangram.libtangram import libtangram, ffi

class Model:

	@classmethod
	def from_file(cls, model_path, tangram_url=None):
		model_data = open(model_path, mode='rb').read()
		return cls.from_data(model_data, tangram_url=tangram_url)

	@classmethod
	def from_data(cls, model_data, tangram_url=None):
		return cls(model_data, tangram_url=tangram_url)

	def __init__(self, model_data, tangram_url=None):
		self.tangram_url = tangram_url if tangram_url is not None else 'https://app.tangramhq.com.com'
		self.log_queue = []
		model_ptr = ffi.new('tangram_model**')
		result = libtangram.tangram_model_load(model_data, len(model_data), model_ptr)
		if result != 0:
			raise 'tangram error'
		self.model = model_ptr[0]

	def __del__(self):
		result = libtangram.tangram_model_free(self.model)
		if result != 0:
			raise 'tangram error'

	def id(self):
		id_ptr = ffi.new('char**')
		result = libtangram.tangram_model_id(self.model, id_ptr)
		if result != 0:
			raise Exception('tangram error')
		id_ptr = id_ptr[0]
		id = ffi.string(id_ptr).decode('utf-8')
		libtangram.tangram_string_free(id_ptr)
		return id

	def predict(self, input, options=None):
		is_array = isinstance(input, list)
		input = input if is_array else [input]
		input = json.dumps(input).encode('utf-8')
		options = json.dumps(options).encode('utf-8') if options is not None else ffi.NULL
		output_ptr = ffi.new('char**')
		result = libtangram.tangram_model_predict(self.model, input, options, output_ptr)
		if result != 0:
			raise Exception('tangram error')
		output_ptr = output_ptr[0]
		output = ffi.string(output_ptr).decode('utf-8')
		result = libtangram.tangram_string_free(output_ptr)
		if result != 0:
			raise Exception('tangram error')
		output = json.loads(output)
		output = output if is_array else output[0]
		return output

	def log_prediction(self, identifier, input, output, options=None):
		self.log_event(self.prediction_event(identifier, input, output, options))

	def enqueue_log_prediction(self, identifier, input, output, options=None):
		self.log_queue.push(self.prediction_event(identifier, input, output, options))

	def log_true_value(self, identifier, true_value):
		self.log_event(self.true_value_event(identifier, true_value))

	def enqueue_log_true_value(self, identifier, true_value):
		self.log_queue.push(self.true_value_event(identifier, true_value))

	def flush_log_queue(self):
		self.log_events(self.log_queue)
		self.log_queue = []

	def log_event(self, event):
		self.log_events([event])

	def log_events(self, events):
		url = self.tangram_url + '/track'
		body = json.dumps(events)
		response = requests.post(
			url,
			data=body,
			headers={
				'Content-Type': 'application/json',
			}
		)
		response.raise_for_status()

	def prediction_event(self, identifier, input, output, options=None):
		return {
			'type': 'prediction',
			'modelId': self.id(),
			'identifier': identifier,
			'options': options,
			'input': input,
			'output': output,
		}

	def true_value_event(self, identifier, true_value):
		self.log_event({
			'type': 'true_value',
			'modelId': self.id(),
			'identifier': identifier,
			'trueValue': true_value,
		})
