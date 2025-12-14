package main

import (
	"encoding/base64"
	"flag"
	"net/url"
	"strconv"
	"time"

	"github.com/go-rod/rod"
	"github.com/go-rod/rod/lib/launcher"
	"github.com/go-rod/rod/lib/proto"
	"github.com/hamao0820/daily-akari-solver/detect"
)

var headless bool
var no int

func main() {
	// コマンドライン引数
	flag.BoolVar(&headless, "headless", true, "Run browser in headless mode")
	flag.IntVar(&no, "no", 0, "Puzzle No.")
	flag.Parse()

	u := launcher.New().Headless(headless).MustLaunch()
	browser := rod.New().ControlURL(u).MustConnect()
	defer browser.MustClose()

	pageURL := "https://dailyakari.com"
	if no > 0 {
		pageURL, _ = url.JoinPath(pageURL, "archive", strconv.Itoa(no))
	}
	page := browser.MustPage(pageURL)
	defer page.MustClose()

	if err := page.WaitLoad(); err != nil {
		panic(err)
	}

	for {
		has, _, err := page.Has(".box .ng-star-inserted .button.is-success.is-loading")
		if err != nil {
			panic(err)
		}
		if !has {
			break
		}

		time.Sleep(300 * time.Millisecond)
	}

	elm, err := page.Element(".box .ng-star-inserted .button.is-success")
	if err != nil {
		panic(err)
	}

	if err := elm.Click(proto.InputMouseButtonLeft, 1); err != nil {
		panic(err)
	}

	// 盤面が描画されるまで待機
	if err := page.WaitLoad(); err != nil {
		panic(err)
	}
	time.Sleep(5 * time.Second)

	iframe := page.MustElement("iframe")
	iframePage, err := iframe.Frame()
	if err != nil {
		panic(err)
	}

	// canvas要素のデータURLを取得
	obj, err := iframePage.Eval(`() => {
		const canvas = document.getElementById('canvas');
		return canvas.toDataURL('image/png');
	}`)
	if err != nil {
		panic(err)
	}
	b64 := obj.Value.Str()

	// png画像として保存
	data, err := base64.StdEncoding.DecodeString(b64[len("data:image/png;base64,"):])
	if err != nil {
		panic(err)
	}

	result := detect.Detect(data)
	println(result)
}
