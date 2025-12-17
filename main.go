package main

import (
	"encoding/base64"
	"encoding/json"
	"io"
	"log"
	"net/http"

	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	"github.com/hamao0820/daily-akari-solver/detect"
)

type requestBody struct {
	ImageDataBase64 string     `json:"image_data_base64"`
	ProblemData     [][]string `json:"problem_data"`
}

func main() {
	router := chi.NewRouter()

	router.Use(middleware.Logger)

	router.Get("/", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"message": "Hello, World!"}`))
	})

	router.Post("/positions", func(w http.ResponseWriter, r *http.Request) {
		b, err := io.ReadAll(r.Body)
		if err != nil {
			log.Println("Error reading request body:", err)
			http.Error(w, "Failed to read request body", http.StatusBadRequest)
			return
		}

		var data requestBody
		err = json.Unmarshal(b, &data)
		if err != nil {
			log.Println("Error unmarshaling JSON:", err)
			http.Error(w, "Invalid JSON", http.StatusBadRequest)
			return
		}

		// base64デコード
		imageData, err := base64.StdEncoding.DecodeString(data.ImageDataBase64[len("data:image/png;base64,"):])
		if err != nil {
			log.Println("Error decoding base64 image data:", err)
			http.Error(w, "Invalid base64 image data", http.StatusBadRequest)
			return
		}

		cells, err := detect.DetectCells(imageData, len(data.ProblemData), len(data.ProblemData[0]))
		if err != nil {
			log.Println("Error detecting cell positions:", err)
			http.Error(w, "Failed to detect cell positions", http.StatusInternalServerError)
			return
		}

		// 結果をJSONで返す
		responseData, err := json.Marshal(struct {
			CellPositions []detect.Cell `json:"cells"`
		}{
			CellPositions: cells,
		})
		if err != nil {
			http.Error(w, "Failed to marshal response JSON", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		w.Write(responseData)
	})

	log.Println("Starting server on :8080")
	if err := http.ListenAndServe(":8080", router); err != nil {
		panic(err)
	}
}
