import * as api from "../api.js";

async function isWindowOnScreen(mainWindow) {
  try {
    const { availableMonitors } = await import("@tauri-apps/api/window");
    const monitors = await availableMonitors();
    const pos = await mainWindow.outerPosition();
    const size = await mainWindow.innerSize();
    const wx = pos.x;
    const wy = pos.y;
    const ww = size.width;
    const wh = size.height;
    for (const m of monitors) {
      const a = m.workArea;
      if (wx + ww > a.position.x && wx < a.position.x + a.size.width &&
          wy + wh > a.position.y && wy < a.position.y + a.size.height) {
        return true;
      }
    }
  } catch (e) {}
  return false;
}

export async function restoreWindowState(mainWindow) {
  const { LogicalSize, PhysicalPosition } = await import("@tauri-apps/api/window");

  const savedSize = await api.getSetting("window-size");
  if (savedSize) {
    try {
      const { width, height } = JSON.parse(savedSize);
      await mainWindow.setSize(new LogicalSize(width, height));
    } catch (e) {}
  }

  const savedPos = await api.getSetting("window-position");
  if (savedPos) {
    try {
      const { x, y } = JSON.parse(savedPos);
      await mainWindow.setPosition(new PhysicalPosition(x, y));
      if (!(await isWindowOnScreen(mainWindow))) {
        await mainWindow.center();
      }
    } catch (e) {}
  }
}

export async function setupWindowStateSaving(mainWindow, cleanup) {
  let resize_restore_done = false;
  const resize_restore_timeout = setTimeout(() => { resize_restore_done = true; }, 2000);
  cleanup.add(() => clearTimeout(resize_restore_timeout));

  let resize_timer;

  async function save_window_state() {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      const physicalSize = await win.innerSize();
      const scaleFactor = await win.scaleFactor();
      const logical = physicalSize.toLogical(scaleFactor);
      await api.setSetting("window-size", JSON.stringify({
        width: Math.round(logical.width),
        height: Math.round(logical.height)
      }));
      const pos = await win.outerPosition();
      await api.setSetting("window-position", JSON.stringify({
        x: pos.x,
        y: pos.y
      }));
    } catch (e) {}
  }

  function on_resize() {
    clearTimeout(resize_timer);
    if (!resize_restore_done) return;
    resize_timer = setTimeout(save_window_state, 500);
  }

  window.addEventListener("resize", on_resize);
  cleanup.add(() => {
    window.removeEventListener("resize", on_resize);
    clearTimeout(resize_timer);
  });

  try {
    const unlistenMoved = await mainWindow.onMoved(() => {
      clearTimeout(resize_timer);
      if (!resize_restore_done) return;
      resize_timer = setTimeout(save_window_state, 500);
    });
    cleanup.add(unlistenMoved);
  } catch (e) {
    console.warn("[window] onMoved not available:", e);
  }
}
