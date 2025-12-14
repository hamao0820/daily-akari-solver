package main

import (
	"encoding/base64"
	"os"
	"time"

	"github.com/go-rod/rod"
	"github.com/go-rod/rod/lib/proto"
)

func main() {
	browser := rod.New().MustConnect()
	defer browser.MustClose()

	page := browser.MustPage("https://dailyakari.com/")
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

	file, err := os.Create("board.png")
	if err != nil {
		panic(err)
	}
	defer file.Close()

	if _, err := file.Write(data); err != nil {
		panic(err)
	}
}
