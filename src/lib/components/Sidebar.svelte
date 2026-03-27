<script lang="ts">
  import { currentRoute } from "../stores/router.js";
  import type { Route } from "../types/index.js";

  interface NavItem {
    route: Route;
    label: string;
    icon: string;
    phase?: number; // which implementation phase adds this
  }

  const navItems: NavItem[] = [
    { route: "dashboard", label: "Dashboard", icon: "home" },
    { route: "match-history", label: "Match History", icon: "history", phase: 3 },
    { route: "patterns", label: "Patterns", icon: "brain", phase: 6 },
    { route: "improvement", label: "Improvement", icon: "trending-up", phase: 6 },
    { route: "settings", label: "Settings", icon: "settings" },
  ];

  function navigate(route: Route) {
    currentRoute.set(route);
  }

  let active = $derived($currentRoute);

  const icons: Record<string, string> = {
    home: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6",
    history: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
    brain: "M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z",
    "trending-up": "M13 7h8m0 0v8m0-8l-8 8-4-4-6 6",
    settings: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z",
  };
</script>

<nav class="flex h-full w-56 flex-col border-r" style="background: var(--bg-secondary); border-color: var(--border)">
  <!-- Logo -->
  <div class="flex h-14 items-center gap-3 border-b px-5" style="border-color: var(--border)">
    <div class="flex h-8 w-8 items-center justify-center rounded-lg" style="background: var(--accent-blue)">
      <span class="text-sm font-bold text-white">S</span>
    </div>
    <div>
      <h1 class="text-sm font-semibold" style="color: var(--text-primary)">Sentinel</h1>
      <p class="text-[10px]" style="color: var(--text-muted)">LoL Intelligence</p>
    </div>
  </div>

  <!-- Nav Items -->
  <div class="flex flex-1 flex-col gap-1 p-3">
    {#each navItems as item}
      <button
        onclick={() => navigate(item.route)}
        class="flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors"
        style="background: {active === item.route ? 'var(--bg-tertiary)' : 'transparent'}; color: {active === item.route ? 'var(--text-primary)' : 'var(--text-secondary)'}"
        onmouseenter={(e) => { if (active !== item.route) (e.currentTarget as HTMLElement).style.background = 'var(--bg-hover)' }}
        onmouseleave={(e) => { if (active !== item.route) (e.currentTarget as HTMLElement).style.background = 'transparent' }}
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d={icons[item.icon]} />
        </svg>
        {item.label}
        {#if item.phase && item.phase > 1}
          <span class="ml-auto rounded px-1.5 py-0.5 text-[10px]" style="background: var(--bg-primary); color: var(--text-muted)">Soon</span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Footer -->
  <div class="border-t p-3" style="border-color: var(--border)">
    <p class="text-center text-[10px]" style="color: var(--text-muted)">See everything. Carry nothing.</p>
  </div>
</nav>
