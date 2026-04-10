import { render } from "solid-js/web";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "./App";
import TrayApp from "./TrayApp";

const currentWindow = getCurrentWebviewWindow();

render(
  () => (currentWindow.label === "tray" ? <TrayApp /> : <App />),
  document.getElementById("app")!,
);
