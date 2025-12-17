package detect

import (
	"errors"
	"fmt"
	"image"
	"math"
	"sort"
)

// Cell 判定結果
type Cell struct {
	Rect image.Rectangle
	Row  int
	Col  int
}

// 内部計算用の構造体
type pointInfo struct {
	value   float64 // 中心座標
	origIdx int     // 元の配列でのインデックス
	// assigned int     // 割り当てられたグリッドインデックス
}

func IdentifyGridCells(boardSize image.Point, totalRows, totalCols int, observed []image.Rectangle) ([]Cell, error) {
	if len(observed) == 0 {
		return nil, errors.New("observed rectangles are empty")
	}

	// 1. 中心座標の抽出
	centersX := make([]float64, len(observed))
	centersY := make([]float64, len(observed))
	for i, r := range observed {
		centersX[i] = float64(r.Min.X+r.Max.X) / 2.0
		centersY[i] = float64(r.Min.Y+r.Max.Y) / 2.0
	}

	// 2. X軸（列）の特定 (クラスタリング + 順序維持)
	cols, err := solveAxisRobust(float64(boardSize.X), totalCols, centersX)
	if err != nil {
		return nil, fmt.Errorf("x-axis error: %w", err)
	}

	// 3. Y軸（行）の特定 (クラスタリング + 順序維持)
	rows, err := solveAxisRobust(float64(boardSize.Y), totalRows, centersY)
	if err != nil {
		return nil, fmt.Errorf("y-axis error: %w", err)
	}

	// 4. 結果結合
	results := make([]Cell, len(observed))
	for i := range observed {
		// 重複チェック（任意）：もし (Row, Col) が完全に重複してはいけない仕様ならここでエラーにする
		results[i] = Cell{
			Rect: observed[i],
			Col:  cols[i],
			Row:  rows[i],
		}
	}

	return results, nil
}

// solveAxisRobust はノイズのある1次元座標を、重複しないように適切なグリッドインデックスに割り当てます
func solveAxisRobust(boardLen float64, maxGridCount int, coords []float64) ([]int, error) {
	n := len(coords)
	points := make([]*pointInfo, n)
	for i, v := range coords {
		points[i] = &pointInfo{value: v, origIdx: i}
	}

	// 1. 座標順にソート
	sort.Slice(points, func(i, j int) bool {
		return points[i].value < points[j].value
	})

	// 2. クラスタリング (近接する点を同じ「論理ライン」にまとめる)
	// 閾値: 理想的なマス幅の半分以下なら「同じ行/列」とみなす
	idealPitch := boardLen / float64(maxGridCount)
	clusterThreshold := idealPitch * 0.5

	type cluster struct {
		points   []*pointInfo // このラインに属する点たち
		avgValue float64      // このラインの代表座標
	}
	var clusters []*cluster

	if n > 0 {
		currentCluster := &cluster{points: []*pointInfo{points[0]}}
		for i := 1; i < n; i++ {
			diff := points[i].value - points[i-1].value
			if diff < clusterThreshold {
				// 閾値以内なら同じクラスタに追加
				currentCluster.points = append(currentCluster.points, points[i])
			} else {
				// 閾値を超えたら新しいクラスタを開始
				clusters = append(clusters, currentCluster)
				currentCluster = &cluster{points: []*pointInfo{points[i]}}
			}
		}
		clusters = append(clusters, currentCluster)
	}

	// クラスタの平均座標を計算
	for _, c := range clusters {
		sum := 0.0
		for _, p := range c.points {
			sum += p.value
		}
		c.avgValue = sum / float64(len(c.points))
	}

	// 3. クラスタ群に対して位置合わせ (回帰分析)
	// ここでは「クラスタの順番」と「座標」を使って、全体的なズレとピッチを補正します
	clusterCenters := make([]float64, len(clusters))
	for i, c := range clusters {
		clusterCenters[i] = c.avgValue
	}

	// 単回帰用のデータ作成 (x: 相対的な順番, y: 座標)
	// ただし、クラスタ間に「空行」がある可能性を考慮し、
	// まずは単純に推定ピッチで割って「相対インデックス」を推定します。
	baseVal := clusterCenters[0]
	var xData, yData []float64
	relIndices := make([]int, len(clusters))

	for i, val := range clusterCenters {
		// round((現在地 - 先頭) / ピッチ) で、先頭から何マス離れているか推定
		relIdx := int(math.Round((val - baseVal) / idealPitch))

		// 【重要】順序制約の強制
		// クラスタi は クラスタi-1 より必ず後ろの行でなければならない
		if i > 0 && relIdx <= relIndices[i-1] {
			relIdx = relIndices[i-1] + 1
		}

		relIndices[i] = relIdx
		xData = append(xData, float64(relIdx))
		yData = append(yData, val)
	}

	// 回帰分析で正確なパラメータ(slope, intercept)を出す
	slope, intercept := linearRegression(xData, yData)

	// 4. 全体のシフト量を決定 (盤面に収まるように)
	bestShift := 0
	minError := math.MaxFloat64

	// 探索範囲
	minRel := relIndices[0]
	maxRel := relIndices[len(relIndices)-1]
	startShift := -minRel
	endShift := maxGridCount - 1 - maxRel

	for shift := startShift; shift <= endShift; shift++ {
		// 盤面中心とのズレで評価
		gridCenterIdx := float64(maxGridCount-1) / 2.0
		predictedCenter := slope*(gridCenterIdx-float64(shift)) + intercept
		actualCenter := boardLen / 2.0

		diff := math.Abs(predictedCenter - actualCenter)
		if diff < minError {
			minError = diff
			bestShift = shift
		}
	}

	// 5. 最終的なインデックスを書き戻し
	resultIndices := make([]int, n)
	for i, c := range clusters {
		// このクラスタのインデックスを決定
		finalIdx := relIndices[i] + bestShift

		// 範囲外ガード
		if finalIdx < 0 {
			finalIdx = 0
		}
		if finalIdx >= maxGridCount {
			finalIdx = maxGridCount - 1
		}

		// クラスタ内のすべての点に同じインデックスを付与
		for _, p := range c.points {
			resultIndices[p.origIdx] = finalIdx
		}
	}

	return resultIndices, nil
}

// linearRegression (前回と同様)
func linearRegression(x, y []float64) (slope, intercept float64) {
	n := float64(len(x))
	if n == 0 {
		return 0, 0
	}
	sumX, sumY, sumXY, sumXX := 0.0, 0.0, 0.0, 0.0
	for i := 0; i < len(x); i++ {
		sumX += x[i]
		sumY += y[i]
		sumXY += x[i] * y[i]
		sumXX += x[i] * x[i]
	}
	slope = (n*sumXY - sumX*sumY) / (n*sumXX - sumX*sumX)
	intercept = (sumY - slope*sumX) / n
	return slope, intercept
}
