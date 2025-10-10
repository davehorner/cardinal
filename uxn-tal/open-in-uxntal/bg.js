// ----- Defaults -----
const DEFAULT_SETTINGS = {
  mode: "query",            // "query" | "encoded" | "plain"
  openInCurrentTab: false   // if true, update current tab instead of opening a new one
};

async function getSettings() {
  const stored = await chrome.storage.sync.get(DEFAULT_SETTINGS);
  return { ...DEFAULT_SETTINGS, ...stored };
}

function buildUxntalUrl(pageUrl, settings) {
  switch (settings.mode) {
    case "query":
      // uxntal://open?url=<encoded>
      return "uxntal://open?url=" + encodeURIComponent(pageUrl);
    case "encoded":
      // uxntal://<percent-encoded-full-url>
      return "uxntal://" + encodeURIComponent(pageUrl);
    case "plain":
      // uxntal://https://example.com/...
      return "uxntal://" + pageUrl;
    default:
      return "uxntal://open?url=" + encodeURIComponent(pageUrl);
  }
}

async function openInUxntal(pageUrl) {
  const settings = await getSettings();
  const target = buildUxntalUrl(pageUrl, settings);

  if (settings.openInCurrentTab) {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (tab && tab.id) {
      await chrome.tabs.update(tab.id, { url: target });
    } else {
      await chrome.tabs.create({ url: target });
    }
  } else {
    await chrome.tabs.create({ url: target });
  }
}

// ----- On install: create context menu -----
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "uxntal-open",
    title: "Open in uxntal",
    contexts: ["page"]
  });
});

// Context menu click
chrome.contextMenus.onClicked.addListener((info) => {
  if (info.menuItemId === "uxntal-open" && info.pageUrl) {
    openInUxntal(info.pageUrl);
  }
});

// Toolbar button click
chrome.action.onClicked.addListener(async (tab) => {
  if (tab && tab.url) {
    openInUxntal(tab.url);
  }
});

// Keyboard shortcut
chrome.commands.onCommand.addListener(async (command) => {
  if (command === "open-in-uxntal") {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (tab?.url) openInUxntal(tab.url);
  }
});
