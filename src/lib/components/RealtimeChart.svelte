<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { history, loggingStatus, startLogging, stopLogging, setPollRate } from "$lib/stores/device";
  import { t } from "$lib/i18n";

  let container: HTMLDivElement;
  let chart: uPlot | null = null;
  let rafId: number;
  let paused = $state(false);
  let pollRate = $state(50); // ms

  function handleRateChange(e: Event) {
    pollRate = parseInt((e.target as HTMLSelectElement).value);
    setPollRate(pollRate);
  }

  function clearChart() {
    history.set([]);
  }
  let showVoltage = $state(true);
  let showCurrent = $state(true);
  let showPower = $state(true);

  // Tooltip state
  let tooltipVisible = $state(false);
  let tooltipX = $state(0);
  let tooltipY = $state(0);
  let tooltipVoltage = $state(0);
  let tooltipCurrent = $state(0);
  let tooltipPower = $state(0);
  let tooltipTime = $state("");

  function isDark() {
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }

  function cssVar(name: string) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  function getColors() {
    const dark = isDark();
    const voltage = cssVar("--color-voltage") || (dark ? "#ffd60a" : "#f5c542");
    const current = cssVar("--color-current") || (dark ? "#30d158" : "#34c759");
    const power = cssVar("--color-power") || (dark ? "#bf5af2" : "#af52de");
    return {
      grid: dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)",
      axis: dark ? "#98989d" : "#86868b",
      voltage,
      current,
      power,
    };
  }

  function tooltipPlugin() {
    return {
      hooks: {
        setCursor: [(u: uPlot) => {
          const idx = u.cursor.idx;
          if (idx == null || idx < 0 || !u.data[0] || idx >= u.data[0].length) {
            tooltipVisible = false;
            return;
          }

          const cx = u.cursor.left ?? 0;
          const cy = u.cursor.top ?? 0;
          if (cx < 0 || cy < 0) {
            tooltipVisible = false;
            return;
          }

          const ts = u.data[0][idx];
          const v = u.data[1]?.[idx] ?? 0;
          const i = u.data[2]?.[idx] ?? 0;
          const p = u.data[3]?.[idx] ?? 0;

          const sec = Math.round(Date.now() / 1000 - ts);
          tooltipTime = sec === 0 ? $t("now") : `-${sec}s`;
          tooltipVoltage = v;
          tooltipCurrent = i;
          tooltipPower = p;

          const rect = container.getBoundingClientRect();
          tooltipX = cx + u.over.offsetLeft;
          tooltipY = cy + u.over.offsetTop;
          tooltipVisible = true;
        }],
      },
    };
  }

  function createChart() {
    if (chart) chart.destroy();
    const c = getColors();
    const width = container.clientWidth;
    const height = getChartHeight();

    const opts: uPlot.Options = {
      width,
      height,
      padding: [12, 16, 0, 0],
      cursor: {
        show: true,
        drag: { x: false, y: false },
        points: { show: true, size: 6, fill: "var(--text-primary)", stroke: "var(--bg-card)", width: 2 },
      },
      legend: { show: false },
      plugins: [tooltipPlugin()],
      axes: [
        {
          stroke: c.axis,
          grid: { stroke: c.grid, width: 1 },
          ticks: { stroke: c.grid, width: 1, size: 4 },
          font: '10px -apple-system, system-ui, sans-serif',
          values: (_self: uPlot, splits: number[]) =>
            splits.map((v: number) => {
              const sec = Math.round(Date.now() / 1000 - v);
              return sec === 0 ? $t("now") : `-${sec}s`;
            }),
        },
        {
          stroke: c.voltage,
          grid: { stroke: c.grid, width: 1 },
          ticks: { stroke: c.grid, width: 1, size: 4 },
          font: '10px -apple-system, system-ui, sans-serif',
          label: "V",
          labelFont: 'bold 10px -apple-system, system-ui, sans-serif',
          size: 46,
        },
        {
          stroke: c.current,
          grid: { show: false },
          ticks: { stroke: c.grid, width: 1, size: 4 },
          font: '10px -apple-system, system-ui, sans-serif',
          label: "A",
          labelFont: 'bold 10px -apple-system, system-ui, sans-serif',
          side: 1,
          size: 46,
          scale: "current",
        },
      ],
      scales: {
        x: { time: false },
        y: { auto: true, range: (_self: uPlot, min: number, max: number) => [Math.max(0, min - 0.5), max + 0.5] },
        current: { auto: true, range: (_self: uPlot, min: number, max: number) => [Math.max(0, min - 0.1), max + 0.1] },
      },
      series: [
        {},
        {
          label: $t("voltage"),
          stroke: c.voltage,
          width: 1.5,
          fill: c.voltage + "10",
          scale: "y",
        },
        {
          label: $t("current"),
          stroke: c.current,
          width: 1.5,
          fill: c.current + "10",
          scale: "current",
        },
        {
          label: $t("power"),
          stroke: c.power,
          width: 1,
          dash: [4, 4],
          scale: "y",
        },
      ],
    };

    chart = new uPlot(opts, [[], [], [], []], container);
  }

  function update() {
    if (!chart) return;
    if (!paused) {
      const h = $history;
      if (h.length > 0) {
        chart.setData([
          h.map((p) => p.time),
          h.map((p) => p.voltage),
          h.map((p) => p.current),
          h.map((p) => p.power),
        ]);
      }
    }
    rafId = requestAnimationFrame(update);
  }

  function toggleSeries(idx: number) {
    if (!chart) return;
    if (idx === 1) showVoltage = !showVoltage;
    else if (idx === 2) showCurrent = !showCurrent;
    else if (idx === 3) showPower = !showPower;
    const show = idx === 1 ? showVoltage : idx === 2 ? showCurrent : showPower;
    chart.setSeries(idx, { show });
  }

  function togglePause() {
    paused = !paused;
  }

  function getChartHeight() {
    if (!container) return 200;
    const parent = container.closest('.chart-container');
    if (!parent) return 200;
    const header = parent.querySelector<HTMLElement>('.chart-header');
    const headerH = header ? header.offsetHeight + 8 : 30;
    return Math.max(100, parent.clientHeight - headerH - 22);
  }

  function handleResize() {
    if (chart && container) {
      chart.setSize({ width: container.clientWidth, height: getChartHeight() });
    }
  }

  onMount(() => {
    createChart();
    rafId = requestAnimationFrame(update);

    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const themeHandler = () => createChart();
    mql.addEventListener("change", themeHandler);

    const ro = new ResizeObserver(() => handleResize());
    const parent = container.closest('.chart-container');
    if (parent) ro.observe(parent);

    return () => {
      mql.removeEventListener("change", themeHandler);
      ro.disconnect();
    };
  });

  $effect(() => {
    $t("language");
    if (container) createChart();
  });

  onDestroy(() => {
    cancelAnimationFrame(rafId);
    chart?.destroy();
  });
