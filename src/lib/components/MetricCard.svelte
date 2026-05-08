<script lang="ts">
  interface Props {
    label: string;
    value: number;
    unit: string;
    max: number;
    color: string;
    badge?: string;
    precision?: number;
  }

  let { label, value, unit, max, color, badge = "", precision = 2 }: Props = $props();

  let fraction = $derived(Math.min(value / max, 1));
  let display = $derived(value.toFixed(precision));
</script>

<div class="card">
  <div class="card-header">
    <span class="card-label">{label}</span>
    {#if badge}
      <span class="card-badge" style:background={color}>{badge}</span>
    {/if}
  </div>
  <div class="card-value">
    <span class="number" style:color={color}>{display}</span>
    <span class="unit">{unit}</span>
  </div>
  <div class="progress-track">
    <div
      class="progress-fill"
      style:width="{fraction * 100}%"
      style:background={color}
    ></div>
  </div>
  <div class="card-range">
    <span>0</span>
    <span>{max} {unit}</span>
  </div>
</div>

<style>
  .card {
    background: var(--bg-card);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 14px 16px 12px;
    box-shadow: var(--shadow-card);
    flex: 1;
    min-width: 0;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .card-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .card-badge {
    font-size: 10px;
    font-weight: 600;
    color: white;
    padding: 1px 6px;
    border-radius: 4px;
    letter-spacing: 0.2px;
  }

  .card-value {
    display: flex;
    align-items: baseline;
    gap: 4px;
    margin-bottom: 10px;
  }

  .number {
    font-family: var(--font-rounded);
    font-size: 32px;
    font-weight: 600;
    line-height: 1;
    font-variant-numeric: tabular-nums;
    transition: color 0.2s;
  }

  .unit {
    font-size: 15px;
    font-weight: 500;
    color: var(--text-tertiary);
  }

  .progress-track {
    height: 4px;
    background: var(--bg-tertiary);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: 6px;
  }

  .progress-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.15s ease-out;
    opacity: 0.85;
  }

  .card-range {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }
</style>
