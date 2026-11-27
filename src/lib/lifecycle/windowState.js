import * as api from "../api.js";

export const DEFAULT_WINDOW_SIZE = Object.freeze({ width: 900, height: 600 });
export const MIN_WINDOW_SIZE = Object.freeze({ width: 800, height: 600 });
const MAX_WINDOW_DIMENSION = 32768;

function parseJson(value) {
  if (typeof value !== "string") return value;
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
}

/** 只接受可实际操作的窗口尺寸，损坏或零尺寸配置回退到默认值。 */
export function normalizeWindowSize(value) {
  const parsed = parseJson(value);
  if (!parsed || typeof parsed !== "object") return null;
  const { width, height } = parsed;
  if (!Number.isFinite(width) || !Number.isFinite(height)) return null;
  if (width < MIN_WINDOW_SIZE.width || height < MIN_WINDOW_SIZE.height) return null;
  if (width > MAX_WINDOW_DIMENSION || height > MAX_WINDOW_DIMENSION) return null;
  return { width: Math.round(width), height: Math.round(height) };
}

export function normalizeWindowPosition(value) {
  const parsed = parseJson(value);
  if (!parsed || typeof parsed !== "object") return null;
  const { x, y } = parsed;
  if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
  return { x: Math.round(x), y: Math.round(y) };
}

async function isWindowOnScreen(mainWindow) {
  try {
    const { availableMonitors } = await import("@tauri-apps/api/window");
    const monitors = await availableMonitors();
    const pos = await mainWindow.outerPosition();
    const size = await mainWindow.innerSize();
    for (const monitor of monitors) {
      const area = monitor.workArea;
      const overlapWidth = Math.min(pos.x + size.width, area.position.x + area.size.width)
        - Math.max(pos.x, area.position.x);
      const overlapHeight = Math.min(pos.y + size.height, area.position.y + area.size.height)
        - Math.max(pos.y, area.position.y);
      // 至少露出完整的标题栏区域；仅相交几个像素仍等同窗口不可见。
      if (overlapWidth >= Math.min(240, size.width)
          && overlapHeight >= Math.min(80, size.height)) {
        return true;
      }
    }
  } catch {
    // 无法读取显示器时由调用方居中，优先保证窗口可见。
  }
  return false;
}

export async function restoreWindowState(mainWindow) {
  const { LogicalSize, PhysicalPosition } = await import("@tauri-apps/api/window");
  const [savedSize, savedPos] = await Promise.all([
    api.getSetting("window-size").catch(() => null),
    api.getSetting("window-position").catch(() => null),
  ]);

  const size = normalizeWindowSize(savedSize) ?? DEFAULT_WINDOW_SIZE;
  // 首次启动和异常配置都显式设置尺寸，避免 Windows 隐藏窗口以零尺寸首绘。
  await mainWindow.setSize(new LogicalSize(size.width, size.height));

  const position = normalizeWindowPosition(savedPos);
  if (!position) return;
  try {
    await mainWindow.setPosition(new PhysicalPosition(position.x, position.y));
    if (!(await isWindowOnScreen(mainWindow))) {
      await mainWindow.center();
    }
  } catch {
    await mainWindow.center().catch(() => {});
  }
}

export async function setupWindowStateSaving(mainWindow, cleanup) {
  let resize_restore_done = false;
  const resize_restore_timeout = setTimeout(() => { resize_restore_done = true; }, 2000);
  cleanup.add(() => clearTimeout(resize_restore_timeout));

  let resize_timer;

  async function save_window_state() {
    try {
      // Windows 隐藏/最小化可能回报 0×0；这种状态绝不能覆盖正常配置。
      if (!(await mainWindow.isVisible()) || await mainWindow.isMinimized()) return;
      const physicalSize = await mainWindow.innerSize();
      const scaleFactor = await mainWindow.scaleFactor();
      const logical = physicalSize.toLogical(scaleFactor);
      const size = normalizeWindowSize({
        width: Math.round(logical.width),
        height: Math.round(logical.height),
      });
      if (!size) return;
      await api.setSetting("window-size", JSON.stringify(size));

      const position = normalizeWindowPosition(await mainWindow.outerPosition());
      if (position) {
        await api.setSetting("window-position", JSON.stringify(position));
      }
    } catch {
      // 窗口关闭过程中的状态读取失败不影响退出流程。
    }
  }

  function schedule_save() {
    clearTimeout(resize_timer);
    if (!resize_restore_done) return;
    resize_timer = setTimeout(save_window_state, 500);
  }

  window.addEventListener("resize", schedule_save);
  cleanup.add(() => {
    window.removeEventListener("resize", schedule_save);
    clearTimeout(resize_timer);
  });

  try {
    const unlistenMoved = await mainWindow.onMoved(schedule_save);
    cleanup.add(unlistenMoved);
  } catch (e) {
    console.warn("[window] onMoved not available:", e);
  }
}
