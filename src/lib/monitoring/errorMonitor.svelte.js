import { logFrontendEvent } from "../api.js";

let notice = $state("");
let lastErrorKey = "";
let lastErrorAt = 0;
let monitoring = false;

function normalizeError(error) {
  if (error instanceof Error) {
    return { message: error.message || error.name, stack: error.stack || "" };
  }
  if (typeof error === "string") return { message: error, stack: "" };
  try {
    const serialized = JSON.stringify(error);
    return { message: serialized ?? String(error), stack: "" };
  } catch {
    return { message: String(error), stack: "" };
  }
}

/** 上报错误并给用户非技术化提示；相同错误 5 秒内只记录一次。 */
export async function reportFrontendError(error, context = "unknown", notify = true) {
  const normalized = normalizeError(error);
  const key = `${context}:${normalized.message}:${normalized.stack}`;
  const now = Date.now();
  if (key === lastErrorKey && now - lastErrorAt < 5000) return;
  lastErrorKey = key;
  lastErrorAt = now;

  console.error(`[frontend:${context}]`, error);
  if (notify) {
    notice = "刚才的操作没有完成，请再试一次。详细信息已经记到日志里啦。";
  }
  try {
    await logFrontendEvent("error", normalized.message, {
      source: context,
      stack: normalized.stack,
    });
  } catch {
    // 浏览器预览或后端尚未就绪时只保留 console，避免日志失败形成二次异常。
  }
}

export function dismissErrorNotice() {
  notice = "";
}

export const errorNotice = {
  get message() {
    return notice;
  },
};

/** 捕获未处理异常和 Promise rejection；返回解绑函数供窗口卸载时调用。 */
export function initErrorMonitoring() {
  if (monitoring) return () => {};
  monitoring = true;

  const onError = (event) => {
    // 忽略 favicon 等资源加载错误，只处理 JavaScript 运行时异常。
    if (!event.error) return;
    void reportFrontendError(event.error, "window.error");
  };
  const onUnhandledRejection = (event) => {
    void reportFrontendError(event.reason, "unhandledrejection");
  };
  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onUnhandledRejection);

  return () => {
    window.removeEventListener("error", onError);
    window.removeEventListener("unhandledrejection", onUnhandledRejection);
    monitoring = false;
  };
}
