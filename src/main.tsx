import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { Settings } from "./components/settings/Settings";
import { Summary } from "./components/summary/Summary";
import { SubCat } from "./components/cat/SubCat";
import type { CatColor } from "./types/cat";
import "./styles/global.css";

const windowLabel = getCurrentWindow().label;
const isSettings = windowLabel === "settings";
const isSummary = windowLabel === "summary";
const isSubCat = windowLabel.startsWith("cat-sub-");

if (isSettings) {
  document.body.classList.add("settings-window");
}
if (isSummary) {
  document.body.classList.add("summary-window");
}

// sub cat: color from URL query param
const subCatColor = (() => {
  if (!isSubCat) return "white" as CatColor;
  const params = new URLSearchParams(window.location.search);
  return (params.get("color") as CatColor) || "white";
})();

function Root() {
  if (isSettings) return <Settings />;
  if (isSummary) return <Summary />;
  if (isSubCat) return <SubCat color={subCatColor} />;
  return <App />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>
);
