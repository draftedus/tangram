require 'json'
require 'net/http'
require 'tangram/libtangram'

module Tangram
	class Model

		def self.from_file(model_path, tangram_url: nil)
			model_data = IO.binread(model_path)
			self.from_data(model_data, tangram_url: tangram_url)
		end

		def self.from_data(model_data, tangram_url: nil)
			self.new(model_data, tangram_url: tangram_url)
		end

		def initialize(model_data, tangram_url: nil)
			@tangram_url = tangram_url.nil? ? 'https://app.tangramhq.com.com' : tangram_url
			@log_queue = []
			model_ptr = FFI::MemoryPointer.new(:pointer)
			result = LibTangram.tangram_model_load(model_data, model_data.size, model_ptr)
			if result != 0
				raise 'tangram error'
			end
			@model = FFI::AutoPointer.new(model_ptr.read_pointer, LibTangram.method(:tangram_model_free))
		end

		def id()
			id_ptr = FFI::MemoryPointer.new(:pointer)
			result = LibTangram.tangram_model_id(@model, id_ptr)
			if result != 0
				raise 'tangram error'
			end
			id = id_ptr.read_pointer.read_string.force_encoding('utf-8')
			LibTangram.tangram_string_free(id_ptr.read_pointer)
			id
		end

		def predict(input, options: nil)
			is_array = input.kind_of?(Array)
			input = is_array ? input : [input]
			input = JSON.unparse(input)
			unless options.nil?
				options = JSON.unparse(options)
			else
				options = nil
			end
			output_ptr = FFI::MemoryPointer.new(:pointer)
			result = LibTangram.tangram_model_predict(@model, input, options, output_ptr)
			if result != 0
				raise 'tangram error'
			end
			output = output_ptr.read_pointer.read_string.force_encoding('utf-8')
			LibTangram.tangram_string_free(output_ptr.read_pointer)
			output = JSON.parse(output)
			output = is_array ? output : output[0]
			output
		end

		def log_prediction(identifier:, options: nil, input:, output:)
			self.log_event(self.prediction_event(identifier, options, input, output))
		end

		def enqueue_log_prediction(identifier:, options: nil, input:, output:)
			self.log_queue.push(self.prediction_event(identifier, options, input, output))
		end

		def log_true_value(identifier:, true_value:)
			self.log_event(self.true_value_event(identifier, true_value))
		end

		def enqueue_log_true_value(identifier:, true_value:)
			self.log_queue.push(self.true_value_event(identifier, true_value))
		end

		def flush_log_queue()
			self.log_events(@log_queue)
			@log_queue = []
		end

		def log_event(event)
			self.log_events([event])
		end

		def log_events(events)
			headers = {
				'Content-Type': 'application/json',
			}
			uri = URI(@tangram_url + '/track')
			http = Net::HTTP.new(uri.host, uri.port)
			request = Net::HTTP::Post.new(uri.request_uri, headers)
			request.body = events.to_json
			response = http.request(request)
			unless response.kind_of? Net::HTTPSuccess
				raise response
			end
		end

		def prediction_event(identifier:, options: nil, input:, output:)
			return {
				type: 'prediction',
				modelId: self.id,
				identifier: identifier,
				options: options,
				input: input,
				output: output,
			}
		end

		def true_value_event(identifier:, true_value:)
			return {
				type: 'true_value',
				modelId: self.id,
				identifier: identifier,
				trueValue: true_value,
			}
		end

	end
end
