const DEFAULTS = {
  mode: "query",
  openInCurrentTab: false
};

async function load() {
  const stored = await chrome.storage.sync.get(DEFAULTS);
  const mode = stored.mode || DEFAULTS.mode;
  document.querySelectorAll('input[name="mode"]').forEach(r => {
    r.checked = (r.value === mode);
  });
  document.getElementById('openInCurrentTab').checked = !!stored.openInCurrentTab;
}

async function save(e) {
  e.preventDefault();
  const mode = document.querySelector('input[name="mode"]:checked')?.value || DEFAULTS.mode;
  const openInCurrentTab = document.getElementById('openInCurrentTab').checked;

  await chrome.storage.sync.set({ mode, openInCurrentTab });

  const status = document.getElementById('status');
  status.style.visibility = 'visible';
  setTimeout(() => { status.style.visibility = 'hidden'; }, 900);
}

document.getElementById('form').addEventListener('submit', save);
load();
