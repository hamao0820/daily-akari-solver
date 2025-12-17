package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"

	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	"github.com/hamao0820/daily-akari-solver/detect"
)

type requestBody struct {
	ImageDataBase64 string `json:"image_data_base64"`
	ProblemNo       int    `json:"problem_no"`
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

		grid, err := getProblemData(data.ProblemNo)
		if err != nil {
			log.Println("Error fetching problem data:", err)
			http.Error(w, "Failed to fetch problem data", http.StatusInternalServerError)
			return
		}

		if len(grid) == 0 || len(grid[0]) == 0 {
			log.Println("Error: problem data is empty")
			http.Error(w, "Problem data is empty", http.StatusInternalServerError)
			return
		}

		cells, err := detect.DetectCells(imageData, len(grid), len(grid[0]))
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

func getProblemData(problemNo int) ([][]string, error) {
	var url string
	if problemNo == -1 {
		url = "https://dailyakari.com/dailypuzzle"
	} else {
		url = fmt.Sprintf("https://dailyakari.com/archivepuzzle?number=%d", problemNo)
	}
	resp, err := http.Get(url)
	if err != nil {
		return [][]string{}, fmt.Errorf("failed to fetch problem data: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return [][]string{}, fmt.Errorf("failed to read problem data: %w", err)
	}

	type apiResponse struct {
		LevelData string `json:"levelData"`
	}

	var responseData apiResponse
	if err := json.Unmarshal(body, &responseData); err != nil {
		return [][]string{}, fmt.Errorf("failed to unmarshal problem data: %w", err)
	}

	grid := parseLevelData(responseData.LevelData)
	return grid, nil
}

func parseLevelData(str string) [][]string {
	var grid [][]string
	// \n\nより手前がグリッドデータ
	dataParts := strings.SplitN(str, "\n\n", 2)[0]

	rows := strings.Split(dataParts, "\n")
	for _, row := range rows {
		grid = append(grid, strings.Split(row, " "))
	}
	return grid
}
