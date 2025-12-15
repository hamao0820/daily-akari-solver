package detect

import (
	"embed"
	"image"
	"image/color"
	"math"
	"slices"

	"gocv.io/x/gocv"
)

type Symbol int

const (
	SymbolEmpty Symbol = iota
	SymbolOne
	SymbolTwo
	SymbolThree
	SymbolFour
	SymbolLOL
)

//go:embed template
var templateFS embed.FS

var oneEdge gocv.Mat
var twoEdge gocv.Mat
var threeEdge gocv.Mat
var fourEdge gocv.Mat
var lolEdge gocv.Mat

func init() {
	oneData, _ := templateFS.ReadFile("template/one.png")
	one, _ := gocv.IMDecode(oneData, gocv.IMReadGrayScale)
	oneEdge = gocv.NewMat()
	gocv.Canny(one, &oneEdge, 50, 150)
	one.Close()

	twoData, _ := templateFS.ReadFile("template/two.png")
	two, _ := gocv.IMDecode(twoData, gocv.IMReadGrayScale)
	twoEdge = gocv.NewMat()
	gocv.Canny(two, &twoEdge, 50, 150)
	two.Close()

	threeData, _ := templateFS.ReadFile("template/three.png")
	three, _ := gocv.IMDecode(threeData, gocv.IMReadGrayScale)
	threeEdge = gocv.NewMat()
	gocv.Canny(three, &threeEdge, 50, 150)
	three.Close()

	fourData, _ := templateFS.ReadFile("template/four.png")
	four, _ := gocv.IMDecode(fourData, gocv.IMReadGrayScale)
	fourEdge = gocv.NewMat()
	gocv.Canny(four, &fourEdge, 50, 150)
	four.Close()

	lolData, _ := templateFS.ReadFile("template/lol.png")
	lol, _ := gocv.IMDecode(lolData, gocv.IMReadGrayScale)
	lolEdge = gocv.NewMat()
	gocv.Canny(lol, &lolEdge, 50, 150)
	lol.Close()
}

