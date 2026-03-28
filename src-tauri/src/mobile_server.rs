use std::sync::Arc;

use axum::{extract::State, response::Html, routing::get, Json, Router};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::game_client::state::LiveGameState;

/// Shared state for the mobile server
pub struct MobileServerState {
    pub live_state: Arc<Mutex<Option<LiveGameState>>>,
}

/// Start the mobile companion HTTP server on port 3333
pub async fn start(live_state: Arc<Mutex<Option<LiveGameState>>>) {
    let state = Arc::new(MobileServerState { live_state });

    let app = Router::new()
        .route("/", get(serve_page))
        .route("/api/state", get(get_state))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3333";
    tracing::info!("Mobile companion server starting on http://{addr}");

    if let Ok(local_ip) = local_ip_address::local_ip() {
        tracing::info!("Open on your phone: http://{}:3333", local_ip);
    }

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Failed to start mobile server: {e}");
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        tracing::warn!("Mobile server error: {e}");
    }
}

/// API endpoint: returns current live game state as JSON
async fn get_state(
    State(state): State<Arc<MobileServerState>>,
) -> Json<serde_json::Value> {
    let game = state.live_state.lock().await.clone();
    match game {
        Some(s) => Json(serde_json::to_value(&s).unwrap_or_default()),
        None => Json(serde_json::json!({"status": "no_game"})),
    }
}

/// Serve the mobile companion web page
async fn serve_page() -> Html<String> {
    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "localhost".to_string());

    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
