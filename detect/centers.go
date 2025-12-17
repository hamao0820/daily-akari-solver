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

// IdentifyGridCells 観測された矩形の行・列を特定します
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

	// 2. X軸（列）の特定 (サイズ超過時は自動補正)
	cols, err := solveAxisAdaptive(float64(boardSize.X), totalCols, centersX)
	if err != nil {
		return nil, fmt.Errorf("x-axis identification failed: %w", err)
	}

	// 3. Y軸（行）の特定 (サイズ超過時は自動補正)
	rows, err := solveAxisAdaptive(float64(boardSize.Y), totalRows, centersY)
	if err != nil {
		return nil, fmt.Errorf("y-axis identification failed: %w", err)
	}

	// 4. 結果結合
	results := make([]Cell, len(observed))
	for i := range observed {
		results[i] = Cell{
			Rect: observed[i],
			Col:  cols[i],
			Row:  rows[i],
		}
	}

	return results, nil
}

// 内部計算用の構造体
type pointInfo struct {
	value   float64
	origIdx int
}

// solveAxisAdaptive は、座標群を指定された最大グリッド数内に収まるようマッピングします。
// 必要に応じてピッチを調整し、盤面サイズを超えないようにします。
func solveAxisAdaptive(boardLen float64, maxGridCount int, coords []float64) ([]int, error) {
	n := len(coords)
	if n == 0 {
		return []int{}, nil
	}

	points := make([]*pointInfo, n)
	for i, v := range coords {
		points[i] = &pointInfo{value: v, origIdx: i}
	}

	// 1. 座標順にソート
	sort.Slice(points, func(i, j int) bool {
		return points[i].value < points[j].value
	})

	// 2. クラスタリング (ノイズ除去)
	idealPitch := boardLen / float64(maxGridCount)
	clusterThreshold := idealPitch * 0.45 // 少し許容値を広げる

	type cluster struct {
		points   []*pointInfo
		avgValue float64
	}
	var clusters []*cluster
	currentCluster := &cluster{points: []*pointInfo{points[0]}}

	for i := 1; i < n; i++ {
		diff := points[i].value - points[i-1].value
		if diff < clusterThreshold {
			currentCluster.points = append(currentCluster.points, points[i])
		} else {
			clusters = append(clusters, currentCluster)
			currentCluster = &cluster{points: []*pointInfo{points[i]}}
		}
	}
	clusters = append(clusters, currentCluster)

	// クラスタ数のチェック (鳩の巣原理)
	if len(clusters) > maxGridCount {
		return nil, fmt.Errorf("too many unique positions (%d) observed for board size %d", len(clusters), maxGridCount)
	}

	// クラスタ中心の計算
	clusterCenters := make([]float64, len(clusters))
	for i, c := range clusters {
		sum := 0.0
		for _, p := range c.points {
			sum += p.value
		}
		c.avgValue = sum / float64(len(c.points))
		clusterCenters[i] = c.avgValue
	}

	// 3. 相対インデックスの計算とピッチの自動調整
	// 最初に idealPitch で計算し、収まらなければピッチを広げて再試行する
	currentPitch := idealPitch
	var relIndices []int

	// 最大10回程度のリトライで収める（通常は1回で補正される）
	for attempt := 0; attempt < 10; attempt++ {
		relIndices = calcRelativeIndices(clusterCenters, currentPitch)

		// 必要な幅を計算
		width := relIndices[len(relIndices)-1] - relIndices[0] + 1

		if width <= maxGridCount {
			// 収まった場合はループを抜ける
			break
		}

		// 収まらなかった場合: ピッチを広げて再計算
		// width / maxGridCount の比率で拡大すれば理論上は収まるはずだが、
		// 微妙な誤差を考慮して少し大きめ(1.05倍)に補正する
		ratio := float64(width) / float64(maxGridCount)
		// 補正係数が1.0以下にならないようにガード
		if ratio < 1.01 {
			ratio = 1.01
		}
		currentPitch = currentPitch * ratio
	}

	// 最終チェック
	finalWidth := relIndices[len(relIndices)-1] - relIndices[0] + 1
	if finalWidth > maxGridCount {
		// リトライしても収まらない場合（極端な外れ値がある場合など）
		// 強制的に詰める処理（最後の手段）
		relIndices = compressIndices(relIndices, maxGridCount)
	}

	// 4. 回帰分析とシフト探索
	// ピッチは補正後のものを使う可能性があるため、回帰分析で現在のデータに最適な係数を出し直す
	var xData, yData []float64
	for i, val := range clusterCenters {
		xData = append(xData, float64(relIndices[i]))
		yData = append(yData, val)
	}
	slope, intercept := linearRegression(xData, yData)

	// シフト探索範囲
	minRel := relIndices[0]
	maxRel := relIndices[len(relIndices)-1]
	startShift := -minRel
	endShift := maxGridCount - 1 - maxRel

	bestShift := 0
	minError := math.MaxFloat64
	foundValidShift := false

	for shift := startShift; shift <= endShift; shift++ {
		gridCenterIdx := float64(maxGridCount-1) / 2.0
		predictedCenter := slope*(gridCenterIdx-float64(shift)) + intercept
		actualCenter := boardLen / 2.0

		diff := math.Abs(predictedCenter - actualCenter)
		if diff < minError {
			minError = diff
			bestShift = shift
			foundValidShift = true
		}
	}

	if !foundValidShift {
		// ここに来ることは稀だが、万が一の場合は左詰めで返す
		bestShift = -minRel
	}

	// 5. 結果の格納
	resultIndices := make([]int, n)
	for i, c := range clusters {
		finalIdx := relIndices[i] + bestShift

		// 最終安全装置
		if finalIdx < 0 {
			finalIdx = 0
		}
		if finalIdx >= maxGridCount {
			finalIdx = maxGridCount - 1
		}

		for _, p := range c.points {
			resultIndices[p.origIdx] = finalIdx
		}
	}

	return resultIndices, nil
}

