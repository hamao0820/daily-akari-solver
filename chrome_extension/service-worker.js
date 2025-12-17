// content.jsからメッセージを受け取ってfetchを実行し、結果をcontent.jsに返す
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  // chrome.runtime.onMessage.addListenerのコールバック関数は async にできないので、即時関数でラップする
  (async () => {
    const problemData = await fethProblemData(request.data.problemNo).catch((err) => {
      console.error("Error fetching problem data:", err);
      sendResponse({ error: err.message });
    });

    const body = {
      image_data_base64: request.data.dataURL,
      problem_data: problemData,
    };
    try {
      const res = await fetch("http://localhost:8080/positions", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
      });
      const data = await res.json();

      sendResponse(data);
    } catch (err) {
      console.error("Error fetching cell positions:", err);
      sendResponse({ error: err.message });
    }
  })();
  return true; // 非同期でsendResponseを呼び出すためにtrueを返す
});

const fethProblemData = async (problemNo) => {
  const url =
    problemNo == -1
      ? "https://dailyakari.com/dailypuzzle?tz_offset=-540"
      : `https://dailyakari.com/archivepuzzle?number=${problemNo}?tz_offset=-540`;
  const response = await fetch(url);
  const data = await response.json();
  const levelData = data["levelData"];

  // \n\nより手前が問題のデータ
  const problemDataText = levelData.split("\n\n")[0];
  const problemData = problemDataText.split("\n").map((line) => line.split(" "));
  return problemData;
};
