<script lang="ts">
  import MetricCard from "$lib/components/MetricCard.svelte";
  import RealtimeChart from "$lib/components/RealtimeChart.svelte";
  import SettingsSheet from "$lib/components/SettingsSheet.svelte";
  import {
    connected,
    connecting,
    telemetry,
    settings,
    deviceInfo,
    error,
    connectDevice,
    disconnectDevice,
    setOutput,
  } from "$lib/stores/device";
  import { language, languages, setLanguage, t, type Language } from "$lib/i18n";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  function startDrag(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.icon-btn')) return;
    getCurrentWindow().startDragging();
  }


  let settingsOpen = $state(false);
  let voltageInput = $state("0.00");
  let currentInput = $state("0.000");
  let voltageDirty = $state(false);
  let currentDirty = $state(false);
  let energyWh = $state(0);
  let lastEnergyTime = $state(0);

  $effect(() => {
    if ($settings && !voltageDirty) voltageInput = $settings.voltage.toFixed(2);
    if ($settings && !currentDirty) currentInput = $settings.current.toFixed(3);
  });

  $effect(() => {
    if ($telemetry) {
      const now = Date.now();
      if (lastEnergyTime > 0) {
        const dt = (now - lastEnergyTime) / 3600000;
        energyWh += $telemetry.power * dt;
      }
      lastEnergyTime = now;
    }
  });

  async function handleConnect() {
    try {
      await connectDevice();
      energyWh = 0;
      lastEnergyTime = 0;
    } catch {
      // error is set in store
    }
  }

  async function toggleOutput() {
    if (!$settings) return;
    await setOutput(!$settings.enabled, $settings.voltage, $settings.current);
  }

  let maxVoltage = $derived($telemetry?.max_voltage ?? 30.5);

  async function applyVoltage() {
    if (!$settings) return;
    let v = parseFloat(voltageInput) || 0;
    v = Math.max(0, Math.min(v, maxVoltage));
    voltageInput = v.toFixed(2);
    await setOutput($settings.enabled, v, $settings.current);
    voltageDirty = false;
  }

  async function applyCurrent() {
    if (!$settings) return;
    let i = parseFloat(currentInput) || 0;
    i = Math.max(0, Math.min(i, 5.05));
    currentInput = i.toFixed(3);
    await setOutput($settings.enabled, $settings.voltage, i);
    currentDirty = false;
  }

  function handleVoltageInput() {
    voltageDirty = true;
  }

  function handleCurrentInput() {
    currentDirty = true;
  }

  function handleVoltageKey(e: KeyboardEvent) {
    if (e.key === "Enter") applyVoltage();
    if (e.key === "Escape") {
      voltageDirty = false;
      if ($settings) voltageInput = $settings.voltage.toFixed(2);
    }
  }

  function handleCurrentKey(e: KeyboardEvent) {
    if (e.key === "Enter") applyCurrent();
    if (e.key === "Escape") {
      currentDirty = false;
      if ($settings) currentInput = $settings.current.toFixed(3);
    }
  }

  function handleLanguageChange(e: Event) {
    setLanguage((e.currentTarget as HTMLSelectElement).value as Language);
  }
</script>

