<script lang="ts">
  import { getCountryFlagImgUrl } from '$lib/countryFlags';
  import Globe from 'lucide-svelte/icons/globe';

  export let country: string = '';
  export let size: 'sm' | 'md' = 'sm';

  let hasError = false;

  $: flagUrl = country ? getCountryFlagImgUrl(country) : '';
  $: if (country) hasError = false;
</script>

{#if flagUrl && !hasError}
  <img
    src={flagUrl}
    alt="{country} flag"
    class="country-flag-badge size-{size}"
    loading="lazy"
    on:error={() => (hasError = true)}
  />
{:else}
  <Globe size={size === 'md' ? 14 : 12} class="country-flag-fallback text-secondary" />
{/if}

<style>
  .country-flag-badge {
    display: inline-block;
    object-fit: cover;
    border-radius: 2.5px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.15);
    flex-shrink: 0;
  }

  .country-flag-badge.size-sm {
    width: 16px;
    height: 12px;
  }

  .country-flag-badge.size-md {
    width: 20px;
    height: 15px;
  }

  :global(.country-flag-fallback) {
    flex-shrink: 0;
  }
</style>
