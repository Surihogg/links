// 通过深度链接 links://add 将当前页面推送到 Links 桌面应用

chrome.action.onClicked.addListener((tab) => {
  const pageUrl = tab.url || '';
  if (!pageUrl.startsWith('http://') && !pageUrl.startsWith('https://')) return;

  const deepLink =
    'links://add?url=' + encodeURIComponent(pageUrl) +
    '&title=' + encodeURIComponent(tab.title || '');

  chrome.scripting.executeScript({
    target: { tabId: tab.id },
    func: (link) => {
      const i = document.createElement('iframe');
      i.hidden = true;
      i.src = link;
      document.body.appendChild(i);
      setTimeout(() => i.remove(), 2000);
    },
    args: [deepLink],
  });
});
