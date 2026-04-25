import { invoke } from "@tauri-apps/api/core";

let backendReady = false;

export async function waitForBackendReady() {
  if (backendReady) return;

  for (let i = 0; i < 50; i++) {
    try {
      await invoke("get_setting", { key: "dark-mode" });
      backendReady = true;
      return;
    } catch (e) {
      if (e.toString().includes("state not managed")) {
        await new Promise(r => setTimeout(r, 100));
      } else {
        backendReady = true;
        return;
      }
    }
  }

  backendReady = true;
}