</script>

<div class="chart-container">
  <div class="chart-header">
    <span class="chart-title">{$t("output")}</span>
    <div class="chart-controls">
      {#if $loggingStatus.active}
        <span class="rec-info">{$t("recording")} {$t("recordingInfo", { samples: $loggingStatus.samples, duration: Math.floor($loggingStatus.duration_secs) })}</span>
        <button class="ctrl-btn recording" onclick={stopLogging} title={$t("stopRecording")}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="2.5" y="2.5" width="9" height="9" rx="1" fill="currentColor"/>
          </svg>
        </button>
      {:else}
        <button class="ctrl-btn" onclick={startLogging} title={$t("recordToCsv")}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle cx="7" cy="7" r="4.5" fill="var(--system-red)"/>
          </svg>
        </button>
      {/if}
      <button class="ctrl-btn" class:active={paused} onclick={togglePause} title={paused ? $t("resume") : $t("pause")}>
        {#if paused}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3 1.5L12 7L3 12.5V1.5Z" fill="currentColor"/>
          </svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="2" y="1.5" width="3.5" height="11" rx="0.75" fill="currentColor"/>
            <rect x="8.5" y="1.5" width="3.5" height="11" rx="0.75" fill="currentColor"/>
          </svg>
        {/if}
      </button>
      <button class="ctrl-btn" onclick={clearChart} title={$t("clearChart")}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M2.5 4h9M5 4V2.5h4V4M3.5 4v7.5h7V4M6 6.5v3M8 6.5v3" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/>
        </svg>
      </button>
      <select class="rate-select" bind:value={pollRate} onchange={handleRateChange} title={$t("pollingRate")}>
        <option value={20}>{$t("fast")}</option>
        <option value={50}>{$t("mid")}</option>
        <option value={200}>{$t("slow")}</option>
      </select>
    </div>
  </div>
  <div class="chart-area">
    <div class="chart-wrap" bind:this={container}></div>
    {#if tooltipVisible}
      <div
        class="tooltip"

        style:left="{tooltipX}px"
        style:top="{tooltipY}px"
      >
        <div class="tooltip-time">{tooltipTime}</div>
        {#if showVoltage}
          <div class="tooltip-row">
            <span class="tooltip-dot" style:background="var(--color-voltage)"></span>
            {tooltipVoltage.toFixed(2)} V
          </div>
        {/if}
        {#if showCurrent}
          <div class="tooltip-row">
            <span class="tooltip-dot" style:background="var(--color-current)"></span>
            {tooltipCurrent.toFixed(3)} A
          </div>
        {/if}
        {#if showPower}
          <div class="tooltip-row">
            <span class="tooltip-dot" style:background="var(--color-power)"></span>
            {tooltipPower.toFixed(2)} W
          </div>
        {/if}
      </div>
    {/if}
  </div>
  <div class="chart-footer">
    <div class="chart-legend">
      <button class="legend-item" class:off={!showVoltage} onclick={() => toggleSeries(1)}>
        <span class="legend-dot" style:background="var(--color-voltage)"></span>
        {$t("voltage")}
      </button>
      <button class="legend-item" class:off={!showCurrent} onclick={() => toggleSeries(2)}>
        <span class="legend-dot" style:background="var(--color-current)"></span>
        {$t("current")}
      </button>
      <button class="legend-item" class:off={!showPower} onclick={() => toggleSeries(3)}>
        <span class="legend-dot" style:background="var(--color-power)"></span>
        {$t("power")}
      </button>
    </div>
  </div>
</div>

<style>
  .chart-container {
    background: var(--bg-card);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 14px 4px 8px 4px;
    box-shadow: var(--shadow-card);
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .chart-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    margin-bottom: 8px;
    flex-shrink: 0;
  }

  .chart-title {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .chart-controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .chart-footer {
    display: flex;
    justify-content: center;
    padding: 4px 12px 2px;
    flex-shrink: 0;
  }

  .chart-legend {
    display: flex;
    gap: 8px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-secondary);
    background: none;
    border: none;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
    transition: opacity 0.15s, background 0.15s;
  }

  .legend-item:hover {
    background: var(--bg-tertiary);
  }

  .legend-item.off {
    opacity: 0.35;
  }

  .legend-item.off .legend-dot {
    background: var(--system-gray) !important;
  }

  .legend-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }

  .ctrl-btn {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s;
  }

  .ctrl-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .ctrl-btn.active {
    background: var(--system-blue);
    border-color: var(--system-blue);
    color: white;
  }

  .rate-select {
    font-size: 10px;
    padding: 3px 6px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    outline: none;
  }

  .rate-select:focus {
    border-color: var(--system-blue);
  }

  .rec-info {
    font-size: 10px;
    color: var(--system-red);
    font-variant-numeric: tabular-nums;
    animation: rec-pulse 1.2s infinite;
  }

  .ctrl-btn.recording {
    border-color: var(--system-red);
    color: var(--system-red);
    animation: rec-pulse 1.2s infinite;
  }

  @keyframes rec-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .chart-area {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .chart-wrap {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .chart-wrap :global(.u-wrap) {
    border-radius: 6px;
  }

  .tooltip {
    position: absolute;
    pointer-events: none;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 6px 10px;
    box-shadow: var(--shadow-md);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    z-index: 10;
    transform: translate(12px, -50%);
    white-space: nowrap;
  }

  .tooltip-time {
    color: var(--text-tertiary);
    font-size: 10px;
    margin-bottom: 3px;
  }

  .tooltip-row {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--text-primary);
    line-height: 1.5;
    font-family: var(--font-mono);
  }

  .tooltip-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }
</style>
