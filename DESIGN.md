# SENTINEL -- League of Legends Competitive Intelligence Platform

## Design Document v1.0

---

# Step 1: The Problem

## What's broken for LoL players today

**1. Performance tax is unacceptable.**
Every major companion app is built on Electron or Overwolf. Real-world memory consumption:

| App | RAM (measured) | Notes |
|-----|---------------|-------|
| Blitz | 800-1000 MB, leaks to 8 GB | Documented FPS drops of 10-20% |
| Mobalytics | 1400 MB baseline, spikes to 2000+ | Overwolf dependency adds overhead |
| Porofessor | ~1000 MB | Overwolf-based, reliability issues |
| iTero | ~650 MB | Lightest current option |

Players on 8-16 GB systems are forced to choose between companion data and smooth gameplay. This is not a minor annoyance -- it directly impacts the activity the tool is supposed to enhance.

**2. Information without insight.**
Every tool shows the same data: win rates, build paths, rune pages, opponent ranks. This was novel in 2018. It's commodity in 2026. The problem isn't access to data -- it's that no tool tells you what to DO with it, specific to YOUR gameplay.

A player who consistently throws 2k gold leads doesn't need a build optimizer. They need to understand WHY they throw leads and WHEN it happens.

**3. Generic recommendations.**
"Darius counters Irelia (54.2% WR)" is useless if you have 12 games on Darius and 400 on Irelia. Current tools ignore individual champion pools, recent performance trajectories, and opponent-specific tendencies. They optimize for the average player. Average advice produces average results.

**4. Post-game is a dead zone.**
Every tool gives you a stat screen after the game. KDA, damage, gold. The same numbers the client already shows. No tool answers the questions that matter:
- What specific decisions cost me this game?
- Where did my lead evaporate?
- Am I making the same mistake I made last week?

**5. Ads and bloat are the business model.**
Blitz devotes 25-40% of screen to ads. Mobalytics uses pop-ups. Overwolf injects its own UI. The user is the product, not the customer.

## Where current tools fail

| Tool | Strength | Critical Failure |
|------|----------|-----------------|
| U.GG | Clean data, respected accuracy | Pure stats site, no personal insight, no desktop integration |
| Blitz | Auto rune/build import, large dataset | Memory leaks, aggressive ads, black screen crashes, declining quality |
| Mobalytics | Feature-rich, GPI score attempt | Massive resource usage, Overwolf lock-in, reviews trending to 1-star |
| Porofessor | Pre-game scouting | Founder left, stagnating, Overwolf dependent, scouting errors |
| iTero | AI drafting, lowest resources | Relatively new, limited pattern analysis, still 650 MB |
| LoLDraftAI | Specialized draft modeling | Draft-only, no broader game analysis |
| STATUP.GG | AI vision-based voice coaching | Requires screen capture (potential policy risk), narrow focus |

**Common pattern**: every tool either does pre-game scouting (commodity) or tries to do everything (bloat). None focus on the hardest, most valuable problem: understanding YOUR specific patterns and helping you improve.

## What high-ELO players actually need

From community analysis and competitive player feedback:

1. **Draft intelligence tied to champion pools** -- not "X beats Y" but "YOU on X vs THIS opponent on Y given both your recent performance"
2. **Cross-game behavioral pattern detection** -- "You consistently lose games you're ahead at 15 minutes" with the specific mechanism identified
3. **Wave management analysis** from timeline data -- back timings, CS differentials at key intervals, resource allocation patterns
4. **Positional/pathing heatmaps** reconstructed from 1-min position snapshots in timeline data
5. **Practice-to-performance connection** -- track whether working on a specific weakness is producing measurable results
6. **Transparent reasoning** -- show WHY a recommendation is made, not just that it has a high win rate

None of these exist today.

---

# Step 2: Product Vision

## Sentinel is a personal gameplay intelligence engine.

Not a stats dashboard. Not a build importer. Not an overlay.

**Core thesis**: The most impactful tool for competitive improvement is one that understands YOUR specific patterns, connects them across games, and surfaces the specific decisions that determine YOUR outcomes.

### Design principles

1. **Personal, not generic.** Every insight is about YOUR gameplay on YOUR champions against YOUR opponents. Generic win rates are noise.

2. **Decision-enhancing, not information-dumping.** Instead of showing 30 stats, surface the 1-2 decisions that mattered most. Instead of showing a build, explain why this build in THIS game.

3. **Zero performance cost.** Under 100 MB RAM. Sub-second startup. No overlays. No ads. Players should forget it's running.

4. **Local-first.** Your data lives on your machine. The pattern engine runs locally. No required cloud account. Privacy by architecture.

5. **Explain everything.** Every recommendation comes with a reason. "Build MR here" is useless. "Build MR because their 14/2 Syndra is the only threat and your team has no engage to lock her down" is coaching.

### Why it's sticky

- **The pattern engine improves with YOUR data over time.** After 50 games, it knows your tendencies. After 200, it knows your failure modes. This creates a personal data moat -- switching means losing that history.
- **It's the lightest app in the category by 10x.** Once players experience a companion app that doesn't tank their FPS, they don't go back.
- **Post-game insights create a habit loop.** Play game -> see what you did wrong -> play another game trying to fix it -> check if you improved. This is inherently retentive.

