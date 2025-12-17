// content.jsからメッセージを受け取ってfetchを実行し、結果をcontent.jsに返す
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log("Worker received request:", request);

  fetch("http://localhost:8080/positions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request.fetchData),
  })
    .then((res) => res.json())
    .then((data) => {
      console.log("Worker fetched data:", data);
      sendResponse(data);
    })
    .catch((err) => {
      console.error("Fetch error:", err);
      sendResponse({ error: err.message });
    });

  return true; // 非同期レスポンスを有効にする
});
