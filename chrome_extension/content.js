window.addEventListener("load", async () => {
  const solveButton = document.createElement("button");
  solveButton.innerText = "Solve Akari";
  solveButton.style.position = "fixed";
  solveButton.style.top = "10px";
  solveButton.style.right = "300px";
  solveButton.style.zIndex = "1000";
  document.body.appendChild(solveButton);

  solveButton.addEventListener("click", async () => {
    // canvasの画像データを取得
    const iframe = document.querySelector("iframe");
    const iframeDocument = iframe.contentDocument;
    const canvas = iframeDocument.querySelector("canvas");
    const dataURL = canvas.toDataURL("image/png");

    // URLから問題番号を抽出
    const pageURL = window.location.href;
    const archiveMatch = pageURL.match(/\/archive\/(\d+)/);
    const problemNo = archiveMatch ? parseInt(archiveMatch[1], 10) : -1;

    // service-worker.jsにメッセージを送信
    const fetchData = {
      image_data_base64: dataURL,
      problem_no: problemNo,
    };
    chrome.runtime.sendMessage({ fetchData }, (response) => {
      const cells = response.cells;
      for (const cell of cells) {
        const centerX = (cell.Rect.Min.X + cell.Rect.Max.X) / 2;
        const centerY = (cell.Rect.Min.Y + cell.Rect.Max.Y) / 2;

        const div = document.createElement("div");
        div.style.width = "30px";
        div.style.height = "30px";
        div.style.position = "absolute";
        div.style.left = `${centerX / 2}px`;
        div.style.top = `${centerY / 2}px`;
        div.style.transform = "translate(-50%, -50%)";
        div.style.zIndex = "1000";
        div.style.fontSize = "0.8rem";
        div.textContent = `(${cell.Row},${cell.Col})`;

        document.body.appendChild(div);
      }
    });
  });
});