<div class="window">
  <!-- Titlebar -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <header class="titlebar" onmousedown={startDrag}>
    <div class="titlebar-left">
      <!-- Space for traffic lights -->
    </div>
    <div class="titlebar-center">
      {#if $connected && $deviceInfo}
        <span class="titlebar-dot connected"></span>
        <span class="titlebar-title">{$deviceInfo.name}</span>
        <span class="titlebar-subtitle">v{$deviceInfo.firmware_version.toFixed(1)}</span>
      {:else if $connecting}
        <span class="titlebar-dot connecting"></span>
        <span class="titlebar-title">{$t("connecting")}</span>
      {:else}
        <span class="titlebar-dot disconnected"></span>
        <span class="titlebar-title">DP100 Lab</span>
      {/if}
    </div>
    <div class="titlebar-right">
      <select class="language-select" value={$language} onchange={handleLanguageChange} title={$t("language")}>
        {#each languages as option}
          <option value={option.code}>{option.label}</option>
        {/each}
      </select>
      {#if $connected}
        <button class="icon-btn" onclick={() => settingsOpen = true} title={$t("settings")}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M6.5 1.5h3L10 3.2l1.7-.7 2.1 1.2-1 2 .8 1.6 1.9.2v2.4l-1.9.2-.8 1.6 1 2-2.1 1.2-1.7-.7-.5 1.7h-3L6 13.3l-1.7.7-2.1-1.2 1-2-.8-1.6-1.9-.2V6.6l1.9-.2.8-1.6-1-2L4.3 1.6 6 2.3l.5-1.8z" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/>
            <circle cx="8" cy="8.5" r="2" stroke="currentColor" stroke-width="1.1"/>
          </svg>
        </button>
      {/if}
    </div>
  </header>

  <main class="content">
    {#if !$connected && !$connecting}
      <!-- Disconnected State -->
      <div class="empty-state">
        <div class="empty-icon">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <rect x="8" y="14" width="32" height="20" rx="4" stroke="currentColor" stroke-width="2"/>
            <rect x="40" y="20" width="4" height="8" rx="1" fill="currentColor" opacity="0.4"/>
            <path d="M18 24h12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            <path d="M24 20v8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </div>
        <h2 class="empty-title">{$t("connectDp100")}</h2>
        <p class="empty-desc">{$t("connectViaUsb")}</p>
        {#if $error}
          <p class="empty-error">{$error}</p>
        {/if}
        <button class="btn-connect" onclick={handleConnect}>
          {$t("connect")}
        </button>
      </div>
    {:else if $connecting}
      <div class="empty-state">
        <div class="spinner"></div>
        <p class="empty-desc">{$t("lookingForDp100")}</p>
      </div>
    {:else}
      <!-- Dashboard -->
      <div class="dashboard">
        <!-- Metrics Row -->
        <div class="metrics-row">
          <MetricCard
            label={$t("voltage")}
            value={$telemetry?.output_voltage ?? 0}
            unit="V"
            max={$telemetry?.max_voltage ?? 30.5}
            color="var(--color-voltage)"
            badge={$telemetry?.output_mode === "CV" ? "CV" : ""}
            precision={2}
          />
          <MetricCard
            label={$t("current")}
            value={$telemetry?.output_current ?? 0}
            unit="A"
            max={5.1}
            color="var(--color-current)"
            badge={$telemetry?.output_mode === "CC" ? "CC" : ""}
            precision={3}
          />
          <MetricCard
            label={$t("power")}
            value={$telemetry?.power ?? 0}
            unit="W"
            max={155}
            color="var(--color-power)"
            precision={2}
          />
        </div>

        <!-- Chart -->
        <RealtimeChart />

        <!-- Controls Row -->
        <div class="controls-section">
          <div class="controls-row">
            <div class="output-toggle">
              <span class="control-label">{$t("output")}</span>
              <button
                class="toggle-btn"
                class:on={$settings?.enabled}
                onclick={toggleOutput}
              >
                {$settings?.enabled ? $t("on") : $t("off")}
              </button>
            </div>

            <div class="set-fields">
              <div class="set-field">
                <label for="set-v">{$t("setVoltage")} <span class="field-hint">{$t("max", { value: `${maxVoltage.toFixed(1)}V` })}</span></label>
                <div class="input-row">
                  <div class="input-wrap" class:dirty={voltageDirty}>
                    <input
                      id="set-v"
                      type="number"
                      step="0.1"
                      min="0"
                      max={maxVoltage}
                      bind:value={voltageInput}
                      oninput={handleVoltageInput}
                      onkeydown={handleVoltageKey}
                    />
                    <span class="input-unit">V</span>
                  </div>
                  {#if voltageDirty}
                    <button class="btn-apply" onclick={applyVoltage}>{$t("set")}</button>
                  {/if}
                </div>
              </div>
              <div class="set-field">
                <label for="set-i">{$t("setCurrent")} <span class="field-hint">{$t("max", { value: "5.050A" })}</span></label>
                <div class="input-row">
                  <div class="input-wrap" class:dirty={currentDirty}>
                    <input
                      id="set-i"
                      type="number"
                      step="0.1"
                      min="0"
                      max="5.05"
                      bind:value={currentInput}
                      oninput={handleCurrentInput}
                      onkeydown={handleCurrentKey}
                    />
                    <span class="input-unit">A</span>
                  </div>
                  {#if currentDirty}
                    <button class="btn-apply" onclick={applyCurrent}>{$t("set")}</button>
                  {/if}
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Status Bar -->
        <footer class="status-bar">
          <div class="status-item">
            <span class="status-label">{$t("input")}</span>
            <span class="status-value">{($telemetry?.input_voltage ?? 0).toFixed(2)} V</span>
          </div>
          <div class="status-sep"></div>
          <div class="status-item">
            <span class="status-label">{$t("temp")}</span>
            <span class="status-value">{($telemetry?.temperature1 ?? 0).toFixed(1)}° / {($telemetry?.temperature2 ?? 0).toFixed(1)}°</span>
          </div>
          <div class="status-sep"></div>
          <div class="status-item">
            <span class="status-label">{$t("mode")}</span>
            <span class="status-value">{$telemetry?.output_mode ?? "—"}</span>
          </div>
          <div class="status-sep"></div>
          <div class="status-item">
            <span class="status-label">{$t("status")}</span>
            <span class="status-value" class:status-warn={$telemetry?.work_state !== "Normal"}>{$telemetry?.work_state ?? "—"}</span>
          </div>
          <div class="status-sep"></div>
          <div class="status-item">
            <span class="status-label">{$t("energy")}</span>
            <span class="status-value">{energyWh.toFixed(3)} Wh</span>
          </div>
          <div class="status-sep"></div>
          <div class="status-item">
            <span class="status-label">{$t("rail5v")}</span>
            <span class="status-value">{($telemetry?.rail_5v ?? 0).toFixed(2)} V</span>
          </div>
          <div class="status-spacer"></div>
          <button class="disconnect-btn" onclick={disconnectDevice}>
            {$t("disconnect")}
          </button>
        </footer>
      </div>
    {/if}
  </main>
</div>

<SettingsSheet open={settingsOpen} onclose={() => settingsOpen = false} />

<style>
  /* Window Layout */
  .window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-window);
  }

  /* Titlebar */
  .titlebar {
    height: var(--titlebar-height);
    display: flex;
    align-items: center;
    padding: 0 16px;
    flex-shrink: 0;
  }

  .titlebar-left {
    width: 70px;
    flex-shrink: 0;
  }

  .titlebar-center {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .titlebar-right {
    width: 138px;
    flex-shrink: 0;
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 6px;
  }

  .titlebar-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .titlebar-dot.connected {
    background: var(--system-green);
    box-shadow: 0 0 6px rgba(52, 199, 89, 0.4);
  }

  .titlebar-dot.connecting {
    background: var(--system-orange);
    animation: pulse 1.2s infinite;
  }

  .titlebar-dot.disconnected {
    background: var(--system-gray);
  }

  .titlebar-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .titlebar-subtitle {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .icon-btn {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .icon-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .language-select {
    width: 84px;
    height: 28px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    padding: 0 6px;
    outline: none;
  }

  .language-select:focus {
    border-color: var(--system-blue);
  }

  /* Content */
  .content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 8px;
    padding: 40px;
  }

  .empty-icon {
    color: var(--text-tertiary);
    margin-bottom: 8px;
  }

  .empty-title {
    font-size: 20px;
    font-weight: 600;
  }

  .empty-desc {
    font-size: 13px;
    color: var(--text-secondary);
    text-align: center;
  }

  .empty-error {
    font-size: 12px;
    color: var(--system-red);
    background: rgba(255, 59, 48, 0.08);
    border: 1px solid rgba(255, 59, 48, 0.15);
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    margin-top: 4px;
  }

  .btn-connect {
    margin-top: 12px;
    padding: 8px 32px;
    border: none;
    border-radius: var(--radius-md);
    background: var(--system-blue);
    color: white;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s, transform 0.1s;
  }

  .btn-connect:hover {
    opacity: 0.9;
  }

  .btn-connect:active {
    transform: scale(0.97);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2.5px solid var(--bg-tertiary);
    border-top-color: var(--system-blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 8px;
  }

  /* Dashboard */
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 0 16px 16px;
    flex: 1;
    min-height: 0;
  }

  .metrics-row {
    display: flex;
    gap: 10px;
    flex-shrink: 0;
  }

  /* Controls */
  .controls-section {
    background: var(--bg-card);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 14px 16px;
    box-shadow: var(--shadow-card);
    flex-shrink: 0;
  }

  .controls-row {
    display: flex;
    align-items: flex-start;
    gap: 20px;
  }

  .output-toggle {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    padding-top: 2px;
  }

  .control-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .toggle-btn {
    width: 64px;
    height: 36px;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    color: white;
    background: var(--system-gray);
  }

  .toggle-btn.on {
    background: var(--system-green);
    box-shadow: 0 2px 8px rgba(52, 199, 89, 0.25);
  }

  .set-fields {
    display: flex;
    gap: 16px;
    flex: 1;
  }

  .set-field {
    flex: 1;
  }

  .set-field label {
    display: block;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    margin-bottom: 4px;
  }

  .field-hint {
    font-weight: 400;
    text-transform: none;
    color: var(--text-tertiary);
    letter-spacing: 0;
  }

  .input-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .input-wrap {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    flex: 1;
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  .input-wrap:focus-within {
    border-color: var(--system-blue);
    box-shadow: 0 0 0 3px var(--ring-focus);
  }

  .input-wrap.dirty {
    border-color: var(--system-orange);
  }

  .input-wrap input {
    flex: 1;
    border: none;
    background: transparent;
    padding: 6px 8px;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
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

  .btn-apply {
    padding: 5px 10px;
    border: none;
    border-radius: var(--radius-sm);
    background: var(--system-blue);
    color: white;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    transition: opacity 0.15s;
    flex-shrink: 0;
  }

  .btn-apply:hover {
    opacity: 0.85;
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: var(--bg-card);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card);
    flex-shrink: 0;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .status-label {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .status-value {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .status-sep {
    width: 1px;
    height: 14px;
    background: var(--separator);
  }

  .status-warn {
    color: var(--system-red) !important;
    font-weight: 600;
  }

  .status-spacer {
    flex: 1;
  }

  .disconnect-btn {
    padding: 3px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .disconnect-btn:hover {
    background: rgba(255, 59, 48, 0.08);
    border-color: rgba(255, 59, 48, 0.2);
    color: var(--system-red);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
