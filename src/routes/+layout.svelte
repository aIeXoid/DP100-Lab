<script lang="ts">
  import "../app.css";
  import { setupListeners } from "$lib/stores/device";
  import { onMount } from "svelte";

  let { children } = $props();

  onMount(() => {
    setupListeners();
  });
</script>

{@render children()}
