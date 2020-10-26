package main

import (
	"fmt"
	"log"
	"os"
)

func main() {
	// If you are running the Tangram reporting and monitoring web app on your own server you can pass the URL to it with the TANGRAM_URL environment variable.
	tangramURL, present := os.LookupEnv("TANGRAM_URL")
	if !present {
		tangramURL = "https://app.tangramhq.com"
	}

	// Load the model from the file.
	options := tangram.ModelOptions{
		BaseURL: tangramURL,
	}
	model, err := tangram.LoadModelFromFile("./heart_disease.tangram", &options)
	if err != nil {
		log.Fatal(err)
	}
	// Destroy the model when it is no longer needed to free up memory.
	defer model.Destroy()

	// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
	input := tangram.Input{
		"age":                                  63,
		"gender":                               "male",
		"chest_pain":                           "typical angina",
		"resting_blood_pressure":               145,
		"cholesterol":                          233,
		"fasting_blood_sugar_greater_than_120": "true",
		"resting_ecg_result":                   "probable or definite left ventricular hypertrophy",
		"exercise_max_heart_rate":              150,
		"exercise_induced_angina":              "no",
		"exercise_st_depression":               2.3,
		"exercise_st_slope":                    "downsloping",
		"fluoroscopy_vessels_colored":          0,
		"thallium_stress_test":                 "fixed defect",
	}

	// Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram reporting and monitoring web app.
	predictOptions := tangram.PredictOptions{
		Threshold: 0.25,
	}
	output := model.PredictOne(input, &predictOptions)

	// Print out the input and output.
	fmt.Println("Input:", input)
	fmt.Println("Output:", output.ClassName)

	// Log the prediction. This will allow us to view production stats in the Tangram reporting and monitoring web app.
	predictionEvent := tangram.LogPredictionOptions{
		Identifier: "John Doe",
		Options:    predictOptions,
		Input:      input,
		Output:     output,
	}
	err = model.LogPrediction(predictionEvent)
	if err != nil {
		log.Fatal(err)
	}

	// Later on, if we get an official diagnosis for the patient, we can log the true value for the prior prediction. Make sure to match the `identifier` from the former prediction.
	trueValueEvent := tangram.LogTrueValueOptions{
		Identifier: "John Doe",
		TrueValue:  "Positive",
	}
	err = model.LogTrueValue(trueValueEvent)
	if err != nil {
		log.Fatal(err)
	}
}
