// 主题（亮/暗/跟随系统）共享 store。
//
// 此前 App / QuickAdd / Spotlight 三个窗口各自维护一份 apply_theme + 系统主题
// 监听 + theme-changed 跨窗口事件订阅，逻辑高度雷同且容易漂移。统一抽到此处。
//
// 文件后缀 .svelte.js 让模块顶层可以使用 $state / $derived。

import { listen, emit } from "@tauri-apps/api/event";
import { getSetting, setSetting } from "../api.js";

/** 当前主题模式："system" | "light" | "dark" */
let modeState = $state("system");
/** 派生：当前是否为暗色（system 模式下根据 prefers-color-scheme 决定） */
let darkState = $state(false);

let mq = null;
let onSystemThemeChange = null;
let crossWindowUnlisten = null;
let initialized = false;

function computeDark() {
  if (modeState === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  return modeState === "dark";
}

/**
 * 把当前主题应用到 `<html>` 上。会临时给 root 加 `no-transition`
 * 类，避免主题切换时所有过渡同时播放造成的视觉抖动。
 */
function applyToRoot() {
  darkState = computeDark();
  const root = document.documentElement;
  root.classList.add("no-transition");
  root.classList.toggle("dark", darkState);
  // 强制 reflow 让 no-transition 生效
  // eslint-disable-next-line no-unused-expressions
  root.offsetHeight;
  requestAnimationFrame(() => root.classList.remove("no-transition"));
}

/**
 * 切换主题模式并广播给其它窗口。
 * @param {"system"|"light"|"dark"} next
 * @param {object} [opts]
 * @param {boolean} [opts.persist=true] 是否写入配置
 * @param {boolean} [opts.broadcast=true] 是否向其它窗口发 theme-changed 事件
 */
export async function setThemeMode(next, opts = {}) {
  const { persist = true, broadcast = true } = opts;
  modeState = next;
  applyToRoot();
  if (persist) {
    try { await setSetting("theme-mode", next); } catch {}
  }
  if (broadcast) {
    try { await emit("theme-changed", next); } catch {}
  }
}

/**
 * 一次性初始化主题：
 * 1. 从 config 读取 theme-mode（兼容老的 dark-mode 字段）
 * 2. 应用到 root，并给 `<html>` 加 `theme-ready` 解除初始遮蔽
 * 3. 订阅 prefers-color-scheme 变化（system 模式下自动跟随）
 * 4. 订阅 theme-changed 事件，跨窗口同步
 *
 * 多次调用安全（重复调用会复用已注册的监听器）。
 */
export async function initTheme() {
  if (initialized) return;
  initialized = true;

  let saved = "";
  try { saved = (await getSetting("theme-mode")) || ""; } catch {}
  if (!saved) {
    let legacy = "";
    try { legacy = (await getSetting("dark-mode")) || ""; } catch {}
    if (legacy === "true") saved = "dark";
    else if (legacy === "false") saved = "light";
    else saved = "system";
  }
  modeState = saved;
  applyToRoot();
  document.documentElement.classList.add("theme-ready");

  mq = window.matchMedia("(prefers-color-scheme: dark)");
  onSystemThemeChange = () => {
    if (modeState === "system") applyToRoot();
  };
  if (typeof mq.addEventListener === "function") {
    mq.addEventListener("change", onSystemThemeChange);
  } else if (typeof mq.addListener === "function") {
    // Safari 旧版兼容
    mq.addListener(onSystemThemeChange);
  }

  try {
    crossWindowUnlisten = await listen("theme-changed", (e) => {
      const next = e.payload;
      if (next && next !== modeState) {
        modeState = next;
        applyToRoot();
      }
    });
  } catch {}
}

/** 卸载主题 store 的全部监听（一般窗口关闭时由系统回收，无需手动调） */
export function disposeTheme() {
  if (mq && onSystemThemeChange) {
    if (typeof mq.removeEventListener === "function") {
      mq.removeEventListener("change", onSystemThemeChange);
    } else if (typeof mq.removeListener === "function") {
      mq.removeListener(onSystemThemeChange);
    }
  }
  crossWindowUnlisten?.();
  initialized = false;
  mq = null;
  onSystemThemeChange = null;
  crossWindowUnlisten = null;
}

export const themeStore = {
  /** 当前主题模式（响应式）。 */
  get mode() { return modeState; },
  /** 当前是否暗色（响应式）。 */
  get isDark() { return darkState; },
  setMode: setThemeMode,
  init: initTheme,
  dispose: disposeTheme,
};
