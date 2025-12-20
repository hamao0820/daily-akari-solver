// content.jsからメッセージを受け取ってfetchを実行し、結果をcontent.jsに返す
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  // chrome.runtime.onMessage.addListenerのコールバック関数は async にできないので、即時関数でラップする
  (async () => {
    const problemData = await fetchProblemData(request.data.problemNo).catch((err) => {
      console.error("Error fetching problem data:", err);
      sendResponse({ error: err.message });
    });

    const cellPositionData = await fetchCellPositions(request.data.dataURL, problemData).catch((err) => {
      console.error("Error fetching cell positions:", err);
      sendResponse({ error: err.message });
    });

    const solutionData = await fetchSolutionData(problemData).catch((err) => {
      console.error("Error fetching answer data:", err);
      sendResponse({ error: err.message });
    });

    sendResponse({ ...cellPositionData, ...solutionData });
  })();
  return true; // 非同期でsendResponseを呼び出すためにtrueを返す
});

const fetchProblemData = async (problemNo) => {
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

const fetchCellPositions = async (dataURL, problemData) => {
  const body = {
    image_data_base64: dataURL,
    problem_data: problemData,
  };

  const res = await fetch("http://localhost:8080/positions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  return await res.json();
};

const fetchSolutionData = async (problemData) => {
  const body = {
    problem: problemData,
    timeout: 5,
  };
  const res = await fetch("https://akari-solver.kentakom1213.workers.dev", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  return await res.json();
};
