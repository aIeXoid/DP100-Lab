<script lang="ts">
  import "../app.css";
  import { setupListeners } from "$lib/stores/device";
  import { language } from "$lib/i18n";
  import { onMount } from "svelte";

  let { children } = $props();

  onMount(() => {
    setupListeners();

    const unsubscribe = language.subscribe((value) => {
      document.documentElement.lang = value;
    });

    return unsubscribe;
  });
</script>

{@render children()}
