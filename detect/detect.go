package detect

import (
	"errors"
	"image"
	"image/color"

	"gocv.io/x/gocv"
)

// Cell 判定結果
type Cell struct {
	Center image.Point
	Row    int
	Col    int
}

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

	boardRect := calcBoardRectangle(contours)

	margin := int(20.526 - 0.009*float64(rows*cols))
	croppedRect := image.Rect(boardRect.Min.X+margin, boardRect.Min.Y+margin, boardRect.Max.X-margin, boardRect.Max.Y-margin)

	croppedImg := img.Region(croppedRect)
	defer croppedImg.Close()

	width := croppedImg.Size()[1]
	height := croppedImg.Size()[0]

	centersIntervalX := float64(width) / float64(2*cols)
	centersIntervalY := float64(height) / float64(2*rows)

	cells := make([]Cell, 0, rows*cols)
	for r := 0; r < rows; r++ {
		for c := 0; c < cols; c++ {
			centerX := int(centersIntervalX * float64(2*c+1))
			centerY := int(centersIntervalY * float64(2*r+1))

			// 少し外側にずらす
			if c < cols/2 {
				centerX -= int(centersIntervalX * 0.15)
			} else if c > cols/2 {
				centerX += int(centersIntervalX * 0.15)
			}
			if r < rows/2 {
				centerY -= int(centersIntervalY * 0.15)
			} else if r > rows/2 {
				centerY += int(centersIntervalY * 0.15)
			}

			cells = append(cells, Cell{
				Center: image.Pt(centerX, centerY),
				Row:    r,
				Col:    c,
			})
		}
	}

	// 元の画像座標に変換
	for i := range cells {
		cells[i].Center.X += croppedRect.Min.X
		cells[i].Center.Y += croppedRect.Min.Y
	}

	return cells, nil
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
func calcBoardRectangle(contours gocv.PointsVector) image.Rectangle {
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
