// チェックボックスの状態を読み込み
document.addEventListener("DOMContentLoaded", async () => {
  const checkbox = document.getElementById("checkbox");

  // 保存されている状態を読み込む
  const result = await chrome.storage.local.get(["solveWithFairy"]);
  checkbox.checked = result.solveWithFairy || false;

  // チェックボックスの変更を監視して保存
  checkbox.addEventListener("change", async () => {
    await chrome.storage.local.set({ solveWithFairy: checkbox.checked });
    console.log("Auto-solve enabled:", checkbox.checked);
  });
});
