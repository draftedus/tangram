package main

import (
	"fmt"
	"github.com/tangram-hq/tangram/languages/go"
	"log"
)

func main() {
	// Load the model from the file.
	model, err := tangram.LoadModelFromFile("./heart-disease.tangram", nil)
	if err != nil {
		log.Fatal(err)
	}
	// destroy the model when it is no longer needed to free up memory.
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

	// Make the prediction!
	output := model.PredictOne(input, nil)

	// Print out the input and output.
	fmt.Println("Input:", input)
	fmt.Println("Output:", output.ClassName)
}
