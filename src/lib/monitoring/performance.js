import { logFrontendEvent } from "../api.js";

let measureSequence = 0;
const MAX_MEASURES_PER_NAME = 100;

/** 开始一次 Performance API 测量；返回值只能交给对应的 endPerformanceMeasure。 */
export function startPerformanceMeasure(name) {
  measureSequence += 1;
  const token = {
    name,
    startedAt: performance.now(),
    startMark: `${name}:start:${measureSequence}`,
    endMark: `${name}:end:${measureSequence}`,
  };
  performance.mark(token.startMark);
  return token;
}

/**
 * 记录耗时供开发者工具 Performance 面板查看；慢操作额外写 Rust 日志。
 * details 只应包含数量、页码、耗时等非内容数据，不记录搜索词或链接标题。
 */
export function endPerformanceMeasure(token, { details = {}, slowThreshold = 300 } = {}) {
  if (!token) return 0;
  performance.mark(token.endMark);
  performance.measure(token.name, token.startMark, token.endMark);
  const duration = Math.max(0, performance.now() - token.startedAt);
  performance.clearMarks(token.startMark);
  performance.clearMarks(token.endMark);

  const measures = performance.getEntriesByName(token.name, "measure");
  if (measures.length > MAX_MEASURES_PER_NAME) {
    // 保留最近一次，防止长期运行时 Performance entry 无限增长。
    performance.clearMeasures(token.name);
    performance.measure(token.name, {
      start: token.startedAt,
      duration,
    });
  }

  if (duration >= slowThreshold) {
    void logFrontendEvent("warn", `[perf] ${token.name}`, {
      duration_ms: Math.round(duration * 100) / 100,
      ...details,
    }).catch(() => {});
  }
  return duration;
}

export async function measureAsync(name, operation, options = {}) {
  const token = startPerformanceMeasure(name);
  try {
    return await operation();
  } finally {
    endPerformanceMeasure(token, options);
  }
}