---

# Step 3: Core Features

## Feature 1: Personal Pattern Engine

**What it does**: Analyzes your match history using Riot API timeline data (position every 60s, gold, XP, CS, kills, deaths, objectives, items with timestamps) to detect recurring behavioral patterns across games.

**Examples of detected patterns**:
- "On Irelia, you die to ganks 2.3x more than average when you have a CS lead > 15. You're over-extending when winning lane."
- "Your win rate drops from 62% to 38% on games lasting > 30 minutes on Renekton. You're not converting early advantages."
- "You back an average of 45 seconds later than optimal after dragon fights. This costs you 400-600 gold in missed waves."
- "When your team is behind at 15 min, you take 40% more solo fights than when ahead. Your ARAM tendency increases when losing."

**Why it matters**: This is the difference between "here are your stats" and "here is what's holding you back." No tool does this. A human coach would take 5 game reviews to identify these patterns. The engine does it automatically across your entire match history.

**Why it's hard to copy**: Requires deep timeline data processing, statistical modeling for significance testing (you need to distinguish real patterns from noise), and accumulation of per-player data. The engine gets better the more games it analyzes for each specific player. It's not a database query -- it's behavioral analysis.

**Data sources**: Riot API match-v5 timeline data (position snapshots, gold/XP at each minute, item events, kill/death/assist events with positions, ward events, building events).

## Feature 2: Contextual Draft Assistant

**What it does**: During champion select, provides draft recommendations personalized to BOTH teams' actual players.

**How it differs from existing tools**:
- Current: "Darius has 54% WR vs Irelia" (generic stat)
- Sentinel: "YOU have a 67% WR on Darius (23 games) but YOUR opponent has 71% WR on Irelia (89 games, 3 rank tiers above you on this champion). Your Jax is a better pick here -- 72% WR, 45 games, and their player has only 8 Irelia games this season."

**Factors in**:
- Your champion mastery and recent performance (last 20 games on each champion)
- Opponent's champion pool depth and recent performance
- Team composition synergy (engage, peel, damage type balance)
- YOUR historical performance in similar team compositions
- Patch-specific data (recent buffs/nerfs reflected in your own stats)

**Why it matters**: Draft is the highest-leverage phase of the game. A 5% improvement in draft win rate translates directly to LP. Current tools give you population-level data. Sentinel gives you player-level intelligence.

**Why it's hard to copy**: Requires combining real-time LCU champ select data, per-player historical analysis, and composition modeling. The personal data angle means the recommendations improve as the player uses the tool more.

## Feature 3: Automated Post-Game Analysis

**What it does**: After each game, processes the match timeline to identify the 2-3 key decision points that most impacted the outcome. Presents them as a structured narrative, not a stat sheet.

**Example output**:
```
GAME SUMMARY: Loss (31:42) -- Gold lead lost

KEY MOMENT 1 (12:30): You were 2.1k gold ahead in lane.
Between 12:30 and 16:00, you died twice to roaming mid+jungle while
pushing without vision. Gold lead: 2.1k -> -300.
PATTERN MATCH: This matches your "overextend when winning" pattern
(detected in 34% of your Irelia losses).

KEY MOMENT 2 (22:15): Baron fight. Your team started baron at 40% HP
after a 4v5 fight. You arrived 8 seconds late (backing for item).
Enemy team stole baron + aced. Gold swing: -3.2k.
SUGGESTION: In your last 10 games, delayed baron arrivals correlate
with backing within 30s of objective spawn. Track objective timers
against your back timing.

IMPROVEMENT CHECK: Your "vision before pushing" metric improved 15%
this week compared to last week. The overextend pattern is becoming
less frequent.
```

**Why it matters**: Players who want to improve don't know WHERE to look. Watching a 30-minute replay is time-consuming and unfocused. This extracts the signal from the noise automatically and connects it to ongoing patterns.

**Why it's hard to copy**: Requires the Pattern Engine (Feature 1) to contextualize moments against historical behavior. The narrative generation requires understanding game state, not just events.

## Feature 4: Live Game Companion (Second Screen)

**What it does**: During an active game, displays real-time game intelligence on a second monitor or phone (via local network). Uses the Game Client API (localhost:2999) polling at 1 Hz.

