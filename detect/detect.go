package detect

import (
	"errors"
	"fmt"
	"image"
	"image/color"

	"gocv.io/x/gocv"
)

func DetectCells(buf []byte, rows, cols int) ([]Cell, error) {
	img, err := gocv.IMDecode(buf, gocv.IMReadColor)
	if err != nil {
		return []Cell{}, err
	}
	if img.Empty() {
		return []Cell{}, errors.New("画像のデコードに失敗しました")
	}
	defer img.Close()

	contours := findContours(img)
	defer contours.Close()

	boardRectangle := calcBoardRectangle(contours, rows, cols)

	boardImg := img.Region(boardRectangle)
	defer boardImg.Close()

	contoursOnBoard := findContours(boardImg)
	lightPleacableCells := etractLightPleacableCells(contoursOnBoard, boardRectangle.Dx()*boardRectangle.Dy(), rows, cols)

	rects := []image.Rectangle{}
	for _, contour := range lightPleacableCells {
		rect := gocv.BoundingRect(contour)
		rects = append(rects, rect)
	}

	// boardRect := image.Rectangle{Min: image.Pt(0, 0), Max: image.Pt(boardImg.Size()[0], boardImg.Size()[1])}
	cells, err := IdentifyGridCells(image.Pt(boardImg.Size()[0], boardImg.Size()[1]), rows, cols, rects)
	if err != nil {
		return []Cell{}, fmt.Errorf("failed to identify grid cells: %w", err)
	}

	// 矩形の位置を元画像基準に補正
	for i := range cells {
		cells[i].Rect = cells[i].Rect.Add(boardRectangle.Min)
	}

	return cells, nil
}

// countours からアカリを置くことができるセルを抽出する
func etractLightPleacableCells(countours gocv.PointsVector, boardArea int, row, col int) []gocv.PointVector {
	// 標準的なセルの面積を計算しておく
	standardArea := float64(boardArea) / float64(row*col)

	normalCells := []gocv.PointVector{}
	for i := 0; i < countours.Size(); i++ {
		contour := countours.At(i)
		area := gocv.ContourArea(contour)

		// 面積が標準的なセルから2倍以上離れているものは除外
		if area < standardArea*0.5 || area > standardArea*2.0 {
			continue
		}

		// アスペクト比が極端に偏っているものは除外
		rect := gocv.BoundingRect(contour)
		aspectRatio := min(float64(rect.Dx())/float64(rect.Dy()), float64(rect.Dy())/float64(rect.Dx()))
		if aspectRatio < 0.75 {
			continue
		}

		normalCells = append(normalCells, contour)
	}

	return normalCells
}

// img から輪郭を検出
// Canny エッジ検出と膨張処理を組み合わせてノイズを減らす
func findContours(img gocv.Mat) gocv.PointsVector {
	gray := gocv.NewMat()
	defer gray.Close()
	gocv.CvtColor(img, &gray, gocv.ColorBGRToGray)

	edges := gocv.NewMat()
	defer edges.Close()
	gocv.Canny(gray, &edges, 50, 50)

	dilated := gocv.NewMat()
	defer dilated.Close()
	gocv.DilateWithParams(edges, &dilated, gocv.GetStructuringElement(gocv.MorphRect, image.Pt(5, 5)), image.Pt(-1, -1), 2, gocv.BorderConstant, color.RGBA{0, 0, 0, 0})

	hierarchy := gocv.NewMat()
	defer hierarchy.Close()
	contours := gocv.FindContoursWithParams(dilated, &hierarchy, gocv.RetrievalList, gocv.ChainApproxSimple)

	return contours
}

// 各 countour から一番外側の polygon を作成する
func calcBoardRectangle(contours gocv.PointsVector, rows, cols int) image.Rectangle {
	allPoints := []image.Point{}
	for i := 0; i < contours.Size(); i++ {
		contour := contours.At(i)
		for j := 0; j < contour.Size(); j++ {
			allPoints = append(allPoints, contour.At(j))
		}
	}

	boardContour := gocv.NewPointVectorFromPoints(allPoints)
	defer boardContour.Close()

	return gocv.BoundingRect(boardContour)
}
