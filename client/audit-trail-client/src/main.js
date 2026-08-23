import { invoke } from "@tauri-apps/api/core";

const els = {
  statusPill: document.getElementById("status-pill"),
  refreshBtn: document.getElementById("refresh-btn"),
  settingsBtn: document.getElementById("settings-btn"),
  settingsPanel: document.getElementById("settings-panel"),
  baseUrlInput: document.getElementById("base-url-input"),
  saveSettingsBtn: document.getElementById("save-settings-btn"),
  errorBanner: document.getElementById("error-banner"),
  tableBody: document.getElementById("log-table-body"),
  loadMoreBtn: document.getElementById("load-more-btn"),
};

/** @type {{ nextCursor: string | null, hasNextPage: boolean }} */
let pagination = { nextCursor: null, hasNextPage: false };
/** Accumulated rows currently shown in the table. */
let rows = [];

function setStatus(state, label) {
  els.statusPill.textContent = label ?? state;
  els.statusPill.className = `pill pill-${state}`;
}

function showError(message) {
  if (!message) {
    els.errorBanner.classList.add("hidden");
    els.errorBanner.textContent = "";
    return;
  }
  els.errorBanner.textContent = message;
  els.errorBanner.classList.remove("hidden");
}

function formatTimestamp(isoString) {
  try {
    return new Date(isoString).toLocaleString();
  } catch {
    return isoString;
  }
}

function truncate(value, length = 14) {
  if (!value) return "—";
  return value.length > length ? `${value.slice(0, length)}…` : value;
}

function renderRows() {
  if (rows.length === 0) {
    els.tableBody.innerHTML =
      '<tr><td colspan="7" class="empty-state">Tidak ada log ditemukan.</td></tr>';
    return;
  }

  els.tableBody.innerHTML = rows
    .map((record) => {
      const m = record.metadata;
      return `
        <tr>
          <td>${m.log_sequence_number}</td>
          <td class="mono" title="${record.object_id}">${truncate(record.object_id, 18)}</td>
          <td>${formatTimestamp(m.rotation_timestamp)}</td>
          <td>${m.record_count}</td>
          <td class="mono" title="${m.ipfs_cid}">${truncate(m.ipfs_cid, 18)}</td>
          <td class="mono" title="${m.file_hash}">${truncate(m.file_hash, 18)}</td>
          <td class="mono" title="${m.prev_tx_digest ?? ""}">${truncate(
        m.prev_tx_digest,
        14
      )}</td>
        </tr>
      `;
    })
    .join("");
}

async function loadLogs({ append = false } = {}) {
  setStatus("loading", "memuat…");
  showError(null);
  els.refreshBtn.disabled = true;
  els.loadMoreBtn.disabled = true;

  try {
    const params = {
      cursor: append ? pagination.nextCursor : null,
      limit: 25,
    };
    const response = await invoke("fetch_logs", { params });

    rows = append ? [...rows, ...response.data] : response.data;
    pagination = {
      nextCursor: response.next_cursor ?? null,
      hasNextPage: !!response.has_next_page,
    };

    renderRows();
    els.loadMoreBtn.classList.toggle("hidden", !pagination.hasNextPage);
    setStatus("idle", `${rows.length} log dimuat`);
  } catch (err) {
    setStatus("error", "gagal");
    showError(typeof err === "string" ? err : "Gagal memuat log dari audit-trail.");
    console.error(err);
  } finally {
    els.refreshBtn.disabled = false;
    els.loadMoreBtn.disabled = false;
  }
}

async function loadSettingsIntoForm() {
  try {
    const settings = await invoke("get_settings");
    els.baseUrlInput.value = settings.audit_trail_base_url;
  } catch (err) {
    console.error("Gagal memuat settings", err);
  }
}

async function saveSettings() {
  const audit_trail_base_url = els.baseUrlInput.value.trim();
  if (!audit_trail_base_url) return;

  try {
    await invoke("save_settings", {
      settings: { audit_trail_base_url },
    });
    els.settingsPanel.classList.add("hidden");
    await loadLogs({ append: false });
  } catch (err) {
    showError("Gagal menyimpan pengaturan.");
    console.error(err);
  }
}

els.refreshBtn.addEventListener("click", () => loadLogs({ append: false }));
els.loadMoreBtn.addEventListener("click", () => loadLogs({ append: true }));
els.settingsBtn.addEventListener("click", () => {
  els.settingsPanel.classList.toggle("hidden");
});
els.saveSettingsBtn.addEventListener("click", saveSettings);

// Initial load.
(async () => {
  await loadSettingsIntoForm();
  await loadLogs({ append: false });
})();