func Detect(buf []byte) string {
	img, err := gocv.IMDecode(buf, gocv.IMReadColor)
	if err != nil {
		panic(err)
	}
	if img.Empty() {
		panic("画像の読み込みに失敗しました")
	}
	defer img.Close()

	gray := gocv.NewMat()
	defer gray.Close()
	gocv.CvtColor(img, &gray, gocv.ColorBGRToGray)

	blockEdges := gocv.NewMat()
	defer blockEdges.Close()
	gocv.Canny(img, &blockEdges, 200, 300)

	blockDilated := gocv.NewMat()
	defer blockDilated.Close()
	gocv.DilateWithParams(blockEdges, &blockDilated, gocv.GetStructuringElement(gocv.MorphRect, image.Pt(5, 5)), image.Pt(-1, -1), 2, gocv.BorderConstant, color.RGBA{0, 0, 0, 0})

	blockHierarchy := gocv.NewMat()
	defer blockHierarchy.Close()
	blockContours := gocv.FindContoursWithParams(blockDilated, &blockHierarchy, gocv.RetrievalList, gocv.ChainApproxSimple)
	defer blockContours.Close()

	blockMask := gocv.NewMatWithSize(img.Rows(), img.Cols(), gocv.MatTypeCV8U)
	gocv.DrawContours(&blockMask, blockContours, -1, color.RGBA{255, 255, 255, 0}, -1)

	gridEdges := gocv.NewMat()
	defer gridEdges.Close()
	gocv.Canny(gray, &gridEdges, 50, 50)

	gridDilated := gocv.NewMat()
	defer gridDilated.Close()
	gocv.DilateWithParams(gridEdges, &gridDilated, gocv.GetStructuringElement(gocv.MorphRect, image.Pt(5, 5)), image.Pt(-1, -1), 2, gocv.BorderConstant, color.RGBA{0, 0, 0, 0})

	gridHierarchy := gocv.NewMat()
	defer gridHierarchy.Close()
	gridContours := gocv.FindContoursWithParams(gridDilated, &gridHierarchy, gocv.RetrievalList, gocv.ChainApproxSimple)
	defer gridContours.Close()

	validWidths := []int{}
	validHeights := []int{}

	for i := 0; i < gridContours.Size(); i++ {
		countour := gridContours.At(i)
		rect := gocv.BoundingRect(countour)
		area := rect.Dx() * rect.Dy()

		if area < 500 || area > 10000 {
			continue
		}

		ratio := float64(rect.Dx()) / float64(rect.Dy())
		if ratio < 0.5 || ratio > 2.0 {
			continue
		}

		roi := blockMask.Region(rect)
		nonZero := gocv.CountNonZero(roi)
		roi.Close()

		blockRatio := float64(nonZero) / float64(rect.Dx()*rect.Dy())
		if blockRatio < 0.075 {
			validWidths = append(validWidths, rect.Dx())
			validHeights = append(validHeights, rect.Dy())
		} else {
		}
	}

	widthMedian := median(validWidths)
	heightMedian := median(validHeights)

	if math.Abs(float64(widthMedian-heightMedian)) > 1 {
		panic("セルの幅と高さが大きく異なります。グリッド検出に失敗している可能性があります。")
	}

	correctSize := (widthMedian + heightMedian) / 2
	correctArea := correctSize * correctSize
	correctRects := []image.Rectangle{}
	for i := 0; i < gridContours.Size(); i++ {
		countour := gridContours.At(i)
		rect := gocv.BoundingRect(countour)
		area := rect.Dx() * rect.Dy()

		if area < 500 || area > 10000 {
			continue
		}

		ratio := float64(rect.Dx()) / float64(rect.Dy())
		if ratio < 0.5 || ratio > 2.0 {
			continue
		}

		if math.Abs(float64(area-correctArea))/float64(correctArea) > 0.3 {
			continue
		}

		if rect.Dx() < correctSize*3/4 || rect.Dx() > correctSize*5/4 {
			continue
		}

		correctRects = append(correctRects, rect)
	}

	centers := []image.Point{}
	for _, rect := range correctRects {
		center := image.Pt(rect.Min.X+rect.Dx()/2, rect.Min.Y+rect.Dy()/2)
		centers = append(centers, center)
	}

	slices.SortFunc(centers, func(a, b image.Point) int {
		if math.Abs(float64(a.Y-b.Y)) < float64(correctSize)/2 {
			return a.X - b.X
		}
		return a.Y - b.Y
	})

	// 完全なグリッドを生成
	completeGrid, rows, cols := generateCompleteGrid(centers, correctSize, correctSize, correctSize)

	// 各中心の周囲のセルを描画
	cells := []gocv.Mat{}
	for _, center := range completeGrid {
		rect := image.Rect(center.X-correctSize/2+correctSize/10, center.Y-correctSize/2+correctSize/10, center.X+correctSize/2-correctSize/10, center.Y+correctSize/2-correctSize/10)
		cell := img.Region(rect)
		cells = append(cells, cell)
	}

	result := ""
	for row := range rows {
		for col := range cols {
			index := row*cols + col
			if index < len(cells) {
				cell := cells[index]
				if !isBlock(cell) {
					result += "."
					continue
				}
				switch detectSymbol(cell) {
				case SymbolOne:
					result += "1"
				case SymbolTwo:
					result += "2"
				case SymbolThree:
					result += "3"
				case SymbolFour:
					result += "4"
				case SymbolLOL:
					result += "0"
				case SymbolEmpty:
					result += "#"
				}
				cell.Close()
			}
		}
		result += "\n"
	}

	return result
}

func isBlock(cell gocv.Mat) bool {
	gray := gocv.NewMat()
	defer gray.Close()
	gocv.CvtColor(cell, &gray, gocv.ColorBGRToGray)

	thresh := gocv.NewMat()
	defer thresh.Close()
	gocv.Threshold(gray, &thresh, 0, 30, gocv.ThresholdBinary)

	nonZero := gocv.CountNonZero(thresh)
	area := cell.Rows() * cell.Cols()
	blockRatio := float64(nonZero) / float64(area)

	return blockRatio < 0.7
}

func detectSymbol(cell gocv.Mat) Symbol {
	const threshold = 0.4

	gray := gocv.NewMat()
	defer gray.Close()
	gocv.CvtColor(cell, &gray, gocv.ColorBGRToGray)

	edge := gocv.NewMat()
	defer edge.Close()
	gocv.Canny(gray, &edge, 50, 150)

	// テンプレートマッチング
	// LOL → 四 → 三 → 二 → 一 の順で検出し、高スコアなら即座に返す

	lolResult := gocv.NewMat()
	defer lolResult.Close()
	gocv.MatchTemplate(edge, lolEdge, &lolResult, gocv.TmCcoeffNormed, gocv.NewMat())
	_, maxValLOL, _, _ := gocv.MinMaxLoc(lolResult)
	if maxValLOL > threshold {
		return SymbolLOL
	}

	fourResult := gocv.NewMat()
	defer fourResult.Close()
	gocv.MatchTemplate(edge, fourEdge, &fourResult, gocv.TmCcoeffNormed, gocv.NewMat())
	_, maxValFour, _, _ := gocv.MinMaxLoc(fourResult)
	if maxValFour > threshold {
		return SymbolFour
	}

	threeResult := gocv.NewMat()
	defer threeResult.Close()
	gocv.MatchTemplate(edge, threeEdge, &threeResult, gocv.TmCcoeffNormed, gocv.NewMat())
	_, maxValThree, _, _ := gocv.MinMaxLoc(threeResult)
	if maxValThree > threshold {
		return SymbolThree
	}

	twoResult := gocv.NewMat()
	defer twoResult.Close()
	gocv.MatchTemplate(edge, twoEdge, &twoResult, gocv.TmCcoeffNormed, gocv.NewMat())
	_, maxValTwo, _, _ := gocv.MinMaxLoc(twoResult)
	if maxValTwo > threshold {
		return SymbolTwo
	}

	oneResult := gocv.NewMat()
	defer oneResult.Close()
	gocv.MatchTemplate(edge, oneEdge, &oneResult, gocv.TmCcoeffNormed, gocv.NewMat())
	_, maxValOne, _, _ := gocv.MinMaxLoc(oneResult)
	if maxValOne > threshold {
		return SymbolOne
	}

	return SymbolEmpty
}

