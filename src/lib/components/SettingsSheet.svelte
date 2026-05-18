<script lang="ts">
  import {
    settings,
    deviceInfo,
    systemSettings,
    presets,
    activePresetIndex,
    getAllPresets,
    activatePreset,
    savePreset,
    setOutput,
    setSystemSettings,
    startScan,
    stopScan,
    scanStatus,
  } from "$lib/stores/device";

  interface Props {
    open: boolean;
    onclose: () => void;
  }

  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { language, languages, setLanguage, t, type Language } from "$lib/i18n";

  let { open, onclose }: Props = $props();

  type Tab = "presets" | "protection" | "advanced" | "device" | "prefs" | "about";
  let activeTab = $state<Tab>("presets");

  // Scanning
  let scanMode = $state(1); // 0=current, 1=voltage
  let scanOutVal = $state("1.000");
  let scanStart = $state("0.00");
  let scanEnd = $state("5.00");
  let scanStep = $state("0.50");
  let scanDelay = $state("1000");


  // Debug logging
  let debugEnabled = $state(false);
  let debugLogPath = $state("");

  // System settings
  let sysOtp = $state(80);
  let sysOpp = $state(105);
  let sysBlk = $state(2);
  let sysVol = $state(2);
  let sysRep = $state(true);
  let sysAuto = $state(false);
  let sysDirty = $state(false);

  $effect(() => {
    if ($systemSettings && !sysDirty) {
      sysOtp = $systemSettings.otp;
      sysOpp = $systemSettings.opp;
      sysBlk = $systemSettings.backlight;
      sysVol = $systemSettings.volume;
      sysRep = $systemSettings.reverse_protection;
      sysAuto = $systemSettings.auto_output;
    }
  });

  function markSysDirty() { sysDirty = true; }

  async function applySysSettings() {
    await setSystemSettings(sysOtp, sysOpp, sysBlk, sysVol, sysRep, sysAuto);
    sysDirty = false;
  }

  // Presets
  let presetsLoaded = $state(false);
  let editingPreset = $state<number | null>(null);
  let editV = $state("0");
  let editI = $state("0");
  let editOvp = $state("0");
  let editOcp = $state("0");

  $effect(() => {
    if (open && !presetsLoaded) {
      getAllPresets();
      presetsLoaded = true;
    }
    if (!open) {
      presetsLoaded = false;
      editingPreset = null;
    }
  });

  async function handleActivate(index: number) {
    await activatePreset(index);
    editingPreset = null;
    await getAllPresets();
  }

  function startEdit(index: number) {
    const p = $presets.find((p) => p.index === index);
    if (!p) return;
    editingPreset = index;
    editV = p.voltage.toFixed(2);
    editI = p.current.toFixed(3);
    editOvp = p.ovp.toFixed(2);
    editOcp = p.ocp.toFixed(3);
  }

  async function handleSavePreset() {
    if (editingPreset == null) return;
    let v = Math.max(0, Math.min(parseFloat(editV) || 0, 30.5));
    let i = Math.max(0, Math.min(parseFloat(editI) || 0, 5.05));
    let ov = Math.max(v, Math.min(parseFloat(editOvp) || 0, 30.5));
    let oc = Math.max(i, Math.min(parseFloat(editOcp) || 0, 5.05));
    editV = v.toFixed(2);
    editI = i.toFixed(3);
    editOvp = ov.toFixed(2);
    editOcp = oc.toFixed(3);
    await savePreset(editingPreset, v, i, ov, oc);
    // If saving the active preset, apply to output (keep current on/off state)
    if (editingPreset === $activePresetIndex) {
      await setOutput($settings?.enabled ?? false, v, i);
    }
    // Update store optimistically
    presets.update((list) =>
      list.map((p) =>
        p.index === editingPreset
          ? { ...p, voltage: v, current: i, ovp: ov, ocp: oc }
          : p,
      ),
    );
    editingPreset = null;
  }

  function cancelEdit() {
    editingPreset = null;
  }

  async function handleStartScan() {
    await startScan(
      scanMode,
      parseFloat(scanOutVal) || 0,
      parseFloat(scanStart) || 0,
      parseFloat(scanEnd) || 0,
      parseFloat(scanStep) || 0,
      parseInt(scanDelay) || 1000,
    );
  }

  async function handleStopScan() {
    await stopScan();
  }


  async function toggleDebugLog() {
    const path = await invoke<string>("set_debug_log", { enabled: debugEnabled });
    debugLogPath = path;
  }

  function openCoffeePage() {
    openUrl("https://buymeacoffee.com/aleXoid");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={onclose} onkeydown={handleKeydown}>
    <div class="sheet" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <div class="sheet-header">
        <nav class="tab-bar">
          <button class="tab" class:active={activeTab === "presets"} onclick={() => activeTab = "presets"}>{$t("presets")}</button>
          <button class="tab" class:active={activeTab === "protection"} onclick={() => activeTab = "protection"}>{$t("protection")}</button>
          <button class="tab" class:active={activeTab === "advanced"} onclick={() => activeTab = "advanced"}>{$t("advanced")}</button>
          <button class="tab" class:active={activeTab === "device"} onclick={() => activeTab = "device"}>{$t("device")}</button>
          <button class="tab" class:active={activeTab === "prefs"} onclick={() => activeTab = "prefs"}>{$t("settings")}</button>
          <button class="tab" class:active={activeTab === "about"} onclick={() => activeTab = "about"}>{$t("about")}</button>
        </nav>
        <button class="close-btn" onclick={onclose} aria-label={$t("closeSettings")}>
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M2 2L10 10M10 2L2 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="sheet-body">
        {#if activeTab === "presets"}
          {#if $presets.length > 0}
            <div class="preset-grid">
              {#each $presets as preset}
                <button
                  class="preset-card"
                  class:active={preset.index === $activePresetIndex}
                  class:editing={preset.index === editingPreset}
                  ondblclick={() => handleActivate(preset.index)}
                  onclick={() => startEdit(preset.index)}
                  title={$t("tooltipPreset")}
                >
                  <span class="preset-index">P{preset.index}</span>
                  <div class="preset-params">
                    <span>{preset.voltage.toFixed(2)} V</span>
                    <span>{preset.current.toFixed(3)} A</span>
                    <span class="dim">{$t("ovp")} {preset.ovp.toFixed(1)}</span>
                    <span class="dim">{$t("ocp")} {preset.ocp.toFixed(2)}</span>
                  </div>
                </button>
              {/each}
            </div>

            {#if editingPreset != null}
              <div class="preset-edit">
                <div class="preset-edit-header">
                  <span class="preset-edit-title">{$t("editPreset", { index: editingPreset })}</span>
                  <div class="preset-edit-actions">
                    <button class="btn-small" onclick={cancelEdit}>{$t("cancel")}</button>
                    <button class="btn-small primary" onclick={handleSavePreset}>{$t("save")}</button>
                    <button class="btn-small activate" onclick={() => handleActivate(editingPreset!)}>{$t("activate")}</button>
                  </div>
                </div>
                <div class="edit-fields">
                  <div class="field">
                    <label for="edit-v">{$t("voltage")} (V)</label>
                    <div class="input-wrap"><input id="edit-v" type="number" step="0.01" min="0" max="30.5" bind:value={editV} /></div>
                  </div>
                  <div class="field">
                    <label for="edit-i">{$t("current")} (A)</label>
                    <div class="input-wrap"><input id="edit-i" type="number" step="0.001" min="0" max="5.05" bind:value={editI} /></div>
                  </div>
                  <div class="field">
                    <label for="edit-ovp">{$t("ovpVolt")}</label>
                    <div class="input-wrap"><input id="edit-ovp" type="number" step="0.1" min="0" max="30.5" bind:value={editOvp} /></div>
                  </div>
                  <div class="field">
                    <label for="edit-ocp">{$t("ocpAmp")}</label>
                    <div class="input-wrap"><input id="edit-ocp" type="number" step="0.01" min="0" max="5.05" bind:value={editOcp} /></div>
                  </div>
                </div>
              </div>
            {/if}
          {:else}
            <p class="empty-tab">{$t("connectDevicePresets")}</p>
          {/if}

        {:else if activeTab === "protection"}
          <div class="sys-grid">
            <div class="field">
              <label for="sys-opp">{$t("overPower")}</label>
              <div class="input-wrap">
                <input id="sys-opp" type="number" step="0.1" min="0" max="105" bind:value={sysOpp} oninput={markSysDirty} />
                <span class="input-unit">W</span>
              </div>
            </div>
            <div class="field">
              <label for="sys-otp">{$t("overTemperature")}</label>
              <div class="input-wrap">
                <input id="sys-otp" type="number" step="5" min="50" max="80" bind:value={sysOtp} oninput={markSysDirty} />
                <span class="input-unit">°C</span>
              </div>
            </div>
            <div class="field">
              <label for="sys-backlight">{$t("backlight")}</label>
              <div class="input-wrap">
                <input id="sys-backlight" type="number" step="1" min="0" max="4" bind:value={sysBlk} oninput={markSysDirty} />
              </div>
            </div>
            <div class="field">
              <label for="sys-volume">{$t("volume")}</label>
              <div class="input-wrap">
                <input id="sys-volume" type="number" step="1" min="0" max="4" bind:value={sysVol} oninput={markSysDirty} />
              </div>
            </div>
          </div>
          <div class="toggle-row">
            <label class="toggle-label">
              <input type="checkbox" bind:checked={sysRep} onchange={markSysDirty} />
              <span>{$t("reverseProtection")}</span>
            </label>
            <label class="toggle-label">
              <input type="checkbox" bind:checked={sysAuto} onchange={markSysDirty} />
              <span>{$t("autoOutputOnBoot")}</span>
            </label>
          </div>
          {#if sysDirty}
            <button class="btn-primary" onclick={applySysSettings}>{$t("apply")}</button>
          {/if}

        {:else if activeTab === "advanced"}
          <h4 class="sub-title">{$t("voltageCurrentScanning")}</h4>
          <div class="sys-grid">
            <div class="field">
              <label for="scan-mode">{$t("mode")}</label>
              <select id="scan-mode" class="select-input" bind:value={scanMode}>
                <option value={1}>{$t("voltageScan")}</option>
                <option value={0}>{$t("currentScan")}</option>
              </select>
            </div>
            <div class="field">
              <label for="scan-out">{scanMode === 1 ? $t("fixedCurrent") : $t("fixedVoltage")}</label>
              <div class="input-wrap">
                <input id="scan-out" type="number" step="0.1" bind:value={scanOutVal} />
                <span class="input-unit">{scanMode === 1 ? "A" : "V"}</span>
              </div>
            </div>
            <div class="field">
              <label for="scan-start">{$t("start")}</label>
              <div class="input-wrap">
                <input id="scan-start" type="number" step="0.1" bind:value={scanStart} />
                <span class="input-unit">{scanMode === 1 ? "V" : "A"}</span>
              </div>
            </div>
            <div class="field">
              <label for="scan-end">{$t("end")}</label>
              <div class="input-wrap">
                <input id="scan-end" type="number" step="0.1" bind:value={scanEnd} />
                <span class="input-unit">{scanMode === 1 ? "V" : "A"}</span>
              </div>
            </div>
            <div class="field">
              <label for="scan-step">{$t("step")}</label>
              <div class="input-wrap">
                <input id="scan-step" type="number" step="0.01" bind:value={scanStep} />
                <span class="input-unit">{scanMode === 1 ? "V" : "A"}</span>
              </div>
            </div>
            <div class="field">
              <label for="scan-delay">{$t("delay")}</label>
              <div class="input-wrap">
                <input id="scan-delay" type="number" step="100" min="1" max="9999" bind:value={scanDelay} />
                <span class="input-unit">ms</span>
              </div>
            </div>
          </div>
          <div class="action-row">
            {#if $scanStatus.active}
              <div class="scan-progress">
                <span>{$t("step")} {$scanStatus.current_step + 1}/{$scanStatus.total_steps} - {$scanStatus.current_value.toFixed(2)} {scanMode === 1 ? "V" : "A"}</span>
              </div>
              <button class="btn-primary btn-danger" onclick={handleStopScan}>{$t("stopScan")}</button>
            {:else}
              <button class="btn-primary" onclick={handleStartScan}>{$t("startScan")}</button>
            {/if}
          </div>

        {:else if activeTab === "device"}
          {#if $deviceInfo}
            <div class="info-grid">
              <div class="info-row">
                <span class="info-label">{$t("model")}</span>
                <span class="info-value">{$deviceInfo.name}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$t("serial")}</span>
                <span class="info-value mono">{$deviceInfo.serial}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$t("hardware")}</span>
                <span class="info-value">v{$deviceInfo.hardware_version.toFixed(1)}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$t("firmware")}</span>
                <span class="info-value">v{$deviceInfo.firmware_version.toFixed(1)}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$t("state")}</span>
                <span class="info-value">APP</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$t("manufactured")}</span>
                <span class="info-value">{$deviceInfo.year}-{String($deviceInfo.month).padStart(2, '0')}-{String($deviceInfo.day).padStart(2, '0')}</span>
              </div>
            </div>
          {:else}
            <p class="empty-tab">{$t("connectDeviceInfo")}</p>
          {/if}

        {:else if activeTab === "prefs"}
          <div class="info-grid">
            <div class="info-row">
              <span class="info-label">{$t("language")}</span>
              <select class="lang-select" value={$language} onchange={(e) => setLanguage((e.currentTarget as HTMLSelectElement).value as Language)}>
                {#each languages as option}
                  <option value={option.code}>{option.label}</option>
                {/each}
              </select>
            </div>
            <div class="info-row">
              <span class="info-label">{$t("protocolLogging")}</span>
              <button class="toggle-switch" class:on={debugEnabled} onclick={() => { debugEnabled = !debugEnabled; toggleDebugLog(); }}>
                <span class="toggle-knob"></span>
              </button>
            </div>
            {#if debugEnabled && debugLogPath}
              <p class="info-note">{$t("logPath", { path: debugLogPath })}</p>
            {/if}
          </div>

        {:else if activeTab === "about"}
          <div class="about">
            <img src="/logo.png" alt="DP100 Lab" class="about-logo" />
            <p class="about-version">v0.1.1</p>
            <p class="about-desc">
              {$t("openSourceDesc")}
            </p>
            <button class="btn-coffee" onclick={openCoffeePage}>
              {$t("buyMeACoffee")}
            </button>
            <div class="about-links">
              <div class="info-row">
                <span class="info-label">{$t("stack")}</span>
                <span class="info-value">Tauri · Svelte · Rust</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$t("device")}</span>
                <span class="info-value">ATK-DP100 (USB HID)</span>
              </div>
              <div class="info-row">
                <span class="info-label">{$t("license")}</span>
                <span class="info-value">MIT</span>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 0.15s ease-out;
  }

  .sheet {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.2), 0 2px 12px rgba(0, 0, 0, 0.1);
    width: 600px;
    max-height: 85vh;
    overflow-y: auto;
    animation: slideUp 0.2s ease-out;
  }

  .sheet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px 0;
  }

  .tab-bar {
    display: flex;
    gap: 2px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    padding: 2px;
  }

  .tab {
    padding: 5px 10px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .tab:hover {
    color: var(--text-primary);
  }

  .tab.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06);
  }

  .close-btn {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: none;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s;
    flex-shrink: 0;
  }

  .close-btn:hover {
    background: var(--bg-secondary);
  }

  .sheet-body {
    padding: 16px 20px;
  }

  .empty-tab {
    text-align: center;
    color: var(--text-tertiary);
    font-size: 13px;
    padding: 24px 0;
  }

  /* Presets */
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 6px;
  }

  .preset-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 8px 4px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .preset-card:hover {
    border-color: var(--system-blue);
    background: var(--bg-tertiary);
  }

  .preset-card.active {
    border-color: var(--system-blue);
    background: rgba(0, 122, 255, 0.08);
  }

  .preset-card.editing {
    border-color: var(--system-orange);
    background: rgba(255, 149, 0, 0.08);
  }

  .preset-index {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .preset-card.active .preset-index {
    color: var(--system-blue);
  }

  .preset-params {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px 6px;
    font-size: 9px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .preset-params .dim {
    color: var(--text-tertiary);
    font-size: 8px;
  }

  .preset-edit {
    margin-top: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
  }

  .preset-edit-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  .preset-edit-title {
    font-size: 13px;
    font-weight: 600;
  }

  .preset-edit-actions {
    display: flex;
    gap: 6px;
  }

  .btn-small {
    padding: 4px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-small:hover { background: var(--bg-tertiary); }
  .btn-small.primary { background: var(--system-blue); border-color: var(--system-blue); color: white; }
  .btn-small.primary:hover { opacity: 0.85; }
  .btn-small.activate { background: var(--system-green); border-color: var(--system-green); color: white; }
  .btn-small.activate:hover { opacity: 0.85; }

  .edit-fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  /* Shared */
  .field {
    flex: 1;
  }

  .field label {
    display: block;
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 4px;
  }

  .input-wrap {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    transition: border-color 0.15s;
  }

  .input-wrap:focus-within {
    border-color: var(--system-blue);
    box-shadow: 0 0 0 3px var(--ring-focus);
  }

  .input-wrap input {
    flex: 1;
    border: none;
    background: transparent;
    padding: 6px 8px;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
    outline: none;
    min-width: 0;
  }

  .input-unit {
    padding: 0 8px;
    color: var(--text-tertiary);
    font-size: 12px;
    font-weight: 500;
  }

  .btn-primary {
    margin-top: 10px;
    width: 100%;
    padding: 6px 14px;
    border: none;
    border-radius: var(--radius-sm);
    background: var(--system-blue);
    color: white;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .btn-primary:hover { opacity: 0.85; }

  .btn-danger {
    background: var(--system-red);
  }

  .action-row {
    margin-top: 8px;
  }

  .scan-progress {
    font-size: 12px;
    color: var(--system-blue);
    margin-bottom: 6px;
    font-variant-numeric: tabular-nums;
  }

  .select-input {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
  }

  .select-input:focus {
    border-color: var(--system-blue);
  }

  .info-note {
    font-size: 11px;
    color: var(--text-tertiary);
    margin-top: 10px;
    font-style: italic;
  }

  .sub-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    margin-bottom: 8px;
  }

  .sys-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin-bottom: 10px;
  }

  .toggle-row {
    display: flex;
    gap: 16px;
    margin-bottom: 10px;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-primary);
    cursor: pointer;
  }

  .toggle-label input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--system-blue);
  }

  .info-section {
    margin-top: 16px;
    padding-top: 14px;
    border-top: 1px solid var(--separator);
  }

  .info-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 2px 0;
  }

  .info-label {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .info-value {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 12px;
  }

  /* About */
  .sheet-footer {
    display: flex;
    justify-content: flex-end;
    padding: 8px 16px;
    border-top: 1px solid var(--separator);
  }

  .about-footer {
    margin-top: 12px;
    padding-top: 10px;
    border-top: 1px solid var(--separator);
  }

  .toggle-switch {
    width: 36px;
    height: 20px;
    border-radius: 10px;
    border: none;
    background: var(--bg-tertiary);
    position: relative;
    cursor: pointer;
    transition: background 0.2s;
    padding: 0;
  }

  .toggle-switch.on {
    background: var(--system-green);
  }

  .toggle-knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: white;
    transition: left 0.2s;
    box-shadow: 0 1px 2px rgba(0,0,0,0.2);
  }

  .toggle-switch.on .toggle-knob {
    left: 18px;
  }

  .lang-select {
    padding: 5px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
    cursor: pointer;
  }

  .lang-select:focus {
    border-color: var(--system-blue);
  }

  .about-logo {
    height: 40px;
    margin-bottom: 8px;
  }
  .about {
    text-align: center;
    padding: 16px 0;
  }

  .btn-coffee {
    display: inline-block;
    margin: 14px 0;
    padding: 6px 16px;
    border: none;
    border-radius: var(--radius-sm);
    background: #ffdd00;
    color: #000;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .btn-coffee:hover {
    opacity: 0.85;
  }

  .about-version {
    font-size: 12px;
    color: var(--text-tertiary);
    margin-bottom: 12px;
  }

  .about-desc {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 16px;
    line-height: 1.5;
  }

  .about-links {
    text-align: left;
    padding-top: 12px;
    border-top: 1px solid var(--separator);
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(8px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }
</style>