<title>Sentinel Mobile</title>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:#0a0e14; color:#e2e8f0; font-family:'Segoe UI',system-ui,sans-serif; padding:12px; min-height:100vh; }}
  .header {{ display:flex; align-items:center; justify-content:space-between; padding:8px 0; margin-bottom:12px; border-bottom:1px solid #2a3544; }}
  .header h1 {{ font-size:16px; font-weight:800; color:#3b82f6; }}
  .header .timer {{ font-family:monospace; font-size:20px; font-weight:700; }}
  .score {{ display:flex; align-items:center; justify-content:center; gap:12px; margin-bottom:12px; }}
  .score .kills {{ font-size:28px; font-weight:900; }}
  .score .blue {{ color:#3b82f6; }}
  .score .red {{ color:#ef4444; }}
  .score .diff {{ padding:2px 8px; border-radius:6px; font-size:12px; font-weight:700; }}
  .card {{ background:#111821; border:1px solid #2a3544; border-radius:12px; padding:12px; margin-bottom:10px; }}
  .card-title {{ font-size:10px; text-transform:uppercase; letter-spacing:0.5px; color:#64748b; font-weight:700; margin-bottom:8px; }}
  .player {{ display:flex; align-items:center; gap:8px; margin-bottom:6px; }}
  .player img {{ width:32px; height:32px; border-radius:8px; }}
  .player .name {{ font-size:12px; font-weight:600; }}
  .player .kda {{ font-size:14px; font-weight:800; }}
  .player .gold {{ color:#eab308; font-size:14px; font-weight:700; }}
  .items {{ display:flex; gap:4px; flex-wrap:wrap; }}
  .items img {{ width:28px; height:28px; border-radius:4px; border:1px solid #2a3544; }}
  .threat {{ display:flex; align-items:center; gap:6px; padding:6px 8px; border-radius:8px; margin-bottom:4px; }}
  .threat img {{ width:24px; height:24px; border-radius:6px; }}
  .threat .fed {{ background:rgba(239,68,68,0.1); }}
  .bar {{ height:6px; border-radius:3px; background:#1a2332; overflow:hidden; display:flex; }}
  .bar .ad {{ background:#ef4444; }}
  .bar .ap {{ background:#3b82f6; }}
  .dmg-labels {{ display:flex; justify-content:space-between; font-size:10px; font-weight:700; margin-top:4px; }}
  .waiting {{ display:flex; flex-direction:column; align-items:center; justify-content:center; height:60vh; }}
  .waiting p {{ color:#64748b; margin-top:8px; }}
  .rec {{ display:flex; align-items:center; gap:8px; margin-bottom:6px; }}
  .rec img {{ width:28px; height:28px; border-radius:6px; border:1px solid #2a3544; }}
  .rec .tag {{ font-size:9px; font-weight:700; padding:1px 5px; border-radius:3px; }}
  .green {{ color:#22c55e; }} .red {{ color:#ef4444; }} .blue {{ color:#3b82f6; }}
  #no-game {{ display:none; }}
</style>
</head>
<body>

<div id="game">
  <div class="header">
    <h1>SENTINEL</h1>
    <div class="timer" id="timer">0:00</div>
  </div>

  <div class="score">
    <span class="kills blue" id="blue-kills">0</span>
    <span style="color:#64748b; font-size:12px">vs</span>
    <span class="kills red" id="red-kills">0</span>
    <span class="diff" id="gold-diff" style="display:none"></span>
  </div>

  <div class="card" id="me-card" style="display:none">
    <div class="player">
      <img id="me-img" src="" alt="">
      <div style="flex:1">
        <div class="name" id="me-name"></div>
        <div class="kda" id="me-kda"></div>
      </div>
      <div style="text-align:right">
        <div class="gold" id="me-gold"></div>
        <div style="font-size:10px; color:#64748b" id="me-cs"></div>
      </div>
    </div>
    <div class="items" id="me-items"></div>
  </div>

  <div class="card" id="threats-card" style="display:none">
    <div class="card-title">Threats</div>
    <div id="threats"></div>
  </div>

  <div class="card" id="dmg-card" style="display:none">
    <div class="card-title">Enemy Damage</div>
    <div class="bar"><div class="ad" id="ad-bar"></div><div class="ap" id="ap-bar"></div></div>
    <div class="dmg-labels"><span class="red" id="ad-pct">AD 0%</span><span class="blue" id="ap-pct">AP 0%</span></div>
  </div>

  <div class="card" id="team-card" style="display:none">
    <div class="card-title">All Players</div>
    <div id="all-players"></div>
  </div>
</div>

<div id="no-game" class="waiting">
  <h2 style="color:#3b82f6; font-size:20px">Sentinel</h2>
  <p>Waiting for game...</p>
  <p style="font-size:11px; margin-top:16px">Connected to {local_ip}:3333</p>
</div>

<script>
const PATCH = "15.6.1";
const CDN = `https://ddragon.leagueoflegends.com/cdn/${{PATCH}}`;

function champImg(n) {{ return `${{CDN}}/img/champion/${{n}}.png`; }}
function itemImg(id) {{ return `${{CDN}}/img/item/${{id}}.png`; }}
function fmtGold(g) {{ return g >= 1000 ? (g/1000).toFixed(1)+'k' : g; }}

async function poll() {{
  try {{
    const r = await fetch('/api/state');
    const data = await r.json();

    if (data.status === 'no_game' || !data.game_time) {{
      document.getElementById('game').style.display = 'none';
      document.getElementById('no-game').style.display = 'flex';
      return;
    }}

    document.getElementById('game').style.display = 'block';
    document.getElementById('no-game').style.display = 'none';

    // Timer
    const m = Math.floor(data.game_time / 60);
    const s = Math.floor(data.game_time % 60);
    document.getElementById('timer').textContent = `${{m}}:${{String(s).padStart(2,'0')}}`;

    // Score
    const bk = data.my_team.reduce((s,p) => s + p.kills, 0);
    const rk = data.enemy_team.reduce((s,p) => s + p.kills, 0);
    document.getElementById('blue-kills').textContent = bk;
    document.getElementById('red-kills').textContent = rk;

    const diff = data.team_gold_diff;
    const diffEl = document.getElementById('gold-diff');
    if (diff !== 0) {{
      diffEl.style.display = 'inline';
      diffEl.textContent = (diff >= 0 ? '+' : '') + fmtGold(diff);
      diffEl.style.background = diff >= 0 ? 'rgba(59,130,246,0.15)' : 'rgba(239,68,68,0.15)';
      diffEl.style.color = diff >= 0 ? '#3b82f6' : '#ef4444';
    }} else {{ diffEl.style.display = 'none'; }}

    // My player
    const me = data.my_team.find(p => p.is_local_player);
    if (me) {{
      const card = document.getElementById('me-card');
      card.style.display = 'block';
      document.getElementById('me-img').src = champImg(me.champion);
      document.getElementById('me-name').textContent = me.champion + ' Lv' + me.level;
      document.getElementById('me-kda').innerHTML = `<span class="green">${{me.kills}}</span> / <span class="red">${{me.deaths}}</span> / ${{me.assists}}`;
      document.getElementById('me-gold').textContent = Math.floor(data.active_player?.current_gold || 0) + 'g';
      document.getElementById('me-cs').textContent = me.cs + ' CS';

      const itemsEl = document.getElementById('me-items');
      itemsEl.innerHTML = me.items.filter(i => i.item_id > 0).map(i =>
        `<img src="${{itemImg(i.item_id)}}" title="${{i.name}}" onerror="this.style.display='none'">`
      ).join('');
    }}

    // Threats (enemies sorted by gold)
    const enemies = [...data.enemy_team].sort((a,b) =>
      b.items.reduce((s,i) => s+i.price,0) - a.items.reduce((s,i) => s+i.price,0)
    );
    const threatsEl = document.getElementById('threats');
    const threatsCard = document.getElementById('threats-card');
    if (enemies.length > 0) {{
      threatsCard.style.display = 'block';
      threatsEl.innerHTML = enemies.slice(0, 5).map(e => {{
        const gold = e.items.reduce((s,i) => s + i.price, 0);
        const fed = e.kills >= 4 || gold > 8000;
        return `<div class="threat" style="background:${{fed ? 'rgba(239,68,68,0.08)' : 'rgba(26,35,50,0.5)'}}">
          <img src="${{champImg(e.champion)}}" onerror="this.style.display='none'">
          <span style="font-size:11px; font-weight:600; flex:1">${{e.champion}}</span>
          <span style="font-size:11px">${{e.kills}}/${{e.deaths}}/${{e.assists}}</span>
          <span style="font-size:10px; color:#eab308">${{fmtGold(gold)}}</span>
          ${{fed ? '<span style="font-size:8px; font-weight:700; color:#ef4444">FED</span>' : ''}}
        </div>`;
      }}).join('');
    }}

    // Enemy damage (approximate)
    let ap = 0, ad = 0;
    enemies.forEach(e => {{ ad++; }}); // Simplified, real would use tags
    const total = ap + ad || 1;
    const adPct = Math.round(ad / total * 50); // rough estimate
    const apPct = 100 - adPct;
    document.getElementById('dmg-card').style.display = 'block';
    document.getElementById('ad-bar').style.width = adPct + '%';
    document.getElementById('ap-bar').style.width = apPct + '%';
    document.getElementById('ad-pct').textContent = 'AD ' + adPct + '%';
    document.getElementById('ap-pct').textContent = 'AP ' + apPct + '%';

    // All players
    const allPlayers = [...data.my_team, ...data.enemy_team];
    const playersEl = document.getElementById('all-players');
    document.getElementById('team-card').style.display = 'block';
    playersEl.innerHTML = allPlayers.map(p => {{
      const isMe = p.is_local_player;
      const isEnemy = data.enemy_team.includes(p);
      return `<div class="player" style="padding:4px 0; ${{isMe ? 'background:rgba(59,130,246,0.05); border-radius:6px; padding:4px 6px' : ''}}">
        <img src="${{champImg(p.champion)}}" onerror="this.style.display='none'" style="width:24px;height:24px;border-radius:4px">
        <span style="font-size:11px; font-weight:500; flex:1; color:${{isEnemy ? '#ef4444' : '#3b82f6'}}">${{p.champion}} ${{isMe ? '(YOU)' : ''}}</span>
        <span style="font-size:11px">${{p.kills}}/${{p.deaths}}/${{p.assists}}</span>
        <span style="font-size:10px; color:#64748b">${{p.cs}}cs</span>
      </div>`;
    }}).join('');

  }} catch(e) {{ /* retry next poll */ }}
}}

setInterval(poll, 2000);
poll();
</script>
</body>
</html>"#))
}