// 平均座標を計算
func averageCoord(points []image.Point, getCoord func(image.Point) int) int {
	if len(points) == 0 {
		return 0
	}
	sum := 0
	for _, p := range points {
		sum += getCoord(p)
	}
	return sum / len(points)
}

// 完全なグリッドを生成
func generateCompleteGrid(centers []image.Point, rowSpacing, colSpacing, cellSize int) ([]image.Point, int, int) {
	if len(centers) == 0 {
		return nil, 0, 0
	}

	rows := extractRows(centers, cellSize)
	cols := extractCols(centers, cellSize)

	// 各行・列の代表座標を計算
	rowCoords := []int{}
	for _, row := range rows {
		rowCoords = append(rowCoords, averageCoord(row, func(p image.Point) int { return p.Y }))
	}

	colCoords := []int{}
	for _, col := range cols {
		colCoords = append(colCoords, averageCoord(col, func(p image.Point) int { return p.X }))
	}

	// 欠損部分を補完
	completeRows := fillGaps(rowCoords, rowSpacing)
	completeCols := fillGaps(colCoords, colSpacing)

	// 完全なグリッドを生成
	grid := []image.Point{}
	for _, y := range completeRows {
		for _, x := range completeCols {
			grid = append(grid, image.Pt(x, y))
		}
	}

	return grid, len(completeRows), len(completeCols)
}

// ギャップを埋める
func fillGaps(coords []int, spacing int) []int {
	if len(coords) == 0 || spacing == 0 {
		return coords
	}

	result := []int{coords[0]}

	for i := 1; i < len(coords); i++ {
		gap := coords[i] - coords[i-1]
		expectedGaps := int(math.Round(float64(gap) / float64(spacing)))

		if expectedGaps > 1 {
			// ギャップを補完
			for j := 1; j < expectedGaps; j++ {
				interpolated := coords[i-1] + j*spacing
				result = append(result, interpolated)
			}
		}
		result = append(result, coords[i])
	}

	return result
}

func median(data []int) int {
	n := len(data)
	if n == 0 {
		return 0
	}

	sorted := make([]int, n)
	copy(sorted, data)
	slices.Sort(sorted)
	if n%2 == 1 {
		return sorted[n/2]
	}
	return (sorted[n/2-1] + sorted[n/2]) / 2
}

// 同じ行のポイントをグループ化
func extractRows(centers []image.Point, cellSize int) [][]image.Point {
	if len(centers) == 0 {
		return nil
	}

	rows := [][]image.Point{}
	currentRow := []image.Point{centers[0]}

	for i := 1; i < len(centers); i++ {
		if math.Abs(float64(centers[i].Y-currentRow[0].Y)) < float64(cellSize)/2 {
			currentRow = append(currentRow, centers[i])
		} else {
			rows = append(rows, currentRow)
			currentRow = []image.Point{centers[i]}
		}
	}
	rows = append(rows, currentRow)
	return rows
}

// 列ごとにポイントを抽出
func extractCols(centers []image.Point, cellSize int) [][]image.Point {
	if len(centers) == 0 {
		return nil
	}

	// X座標でソート
	sorted := make([]image.Point, len(centers))
	copy(sorted, centers)
	slices.SortFunc(sorted, func(a, b image.Point) int {
		return a.X - b.X
	})

	cols := [][]image.Point{}
	currentCol := []image.Point{sorted[0]}

	for i := 1; i < len(sorted); i++ {
		if math.Abs(float64(sorted[i].X-currentCol[0].X)) < float64(cellSize)/2 {
			currentCol = append(currentCol, sorted[i])
		} else {
			cols = append(cols, currentCol)
			currentCol = []image.Point{sorted[i]}
		}
	}
	cols = append(cols, currentCol)
	return cols
}
