import * as api from "../api.js";

export async function checkForUpdates(check_for_update) {
  const auto_check_update = await api.getSetting("auto-check-update");
  if (auto_check_update !== "false") {
    check_for_update();
  }
}

export async function checkVersionNotes(callbacks) {
  const { setCurrentVersion, setLastUpdateNotes, setShowReleaseNotes } = callbacks;
  try {
    const last_version = await api.getSetting("last-known-version");
    const { getVersion } = await import("@tauri-apps/api/app");
    const current_version = await getVersion();
    setCurrentVersion(current_version);
    if (last_version && last_version !== current_version) {
      let notes = "";
      try {
        notes = await api.fetchReleaseNotes(`v${current_version}`) || "";
      } catch (e) {
        console.warn("[update] failed to fetch release notes for post-update:", e);
      }
      if (!notes) {
        notes = await api.getSetting("last-update-notes") || "";
      }
      if (notes) {
        setLastUpdateNotes(notes);
        setShowReleaseNotes(true);
      }
    }
    await api.setSetting("last-known-version", current_version);
  } catch (e) {
    console.warn("[update] version check failed:", e);
  }
}