**Displays**:
- Team gold differential with trend graph
- Item completion tracking (who's about to finish a power spike item)
- Objective timers with your team's estimated ability to contest (based on current gold/level/position)
- Contextual item suggestions based on actual game state ("Enemy Syndra is 8/1, consider Banshee's next instead of Deathcap")
- Power spike indicators ("Their Kayle hits 16 in ~2 minutes based on current XP rate")

**What it does NOT do**:
- No overlay (Riot is increasingly restrictive; March 2025 banned enemy ult timers, May 2025 banned in-game overlay ads)
- No automated actions
- No information that isn't derivable from the in-game scoreboard (just presented more usefully)

**Why it matters**: The in-game scoreboard gives you raw numbers. This translates numbers into decisions. "Should we force baron?" becomes answerable by looking at gold state, item completions, and objective timers in one view.

**Why it's hard to copy**: The contextual item suggestions require understanding game state holistically. Current tools show "highest win rate build" regardless of the actual game happening.

## Feature 5: Improvement Tracker

**What it does**: Tracks specific performance metrics over time, tied to goals the player sets or the Pattern Engine suggests.

**Metrics tracked**:
- CS/min trend (overall and per champion, normalized by game length)
- Death timing patterns (early deaths, mid-game deaths, late-game caught-out deaths)
- Vision score relative to role average at your rank
- Objective participation rate
- Gold efficiency (gold earned vs gold spent effectively)
- Champion pool performance evolution

**Key differentiator**: Connects to the Pattern Engine's detected weaknesses. If the engine detects "you over-extend when ahead," the tracker monitors that specific metric week-over-week and shows whether you're improving.

**Why it matters**: Improvement in League is slow and invisible without measurement. Players grind hundreds of games without knowing if they're getting better at the specific things that matter. This makes improvement visible and trackable.

---

# Step 4: Platform Decision

## Desktop application. No alternative.

Rationale:
- Must read the League client lockfile (local filesystem)
- Must connect to LCU WebSocket (localhost)
- Must poll Game Client API (localhost:2999)
- Must run persistently alongside the League client
- Must minimize resource usage (web apps can't)

## Framework: Tauri v2

### Comparison

| Criteria | Tauri v2 | Electron | Pure Rust (egui/iced) |
|----------|---------|----------|----------------------|
| **Idle RAM** | 30-80 MB | 120-300 MB | 10-30 MB |
| **Active RAM** | 80-150 MB | 200-400 MB | 30-80 MB |
| **Installer size** | 2.5-10 MB | 85-150 MB | 2-5 MB |
| **Startup time** | < 1 second | 1-4 seconds | < 0.5 seconds |
| **Idle CPU** | ~1% | ~1% + Chromium GC spikes | < 0.5% |
| **UI development speed** | Fast (web tech) | Fast (web tech) | Slow (no HTML/CSS) |
| **Rust backend access** | Native | Via N-API bindings | Native |
| **Ecosystem maturity** | Good (v2 stable) | Excellent | Emerging |
| **Solo dev feasibility** | High | High | Low-Medium |
| **Auto-update** | Built-in plugin | Mature (electron-builder) | Manual implementation |

### Why Tauri v2 wins

**1. Performance is the product differentiator.**
At 50-100 MB RAM, Sentinel would use 10-15x less memory than Blitz/Mobalytics. This isn't just nice -- it's the single most marketable feature for first adoption. "A companion app that doesn't kill your FPS" is a message that sells itself. Electron cannot deliver this.

**2. Rust backend is architecturally correct.**
The pattern engine processes thousands of match timelines, runs statistical analysis, and manages persistent local storage. Rust handles this without garbage collection pauses, memory leaks, or unpredictable performance. The LCU WebSocket connection, Game Client API polling, and Riot API calls all benefit from Rust's async runtime (tokio).

**3. Web frontend maintains development velocity.**
Pure Rust GUI (egui/iced) would produce the lightest possible app, but at severe cost to development speed and UI polish. A solo developer needs to ship an attractive, responsive UI quickly. Web technologies (HTML/CSS/Svelte) provide this. Tauri gives you the web frontend without paying the full Electron tax.

**4. Windows WebView2 is pre-installed.**
On Windows 10 (1803+) and all Windows 11, Edge WebView2 is part of the OS. Tauri doesn't bundle a browser engine -- it uses the system's. This is why the installer is 2.5-10 MB vs Electron's 85-150 MB.

**5. Security model aligns with our needs.**
Tauri's capability-based permission system means the app only has access to what's explicitly configured. This is a good story for users who are wary of companion apps having broad system access.

### Why not the others

**Electron**: Defeats the core value proposition. If Sentinel uses 300 MB like every other app, the performance story dies. The Electron ecosystem advantage (e.g., `league-connect` npm package) saves maybe 2 days of development -- not worth the permanent architectural cost.

**Pure Rust GUI**: The right technical choice but the wrong business choice for a solo developer. 2-6 months to achieve UI parity with what Svelte delivers in days. Could be a future migration target if the product succeeds and warrants the investment.

---

# Step 5: Tech Stack

## Backend (Rust)

```
Runtime:       tokio (async runtime for all I/O)
HTTP client:   reqwest (Riot API, Game Client API)
WebSocket:     tokio-tungstenite (LCU WebSocket)
Serialization: serde + serde_json (all data handling)
Database:      rusqlite (SQLite via Rust bindings)
LCU connect:   Custom (lockfile detection + process query)
Logging:       tracing (structured logging)
Error:         thiserror + anyhow (error handling)
```

### Why these choices

- **tokio**: Industry standard Rust async runtime. LCU WebSocket, Game Client API polling, and Riot API requests all run concurrently without threads.
- **reqwest**: Built on tokio/hyper. Handles TLS (required for both LCU and Game Client API self-signed certs), connection pooling, and timeouts.
- **rusqlite**: SQLite is the correct database for local-first single-user apps. Zero configuration, single file, ACID transactions, full SQL for complex pattern queries. No external database server.
- **serde**: Non-negotiable for Rust JSON handling. All API responses (LCU, Riot, Game Client) are JSON.

## Frontend (Svelte 5)

```
Framework:     Svelte 5 (compiled, no virtual DOM)
Build:         Vite (fast dev server, optimized builds)
Styling:       Tailwind CSS (utility-first, small bundle)
Charts:        Lightweight charting lib (e.g., LayerCake or custom SVG)
State:         Svelte runes ($state, $derived, $effect)
IPC:           @tauri-apps/api (Rust <-> JS communication)
```

### Why Svelte over React/Vue/Solid

- **Bundle size**: Svelte compiles components to vanilla JS. No runtime framework code shipped. ~5-15 KB vs React's ~45 KB minimum.
- **Performance**: No virtual DOM diffing. Direct DOM manipulation. For a companion app updating at 1 Hz, this doesn't matter much -- but the smaller bundle means faster load.
- **Component scoping**: CSS is scoped by default. No CSS-in-JS runtime overhead.
- **Developer experience**: Less boilerplate than React. Reactive by default. A solo developer moves faster in Svelte.
- **Svelte 5 runes**: The new reactivity system ($state, $derived) is cleaner and more predictable than Svelte 4's implicit reactivity.

SolidJS was the other candidate (smaller runtime at ~7 KB, fine-grained reactivity). Svelte wins on ecosystem maturity, documentation, and developer community size.

## Local Data (SQLite)

```
Schema:
├── matches          (match metadata, outcome, duration)
├── match_timelines  (per-minute snapshots: gold, xp, cs, position)
├── match_events     (kills, deaths, objectives, item purchases)
├── patterns         (detected behavioral patterns with confidence)
├── champions        (static champion data per patch)
├── items            (static item data per patch)
├── player_profiles  (opponent data cache)
└── improvement_goals (tracked metrics and history)
```

Local SQLite is the correct choice for:
- No server dependency for core features
- Sub-millisecond query latency for pattern analysis
- Full SQL for complex analytical queries (window functions, CTEs)
- Single-file backup/portability
- Zero operational cost

## Cloud (Optional, Minimal)

```
CDN only:      Patch data updates (champion/item changes per LoL patch)
No auth:       Core features require no account
No telemetry:  No data leaves the machine by default
Future:        Optional aggregate statistics API (what do Diamond players build?)
```

The cloud layer is deliberately minimal. This is both a technical choice (reduce dependencies, eliminate latency/downtime risk) and a product choice (privacy as a feature).

### Tradeoffs acknowledged

| Decision | Tradeoff |
|----------|----------|
| Rust backend | Steeper learning curve, slower iteration vs JS. Justified by performance requirements and pattern engine compute. |
| Svelte | Smaller ecosystem than React. Acceptable for a focused UI with limited third-party component needs. |
| SQLite | Single-user only, no concurrent access. Perfect for this use case. |
| No cloud | Can't show aggregate stats without it. Ship without, add later if needed. |
| Tauri | WebView2 memory can occasionally approach Electron levels. Monitor and optimize. |

---

# Step 6: Data Layer

## Data Sources

### 1. LCU API (Local -- League Client)

**Connection**: Read lockfile at `C:\Riot Games\League of Legends\lockfile` -> parse `PID:PORT:PASSWORD:https` -> connect with Basic Auth (`riot:<password>`).

**Fallback**: Query `LeagueClientUx.exe` process for `--app-port` and `--remoting-auth-token` flags.

**TLS**: Self-signed cert. Pin Riot's root cert (`riotgames.pem`) rather than disabling verification.

**Key endpoints used**:

| Endpoint | Purpose | Frequency |
|----------|---------|-----------|
| `/lol-gameflow/v1/gameflow-phase` | Detect game state transitions | WebSocket event-driven |
| `/lol-champ-select/v1/session` | Champion select data (picks, bans, team) | WebSocket event-driven |
| `/lol-summoner/v1/current-summoner` | Current player identity | On client connect |
| `/lol-ranked/v1/current-ranked-stats` | Current rank/LP | On client connect + post-game |
| `/lol-end-of-game/v1/eog-stats-block` | Post-game stats | WebSocket event-driven |
| `/lol-champion-mastery/v1/local-player/champion-mastery` | Mastery data | On client connect |

**WebSocket**: Connect to `wss://127.0.0.1:<PORT>/` with Basic Auth. Subscribe via WAMP 1.0: `[5, "OnJsonApiEvent"]`. Events arrive as `[8, "EventName", {"uri": "...", "eventType": "Create|Update|Delete", "data": {...}}]`.

### 2. Game Client API (Local -- Game Process)

**Connection**: `https://127.0.0.1:2999/liveclientdata/` -- no auth required, localhost only.

**Available during active games only.**

| Endpoint | Data | Poll Rate |
|----------|------|-----------|
| `/allgamedata` | Full game state (expensive) | On-demand only |
| `/activeplayer` | Your stats, gold, level, abilities | 1 Hz |
| `/playerlist` | All 10 players: champion, items, level, team | 1 Hz |
| `/playerscores?riotId=Name#TAG` | KDA, CS, ward score | 1 Hz |
| `/eventdata` | Timestamped events (kills, dragons, etc.) | 1 Hz |
| `/gamestats` | Game mode, elapsed time | 1 Hz (or less) |

**Important constraints**:
- REST only, no WebSocket -- must poll
- No player positions during live game
- No ability usage data
- No combat interaction details
- Read-only (cannot affect game state)

**Optimization**: Don't poll `/allgamedata` repeatedly. Instead, poll specific endpoints and maintain a local composite state. At 1 Hz, the performance impact is negligible.

### 3. Riot API (Remote)

**Authentication**: API key in request header. Production keys have higher limits.

**Rate limits**:
- Development: 20 requests / 1 second, 100 requests / 2 minutes
- Production: 500 requests / 10 seconds, 30,000 requests / 10 minutes
- Per-region enforcement

**Key endpoints**:

| Endpoint | Purpose | Caching |
|----------|---------|---------|
| `match-v5/matches/by-puuid/{puuid}/ids` | Get match IDs for a player | Cache IDs, check for new periodically |
| `match-v5/matches/{matchId}` | Full match data | Cache permanently (matches don't change) |
| `match-v5/matches/{matchId}/timeline` | Minute-by-minute timeline | Cache permanently |
| `league-v4/entries/by-summoner/{id}` | Rank data | Cache 5 minutes |
| `champion-mastery-v4/champion-masteries/by-puuid/{puuid}` | Mastery scores | Cache 1 hour |
| `spectator-v5/active-games/by-summoner/{puuid}` | Active game lookup | No cache (real-time) |

**Timeline data structure** (the core of our pattern engine):
```json
{
  "info": {
    "frameInterval": 60000,
    "frames": [
      {
        "timestamp": 60000,
        "participantFrames": {
          "1": {
            "position": {"x": 1234, "y": 5678},
            "totalGold": 875,
            "currentGold": 375,
            "xp": 450,
            "minionsKilled": 12,
            "jungleMinionsKilled": 0,
            "level": 2,
            "damageStats": { ... }
          }
        },
        "events": [
          {"type": "ITEM_PURCHASED", "timestamp": 62000, "participantId": 1, "itemId": 1055},
          {"type": "CHAMPION_KILL", "timestamp": 74000, "killerId": 3, "victimId": 7, "position": {...}}
        ]
      }
    ]
  }
}
```

This gives us: position every 60 seconds, gold/XP/CS per minute, every kill/death/assist with position, every item purchase with timestamp, every objective take, ward placements. Enough for deep pattern analysis.

### 4. Static Data (Community Dragon / Data Dragon)

- Champion data: stats, abilities, splash art
- Item data: stats, costs, build paths
- Rune data: trees, keystones, stats
- Patch notes: what changed

Updated per LoL patch (~every 2 weeks). Downloaded once, cached locally.

## Caching Strategy

```
LAYER 1 -- Memory (Rust structs)
├── Current game state (from Game Client API)
├── Current champ select session (from LCU WebSocket)
├── Current player identity and rank
└── Active computations (pattern analysis in progress)

LAYER 2 -- SQLite (persistent)
├── Match history (permanent cache -- matches never change)
├── Match timelines (permanent cache)
├── Computed patterns (recomputed when new matches arrive)
├── Player profiles (opponents, cached 24 hours)
├── Champion/item data (cached per patch)
└── Improvement tracking data (permanent)

LAYER 3 -- Filesystem
├── Static assets (images, patch data)
└── Configuration
```

## Rate Limit Handling

```rust
// Conceptual rate limiter
struct RiotApiClient {
    // Token bucket per rate limit tier
    short_limit: TokenBucket,  // 20 req/s (dev) or 500 req/10s (prod)
    long_limit: TokenBucket,   // 100 req/2min (dev) or 30k req/10min (prod)
    retry_queue: PriorityQueue<ApiRequest>,
}

// Priority system:
// 1. Active game lookups (spectator-v5) -- immediate
// 2. Current champ select opponents -- high
// 3. Recent match fetch (post-game) -- medium
// 4. Historical backfill -- low (background, fills gaps over time)
```

Strategy:
- Never burst. Spread requests evenly across the rate window.
- Exponential backoff on 429 responses (with Retry-After header).
- Match data is permanently cached. A match ID fetched once never needs fetching again.
- Historical backfill runs as background task at lowest priority.
- Prioritize recency: fetch last 20 matches first, backfill older matches gradually.

---

# Step 7: Architecture

## High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         TAURI APPLICATION                           │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    SVELTE FRONTEND                            │   │
│  │                                                              │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────────┐   │   │
│  │  │  Draft    │ │ Live Game│ │ Post-Game│ │  Improvement  │   │   │
│  │  │  View     │ │ Companion│ │ Analysis │ │  Dashboard    │   │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────┬────────┘   │   │
│  │       │             │            │               │            │   │
│  │  ─────┴─────────────┴────────────┴───────────────┴─────────   │   │
│  │                    Tauri IPC Bridge                            │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    RUST BACKEND                               │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │   │
│  │  │ LCU Connector│  │ Game Monitor │  │ Riot API Client  │   │   │
│  │  │              │  │              │  │                  │   │   │
│  │  │ - Lockfile   │  │ - 1Hz poll   │  │ - Rate limited   │   │   │
│  │  │ - WebSocket  │  │ - State diff │  │ - Priority queue │   │   │
│  │  │ - Event bus  │  │ - Event emit │  │ - Retry logic    │   │   │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────────┘   │   │
│  │         │                 │                  │               │   │
│  │  ┌──────┴─────────────────┴──────────────────┴───────────┐   │   │
│  │  │                  EVENT BUS (tokio channels)            │   │   │
│  │  └──────────────────────────┬─────────────────────────────┘   │   │
│  │                             │                                 │   │
│  │  ┌──────────────────────────┴─────────────────────────────┐   │   │
│  │  │                  ANALYSIS ENGINE                        │   │   │
│  │  │                                                        │   │   │
│  │  │  ┌──────────────┐  ┌───────────────┐  ┌────────────┐  │   │   │
│  │  │  │ Pattern      │  │ Draft         │  │ Game State │  │   │   │
│  │  │  │ Detector     │  │ Analyzer      │  │ Analyzer   │  │   │   │
│  │  │  │              │  │               │  │            │  │   │   │
│  │  │  │ - Cross-game │  │ - Champion    │  │ - Gold diff│  │   │   │
│  │  │  │ - Behavioral │  │   pool match  │  │ - Spike    │  │   │   │
│  │  │  │ - Temporal   │  │ - Composition │  │   tracking │  │   │   │
│  │  │  │ - Statistical│  │ - Historical  │  │ - Objective│  │   │   │
│  │  │  └──────────────┘  └───────────────┘  └────────────┘  │   │   │
│  │  └────────────────────────────┬───────────────────────────┘   │   │
│  │                               │                               │   │
│  │  ┌────────────────────────────┴───────────────────────────┐   │   │
│  │  │                  SQLite (rusqlite)                      │   │   │
│  │  │  matches | timelines | patterns | profiles | goals     │   │   │
│  │  └────────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘

External:
  ┌─────────────────┐    ┌──────────────────┐    ┌──────────────┐
  │ League Client   │    │ Game Process     │    │ Riot API     │
  │ (LCU API)       │    │ (port 2999)      │    │ (remote)     │
  │ localhost:<port> │    │ localhost:2999    │    │ api.riot.com │
  └─────────────────┘    └──────────────────┘    └──────────────┘
```

## Data Flow

### Flow 1: Game Lifecycle (Event-Driven)

```
League Client starts
  → LCU Connector detects lockfile
  → WebSocket connection established
  → Subscribe to game flow events

Game flow: None → Lobby → Matchmaking → ChampSelect → InProgress → EndOfGame

ChampSelect:
  LCU WebSocket event
  → Parse picks/bans/team
  → Fetch opponent data (Riot API, cached profiles)
  → Run Draft Analyzer
  → Push recommendations to frontend via IPC

InProgress:
  Game Client API (1 Hz poll)
  → Diff against previous state
  → Run Game State Analyzer
  → Push updates to frontend via IPC

EndOfGame:
  LCU WebSocket event (end of game stats)
  → Fetch full match + timeline from Riot API
  → Store in SQLite
  → Run Pattern Detector (incremental)
  → Generate Post-Game Analysis
  → Push to frontend via IPC
```

### Flow 2: Pattern Detection (Batch)

```
Trigger: New match data arrives (post-game) OR initial history backfill

1. Fetch match timeline from Riot API (if not cached)
2. Extract features per game:
   - Gold/XP/CS curves (per minute)
   - Death positions and timestamps
   - Kill participation windows
   - Back timing relative to objectives
   - Lane phase metrics (CS diff @ 10, 15 min)
   - Team fight participation rate by game phase
3. Aggregate features across games:
   - Group by champion, role, game phase
   - Compute distributions and outliers
4. Pattern detection:
   - Compare player's distributions to rank-appropriate baselines
   - Identify statistically significant deviations
   - Cross-reference with win/loss outcomes
   - Generate natural language insights
5. Store detected patterns in SQLite
6. Update improvement tracking metrics
```

### Flow 3: Real-Time Game Companion (Polling)

```
Every 1 second:
  GET /liveclientdata/activeplayer    → Update own stats
  GET /liveclientdata/playerlist      → Update all player items/levels
  GET /liveclientdata/eventdata       → Check for new events
  GET /liveclientdata/gamestats       → Game time

Compute:
  - Team gold differential (sum of visible gold indicators)
  - Item completion proximity (track components, estimate completion)
  - Power spike detection (key item completions, level breakpoints)
  - Contextual item recommendation (based on enemy items + gold state)

Push:
  - Emit events via Tauri IPC to Svelte frontend
  - Frontend updates reactively (Svelte runes handle granular updates)
```

## Concurrency Model

```
tokio runtime (multi-threaded)
├── Task 1: LCU WebSocket listener (long-lived)
├── Task 2: Game Client API poller (spawned when game starts, dropped when game ends)
├── Task 3: Riot API request processor (processes priority queue)
├── Task 4: Pattern analysis (spawned post-game, CPU-bound → spawn_blocking)
└── Task 5: Tauri IPC handler (responds to frontend requests)

Communication: tokio::sync::broadcast channels for events
               tokio::sync::mpsc for Riot API request queue
               Arc<Mutex<>> for shared state (game state, champ select state)
```

## Offline Capability

- Core features work without internet after initial data fetch
- SQLite contains all historical data locally
- Pattern engine runs entirely locally
- Only Riot API calls require internet:
  - Opponent scouting in champ select (degrade gracefully: show cached data if available)
  - Post-game match/timeline fetch (queue for later)
  - Rank data refresh (use last known)

---

# Step 8: MVP Plan

## Week 1: Foundation

**Goal**: App launches, detects League client, shows game flow state.

- [ ] Tauri v2 project scaffolding (Rust + Svelte 5 + Vite)
- [ ] LCU Connector: lockfile detection, process fallback, WebSocket connection
- [ ] Game flow state machine: detect None/Lobby/ChampSelect/InProgress/EndOfGame
- [ ] SQLite schema creation and migration system
- [ ] Basic UI shell: navigation, status indicator (connected/disconnected)
- [ ] Riot API client with rate limiting and priority queue

**Deliverable**: App that auto-connects to League client and shows current game state.

## Week 2: Pre-Game Intelligence

**Goal**: Useful information during champion select.

- [ ] Champ select parser: read picks, bans, team composition from LCU WebSocket
- [ ] Opponent scouting: fetch opponent rank, champion mastery, recent performance
- [ ] Match history fetcher: pull last 20 matches for current player
- [ ] Basic draft suggestions: recommend from player's champion pool based on win rate + mastery
- [ ] Pre-game view UI: show team, opponents, suggestions

**Deliverable**: During champ select, shows opponent info and champion recommendations.

## Week 3: Live Game + Post-Game

**Goal**: Companion view during games, basic post-game analysis.

- [ ] Game Client API integration: poll at 1 Hz during active games
- [ ] Live companion view: gold diff, items, events, game timer
- [ ] Contextual item suggestions based on current game state
- [ ] Post-game match + timeline fetch and storage
- [ ] Post-game stats display with key moment identification (largest gold swings)

**Deliverable**: Second-screen companion during games. Post-game screen showing key moments.

## Week 4: Pattern Engine + Polish

**Goal**: First version of cross-game pattern detection. Release-ready.

- [ ] Pattern detection: death timing analysis, CS trends, win condition analysis
- [ ] Improvement dashboard: week-over-week metrics
- [ ] Auto-updater setup (Tauri plugin-updater)
- [ ] Installer configuration (NSIS for Windows)
- [ ] Performance profiling and optimization
- [ ] Error handling, graceful degradation, edge cases

**Deliverable**: Full MVP ready for alpha users.

## What's explicitly OUT of MVP

- Cloud backend / aggregate statistics
- Account system / social features
- Multi-region support (start with one region)
- Mobile companion view
- Advanced pattern detection (positional heatmaps, wave management)
- Champion-specific coaching
- Replay file analysis
- Overlay of any kind
- Ads of any kind

## What creates immediate value

1. **Week 1**: The moment the app connects to the League client and shows game state with < 80 MB RAM, it's already differentiated.
2. **Week 2**: Opponent scouting with personal champion pool recommendations during draft is a daily-use feature.
3. **Week 3**: A clean, lightweight second-screen companion during games is something players will keep open every game.
4. **Week 4**: The first pattern insight ("you die to ganks 40% more when ahead") is the moment the product becomes genuinely different from everything else.

---

# Step 9: Differentiation

## Why users switch

**1. Performance (acquisition hook)**
"Try the companion app that uses 50 MB instead of 1000 MB."
This is the door-opener. It's immediately verifiable (open Task Manager), immediately felt (no FPS impact), and immediately shareable ("bro this app uses nothing").

**2. Personal pattern insights (retention hook)**
After 20-50 games, the Pattern Engine knows your weaknesses better than you do. "You throw leads by overextending without vision" is specific, actionable, and personal. No other tool does this. Once you've seen these insights, a generic stats dashboard feels empty.

**3. Transparent reasoning (trust builder)**
"Build Zhonya's because their Zed is 7/1 and you've died to his ult 3 times this game" vs "Build Zhonya's (53.2% WR)". The first creates trust. The second creates compliance without understanding.

**4. No ads, no Overwolf, no account required**
Download. Open. It works. No signup flow, no credit card, no "watch this ad to unlock features." This is increasingly rare and increasingly valued.

**5. Privacy by design**
All data stays on your machine. In an era of data harvesting, "your gameplay data never leaves your PC" is a meaningful differentiator.

## What makes this hard to copy

**1. Architectural moat**
Blitz, Mobalytics, and Porofessor would need to rewrite their entire stack to match Sentinel's resource footprint. They're locked into Electron/Overwolf. This isn't a feature they can add -- it's an architectural decision they made years ago.

**2. Personal data moat**
The Pattern Engine's value scales with the amount of YOUR data it has. After 200 games of analysis, switching to another tool means abandoning that history. And a competitor would need to build the same analytical engine from scratch.

**3. Business model alignment**
Sentinel doesn't need ads because it doesn't have Electron's resource costs, Overwolf's platform fees, or a large team's burn rate. Competitors who depend on ad revenue can't simply remove ads -- it's their business model.

**4. Focus**
Every competitor is expanding: TFT support, Valorant support, multi-game platforms. Sentinel does one thing -- League of Legends personal improvement -- and does it better than anyone. Focus is a moat when competitors are spreading thin.

---

# Step 10: Risks

## Riot Policy Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|------------|
| LCU API changes break functionality | HIGH | HIGH (happens every few patches) | Community monitoring (CommunityDragon), rapid patch-day response, abstract LCU layer for easy updates |
| Riot restricts Game Client API | MEDIUM | LOW | API is Riot-provided and documented. If restricted, degrade gracefully to LCU-only data |
| Riot bans specific feature categories | MEDIUM | MEDIUM (banned enemy ult timers March 2025) | Stay conservative. Only show data derivable from in-game UI. No automation. No overlay. Track policy announcements. |
| Riot requires third-party app registration | LOW | MEDIUM | Pre-register when required. Maintain free tier (Riot requires this). Keep codebase audit-ready. |
| Vanguard interference | LOW | LOW | Sentinel doesn't read game memory. It uses official APIs. Vanguard targets memory readers, not API consumers. |

**Strategic position**: Sentinel's feature set is deliberately conservative. No overlays, no automated actions, no information that isn't already available in the game client. We're on the safe side of every known Riot policy line.

## Technical Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|------------|
| Tauri WebView2 memory exceeds targets | MEDIUM | MEDIUM | Profile early and often. If WebView2 memory is problematic, consider pure Rust UI for critical views |
| Pattern engine produces noise, not signal | HIGH | MEDIUM | Statistical significance testing. Minimum sample size requirements. User feedback loop. Start with simple, high-confidence patterns. |
| Riot API rate limits constrain user experience | MEDIUM | LOW | Aggressive caching. Priority queue. Match data is immutable -- cache permanently. Background backfill. |
| Solo developer maintenance burden on patch days | HIGH | HIGH | Automate LCU endpoint testing. Community CommunityDragon monitors patches. Keep LCU abstraction layer thin. |
| SQLite performance with large match history | LOW | LOW | Index key columns. Partition old data. SQLite handles millions of rows fine for this query pattern. |

## Adoption Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|------------|
| No brand recognition | HIGH | HIGH | Content marketing (Reddit, YouTube). Show pattern engine insights as shareable content. Performance benchmarks as marketing. |
| Users don't understand the value vs existing tools | MEDIUM | MEDIUM | First-run experience that shows a pattern insight immediately (fetch last 20 matches, detect something). Demo before commitment. |
| Solo developer → slow feature velocity | MEDIUM | HIGH | Focus on depth over breadth. One feature done extremely well beats five features done adequately. |
| Player perception: "another companion app" | MEDIUM | HIGH | Lead with performance story. "50 MB, no ads, no Overwolf" cuts through noise. Let the pattern engine sell itself after trial. |

---

# Step 11: Final Recommendation

## Architecture

**Tauri v2** with Rust backend and Svelte 5 frontend.

This is the only architecture that delivers the performance differentiator (10x less RAM than competitors) while maintaining the development velocity a solo developer needs. Pure Rust GUI is too slow to ship. Electron defeats the value proposition. Tauri is the correct tradeoff.

## Stack

| Layer | Choice | Reason |
|-------|--------|--------|
| Framework | Tauri v2 | Performance + web UI speed |
| Backend | Rust (tokio, reqwest, rusqlite) | Async I/O, zero-cost abstractions, no GC |
| Frontend | Svelte 5 + Tailwind | Compiled, small bundle, fast DX |
| Database | SQLite | Local-first, zero-config, full SQL |
| Cloud | None initially (CDN for patch data) | Ship without dependencies |

## Starting Point

**Day 1**: `cargo create-tauri-app sentinel --template svelte`

**First feature to complete**: LCU Connector.
The moment the app detects the League client, reads the lockfile, connects to the WebSocket, and displays the current game phase -- you have a working foundation. Everything else builds on this connection.

**First feature to ship**: Pre-game opponent scouting with personal champion recommendations.
This is the feature players use every game. It's the entry point. It's what gets them to try the app. The Pattern Engine is what makes them stay.

**First marketing moment**: A screenshot of Task Manager showing Sentinel at 60 MB next to Blitz at 900 MB. This image will do more for adoption than any feature list.

## The bet

The bet is that **personal behavioral intelligence** is more valuable than **generic information access**, and that players will choose a tool that's **10x lighter** and tells them what THEY specifically need to change.

Every existing tool is an information terminal. Sentinel is a personal coach that happens to be the lightest app in the category.

Build the connector. Build the pattern engine. Ship it light. Let the data compound.

---

## Project Codename: SENTINEL

*See everything. Carry nothing.*
