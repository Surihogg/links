import { expect, test } from "@playwright/test";
import { installTauriMock } from "./tauriMock.js";

async function openWithMock(page, path = "/", fixture = {}) {
  await page.addInitScript(installTauriMock, fixture);
  await page.goto(path);
}

async function calls(page, command) {
  const all = await page.evaluate(() => window.__LINKS_TEST__.serializedCalls());
  return command ? all.filter((call) => call.cmd === command) : all;
}

test("主窗口支持空格多条件搜索，长查询只显示固定标题", async ({ page }) => {
  await openWithMock(page);
  await expect(page.locator(".header-title")).toHaveText("全部链接");
  await expect(page.locator(".link-card")).toHaveCount(2);

  const longQuery = `Rust   指南 ${"很长的搜索词".repeat(20)}`;
  await page.locator(".search-input").fill(longQuery);
  await expect(page.locator(".header-title")).toHaveText("搜索结果");
  await expect(page.locator(".link-card")).toHaveCount(0);

  await page.locator(".search-input").fill("Rust   指南");
  await expect(page.locator(".link-card")).toHaveCount(1);
  await expect(page.locator(".card-title")).toHaveText("Rust 编程语言");
  const searchCalls = await calls(page, "search_links");
  expect(searchCalls.at(-1).args.params.query).toBe("Rust   指南");
  await expect.poll(() => page.evaluate(() => performance.getEntriesByName("links-search", "measure").length)).toBeGreaterThan(0);
});

test("可以添加链接并在列表中立即看到结果", async ({ page }) => {
  await openWithMock(page);
  await page.locator(".fab").click();
  await page.locator(".url-field input").fill("https://example.com/new");
  await page.locator('input[placeholder="会自动帮你抓取哦"]').fill("新收集的链接");
  await page.getByRole("button", { name: "保存", exact: true }).click();

  await expect(page.locator(".modal")).toHaveCount(0);
  await expect(page.locator(".card-title", { hasText: "新收集的链接" })).toBeVisible();
  const createCalls = await calls(page, "create_link");
  expect(createCalls).toHaveLength(1);
  expect(createCalls[0].args.payload.title).toBe("新收集的链接");
});

test("重复链接提示可直接进入已有链接编辑并保留标签", async ({ page }) => {
  await openWithMock(page);
  await page.locator(".fab").click();
  await page.locator(".url-field input").fill("https://example.com/rust");
  await page.getByRole("button", { name: "编辑已有链接", exact: true }).click();

  const modal = page.locator(".modal");
  await expect(modal.locator(".modal-title")).toHaveText("编辑链接");
  await expect(modal.locator(".url-field input")).toHaveValue("https://example.com/rust");
  await expect(modal.locator('input[placeholder="会自动帮你抓取哦"]')).toHaveValue("Rust 编程语言");
  await expect(modal.locator(".tag-pill", { hasText: "runtime" })).toBeVisible();
  expect(await calls(page, "create_link")).toHaveLength(0);
});

test("QuickAdd 重复链接通过事件跳转主窗口编辑", async ({ page }) => {
  await openWithMock(page, "/quick-add.html", { windowLabel: "quick-add" });
  await page.locator(".url-field input").fill("https://example.com/rust");
  await page.getByRole("button", { name: "编辑已有链接", exact: true }).click();

  await expect.poll(async () => {
    const events = await calls(page, "plugin:event|emit");
    return events.some((call) => call.args.event === "edit-existing-link"
      && call.args.payload.id === 1);
  }).toBe(true);
  const windowCalls = await calls(page);
  expect(windowCalls.some((call) => call.cmd === "plugin:window|show" && call.args.label === "main")).toBe(true);
  expect(windowCalls.some((call) => call.cmd === "plugin:window|hide" && call.args.label === "quick-add")).toBe(true);
});

test("异常窗口尺寸在显示前回退到安全默认值", async ({ page }) => {
  await openWithMock(page, "/", {
    settings: { "window-size": JSON.stringify({ width: 2, height: 2 }) },
  });
  await expect(page.locator(".header-title")).toHaveText("全部链接");
  const windowCalls = await calls(page);
  const resize = windowCalls.find((call) => call.cmd === "plugin:window|set_size");
  const show = windowCalls.find((call) => call.cmd === "plugin:window|show" && call.args.label === "main");
  expect(resize.args.value.Logical).toEqual({ width: 900, height: 600 });
  expect(windowCalls.indexOf(resize)).toBeLessThan(windowCalls.indexOf(show));
});

test("损坏的窗口尺寸 JSON 不会阻止启动", async ({ page }) => {
  await openWithMock(page, "/", { settings: { "window-size": "{broken" } });
  await expect(page.locator(".header-title")).toHaveText("全部链接");
  const [resize] = await calls(page, "plugin:window|set_size");
  expect(resize.args.value.Logical).toEqual({ width: 900, height: 600 });
});

test("冷启动仍有待处理 deep-link 时不显示主窗口", async ({ page }) => {
  await openWithMock(page, "/", {
    // 不设置 startupDeepLink，专门覆盖启动标记尚未到达时的队列兜底。
    pendingDeepLink: { url: "https://example.com/rust", title: "Rust 编程语言" },
  });
  await expect(page.locator(".header-title")).toHaveText("全部链接");
  const windowCalls = await calls(page);
  expect(windowCalls.some((call) => call.cmd === "plugin:window|show" && call.args.label === "main")).toBe(false);
});

test("全局异常显示友好提示并上报 Rust，列表渲染有性能记录", async ({ page }) => {
  await openWithMock(page);
  await expect(page.locator(".link-card")).toHaveCount(2);
  await expect.poll(() => page.evaluate(() => performance.getEntriesByName("link-list-render", "measure").length)).toBeGreaterThan(0);

  await page.evaluate(() => {
    window.dispatchEvent(new ErrorEvent("error", { error: new Error("E2E synthetic failure") }));
  });
  await expect(page.getByRole("status")).toContainText("详细信息已经记到日志里啦");
  await expect.poll(async () => {
    const logs = await page.evaluate(() => window.__LINKS_TEST__.frontendLogs);
    return logs.some((entry) => entry.level === "error"
      && entry.message === "E2E synthetic failure"
      && entry.context.source === "window.error");
  }).toBe(true);
  await expect(page.locator(".link-card")).toHaveCount(2);
});