// calcRelativeIndices 指定されたピッチで相対インデックスを計算するヘルパー関数
func calcRelativeIndices(values []float64, pitch float64) []int {
	indices := make([]int, len(values))
	baseVal := values[0]

	for i, val := range values {
		relIdx := int(math.Round((val - baseVal) / pitch))

		// 順序制約: 前の要素より必ず大きくする
		if i > 0 && relIdx <= indices[i-1] {
			relIdx = indices[i-1] + 1
		}
		indices[i] = relIdx
	}
	return indices
}

// compressIndices どうしても収まらない場合に、無理やり隙間を詰める関数
func compressIndices(indices []int, maxCount int) []int {
	n := len(indices)
	if n == 0 {
		return indices
	}

	currentWidth := indices[n-1] - indices[0] + 1
	excess := currentWidth - maxCount

	if excess <= 0 {
		return indices
	}

	// 新しいスライスを作成
	newIndices := make([]int, n)
	copy(newIndices, indices)

	// 後ろから順に、前の要素との差が2以上あるところを詰めていく
	// ※単純な実装として、過剰分がなくなるまで隙間を1つずつ減らす
	for k := 0; k < excess; k++ {
		// 最も隙間（差分）が大きい箇所を探す
		maxGap := 0
		maxGapIdx := -1
		for i := 1; i < n; i++ {
			gap := newIndices[i] - newIndices[i-1]
			if gap > 1 && gap > maxGap {
				maxGap = gap
				maxGapIdx = i
			}
		}

		if maxGapIdx != -1 {
			// maxGapIdx以降の全てのインデックスを1減らす
			for j := maxGapIdx; j < n; j++ {
				newIndices[j]--
			}
		} else {
			// 隙間がない（すべて隣接している）のに収まらない場合はどうしようもない
			// (鳩の巣チェックで弾かれているはずなのでここは到達しないはず)
			break
		}
	}
	return newIndices
}

// linearRegression 単回帰分析
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
	denom := n*sumXX - sumX*sumX
	if math.Abs(denom) < 1e-9 {
		return 0, sumY / n
	}
	slope = (n*sumXY - sumX*sumY) / denom
	intercept = (sumY - slope*sumX) / n
	return slope, intercept
}
