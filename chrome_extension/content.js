window.addEventListener("load", async () => {
  const solveButton = document.createElement("button");
  solveButton.innerText = "✨ Solve Akari";
  solveButton.style.position = "fixed";
  solveButton.style.top = "10px";
  solveButton.style.right = "20px";
  solveButton.style.zIndex = "1000";
  solveButton.style.padding = "12px 24px";
  solveButton.style.fontSize = "16px";
  solveButton.style.fontWeight = "600";
  solveButton.style.color = "#fff";
  solveButton.style.background = "linear-gradient(135deg, #667eea 0%, #764ba2 100%)";
  solveButton.style.border = "none";
  solveButton.style.borderRadius = "25px";
  solveButton.style.cursor = "pointer";
  solveButton.style.boxShadow = "0 4px 15px rgba(102, 126, 234, 0.4)";
  solveButton.style.transition = "all 0.3s ease";
  solveButton.style.fontFamily = "system-ui, -apple-system, sans-serif";

  // ホバー効果
  solveButton.addEventListener("mouseenter", () => {
    solveButton.style.transform = "translateY(-2px)";
    solveButton.style.boxShadow = "0 6px 20px rgba(102, 126, 234, 0.6)";
  });

  solveButton.addEventListener("mouseleave", () => {
    solveButton.style.transform = "translateY(0)";
    solveButton.style.boxShadow = "0 4px 15px rgba(102, 126, 234, 0.4)";
  });

  // アクティブ状態
  solveButton.addEventListener("mousedown", () => {
    solveButton.style.transform = "translateY(0) scale(0.95)";
  });

  solveButton.addEventListener("mouseup", () => {
    solveButton.style.transform = "translateY(-2px) scale(1)";
  });

  document.body.appendChild(solveButton);

  solveButton.addEventListener("click", async () => {
    // ボタンを無効化
    solveButton.disabled = true;
    solveButton.style.opacity = "0.6";
    solveButton.style.cursor = "not-allowed";
    solveButton.innerText = "⏳ Solving...";

    try {
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
      const data = {
        dataURL,
        problemNo,
      };
      const response = await chrome.runtime.sendMessage({ data });
      const cells = response.cells;
      const solutions = response.solution;

      const img = document.createElement("img");
      img.src = chrome.runtime.getURL("image/妖精.png");
      img.style.width = "32px";
      img.style.height = "32px";
      img.style.position = "absolute";
      img.style.top = "10px";
      img.style.right = "200px";
      img.style.zIndex = "1000";
      img.style.transition = "transform 0.1s ease-out";

      // イージング関数（ease-in-out）
      const easeInOutCubic = (t) => {
        return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
      };

      for (const solution of solutions) {
        document.body.appendChild(img);

        const cell = cells.find((cell) => cell.Row === solution[0] && cell.Col === solution[1]);
        if (!cell) {
          console.error("Cell not found for solution:", solution);
          continue;
        }

        const centerX = (cell.Rect.Min.X + cell.Rect.Max.X) / 2;
        const centerY = (cell.Rect.Min.Y + cell.Rect.Max.Y) / 2;

        const clientX = centerX / 2;
        const clientY = centerY / 2;

        // 開始位置を取得
        const imgRect = img.getBoundingClientRect();
        const startX = imgRect.left + imgRect.width / 2;
        const startY = imgRect.top + imgRect.height / 2;

        // 目標までの距離と角度を計算
        const totalDeltaX = clientX - startX;
        const totalDeltaY = clientY - startY;
        const totalDistance = Math.sqrt(totalDeltaX * totalDeltaX + totalDeltaY * totalDeltaY);
        const angle = Math.atan2(totalDeltaY, totalDeltaX);

        // ベジェ曲線の制御点（弧を描く軌道を作る）
        // 垂直方向の制御点オフセットを増やして、より美しい弧を描くようにする
        const controlPointX = startX + totalDeltaX * 0.5 + Math.sin(angle + Math.PI / 2) * totalDistance * 0.3;
        const controlPointY = startY + totalDeltaY * 0.5 + Math.cos(angle + Math.PI / 2) * totalDistance * 0.3;

        // アニメーションの総時間とステップ
        const duration = Math.min(2000, totalDistance * 4); // 距離に応じた時間（ゆっくり）
        const steps = Math.ceil(duration / 16); // 約60fps
        let currentStep = 0;

        // アニメーションループ
        while (currentStep <= steps) {
          const progress = currentStep / steps;
          const easedProgress = easeInOutCubic(progress);

          // ベジェ曲線上の位置を計算
          const t = easedProgress;
          const x = Math.pow(1 - t, 2) * startX + 2 * (1 - t) * t * controlPointX + Math.pow(t, 2) * clientX;
          const y = Math.pow(1 - t, 2) * startY + 2 * (1 - t) * t * controlPointY + Math.pow(t, 2) * clientY;

          // ふわふわとした上下運動を追加（控えめに）
          const floatOffset = Math.sin(progress * Math.PI * 3) * 2;

          // 軽微な回転のみ（進行方向に少し傾く程度）
          const tiltAngle = Math.sin(progress * Math.PI) * 5; // -5度から+5度の範囲

          // 画像の位置と回転を更新
          img.style.left = `${x - imgRect.width / 2}px`;
          img.style.top = `${y - imgRect.height / 2 + floatOffset}px`;
          img.style.transform = `rotate(${tiltAngle}deg) scale(${1 + Math.sin(progress * Math.PI) * 0.08})`;

          currentStep++;
          await new Promise((resolve) => setTimeout(resolve, 16));
        }

        // 最終位置に正確に配置
        img.style.left = `${clientX - imgRect.width / 2}px`;
        img.style.top = `${clientY - imgRect.height / 2}px`;
        img.style.transform = `rotate(0deg) scale(1)`;

        // クリックイベントをシミュレート
        const mousedownEvent = new MouseEvent("mousedown", {
          clientX,
          clientY,
          bubbles: true,
          cancelable: true,
          view: window,
        });
        canvas.dispatchEvent(mousedownEvent);

        await new Promise((resolve) => setTimeout(resolve, 100));
      }

      document.body.removeChild(img);
    } finally {
      // ボタンを再度有効化
      solveButton.disabled = false;
      solveButton.style.opacity = "1";
      solveButton.style.cursor = "pointer";
      solveButton.innerText = "✨ Solve Akari";
    }
  });
});
