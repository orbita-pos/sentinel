use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{ConnectInfo, Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use tokio::sync::Mutex;

use crate::game_client::state::LiveGameState;

/// Shared state for the mobile server
pub struct MobileServerState {
    pub live_state: Arc<Mutex<Option<LiveGameState>>>,
    pub auth_token: String,
    pub rate_limits: Mutex<HashMap<String, Instant>>,
}

/// Generate a random 8-character alphanumeric token
fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let chars = b"abcdefghjkmnpqrstuvwxyz23456789"; // no ambiguous chars
    (0..8)
        .map(|i| {
            let idx = ((seed >> (i * 5)) as usize) % chars.len();
            chars[idx] as char
        })
        .collect()
}

/// Start the mobile companion HTTP server on port 3333
/// Returns the auth token for the URL
pub async fn start(live_state: Arc<Mutex<Option<LiveGameState>>>) -> String {
    let token = generate_token();
    let state = Arc::new(MobileServerState {
        live_state,
        auth_token: token.clone(),
        rate_limits: Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/", get(serve_page))
        .route("/api/state", get(get_state))
        // No CorsLayer -- same-origin only (fixes H1)
        .with_state(state);

    let addr = "0.0.0.0:3333";
    tracing::info!("Mobile companion server starting on http://{addr}");

    if let Ok(local_ip) = local_ip_address::local_ip() {
        tracing::info!("Open on your phone: http://{}:3333/?token={}", local_ip, token);
    }

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Failed to start mobile server: {e}");
            return token;
        }
    };

    let svc = app.into_make_service_with_connect_info::<SocketAddr>();
    if let Err(e) = axum::serve(listener, svc).await {
        tracing::warn!("Mobile server error: {e}");
    }

    token
}

#[derive(serde::Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

/// Validate auth token from query params
fn validate_token(query: &TokenQuery, expected: &str) -> bool {
    query.token.as_deref() == Some(expected)
}

/// Rate limit check: max 1 request per 500ms per IP
async fn check_rate_limit(state: &MobileServerState, addr: &str) -> bool {
    let mut limits = state.rate_limits.lock().await;
    let now = Instant::now();

    if let Some(last) = limits.get(addr) {
        if now.duration_since(*last).as_millis() < 500 {
            return false; // Too fast
        }
    }

    limits.insert(addr.to_string(), now);

    // Cleanup old entries (> 60 seconds)
    limits.retain(|_, t| now.duration_since(*t).as_secs() < 60);

    true
}

/// API endpoint: returns sanitized live game state as JSON
async fn get_state(
    State(state): State<Arc<MobileServerState>>,
    Query(query): Query<TokenQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Auth check (fixes C1)
    if !validate_token(&query, &state.auth_token) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Rate limit check (fixes H2)
    if !check_rate_limit(&state, &addr.ip().to_string()).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let game = state.live_state.lock().await.clone();
    match game {
        Some(s) => {
            // Sanitize: redact enemy player names (fixes M1)
            let mut val = serde_json::to_value(&s).unwrap_or_default();
            if let Some(enemies) = val.get_mut("enemy_team").and_then(|t| t.as_array_mut()) {
                for enemy in enemies {
                    if let Some(obj) = enemy.as_object_mut() {
                        // Replace name with champion name (remove Riot ID)
                        let champ = obj.get("champion").and_then(|c| c.as_str()).unwrap_or("?").to_string();
                        obj.insert("name".to_string(), serde_json::Value::String(champ));
                    }
                }
            }
            Ok(Json(val))
        }
        None => Ok(Json(serde_json::json!({"status": "no_game"}))),
    }
}

/// Serve the mobile companion web page (fixes M2: no innerHTML)
async fn serve_page(
    State(state): State<Arc<MobileServerState>>,
    Query(query): Query<TokenQuery>,
) -> Result<Html<String>, StatusCode> {
    // Auth check
    if !validate_token(&query, &state.auth_token) {
        return Ok(Html("<html><body style='background:#0a0e14;color:#e2e8f0;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh'><div style='text-align:center'><h2 style='color:#ef4444'>Access Denied</h2><p style='color:#64748b;margin-top:8px'>Invalid or missing token. Open the URL from Sentinel app.</p></div></body></html>".into()));
    }

    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "localhost".to_string());

    let token = &state.auth_token;

    Ok(Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
<title>Sentinel Mobile</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{background:#0a0e14;color:#e2e8f0;font-family:'Segoe UI',system-ui,sans-serif;padding:12px;min-height:100vh}}
.hdr{{display:flex;align-items:center;justify-content:space-between;padding:8px 0;margin-bottom:12px;border-bottom:1px solid #2a3544}}
.hdr h1{{font-size:16px;font-weight:800;color:#3b82f6}}
.timer{{font-family:monospace;font-size:20px;font-weight:700}}
.score{{display:flex;align-items:center;justify-content:center;gap:12px;margin-bottom:12px}}
.big{{font-size:28px;font-weight:900}}
.card{{background:#111821;border:1px solid #2a3544;border-radius:12px;padding:12px;margin-bottom:10px}}
.ct{{font-size:10px;text-transform:uppercase;letter-spacing:.5px;color:#64748b;font-weight:700;margin-bottom:8px}}
.row{{display:flex;align-items:center;gap:8px;margin-bottom:6px}}
.row img{{width:28px;height:28px;border-radius:6px}}
.items{{display:flex;gap:4px;flex-wrap:wrap}}
.items img{{width:28px;height:28px;border-radius:4px;border:1px solid #2a3544}}
.wait{{display:flex;flex-direction:column;align-items:center;justify-content:center;height:60vh}}
.g{{color:#22c55e}}.r{{color:#ef4444}}.b{{color:#3b82f6}}.y{{color:#eab308}}.m{{color:#64748b}}
</style>
</head>
<body>
<div id="app"></div>
<script>
const TOKEN="{token}";
const CDN="https://ddragon.leagueoflegends.com/cdn/15.6.1";
const $=id=>document.getElementById(id);
const el=(t,cls,txt)=>{{const e=document.createElement(t);if(cls)e.className=cls;if(txt)e.textContent=txt;return e}};
const img=(src,s)=>{{const i=document.createElement('img');i.src=src;i.style.cssText=s||'';i.onerror=()=>i.style.display='none';return i}};
function cI(n){{return CDN+'/img/champion/'+n+'.png'}}
function iI(id){{return CDN+'/img/item/'+id+'.png'}}
function fG(g){{return g>=1000?(g/1000).toFixed(1)+'k':g}}

function render(d){{
  const app=$('app');
  app.innerHTML='';

  if(!d.game_time){{
    const w=el('div','wait');
    w.appendChild(el('h2','b','Sentinel'));
    w.appendChild(el('p','m','Waiting for game...'));
    const c=el('p','m');c.style.fontSize='11px';c.style.marginTop='16px';c.textContent='Connected to {local_ip}:3333';
    w.appendChild(c);
    app.appendChild(w);
    return;
  }}

  // Header
  const hdr=el('div','hdr');
  hdr.appendChild(el('h1','','SENTINEL'));
  const m=Math.floor(d.game_time/60),s=Math.floor(d.game_time%60);
  hdr.appendChild(el('div','timer',m+':'+String(s).padStart(2,'0')));
  app.appendChild(hdr);

  // Score
  const sc=el('div','score');
  const bk=d.my_team.reduce((s,p)=>s+p.kills,0);
  const rk=d.enemy_team.reduce((s,p)=>s+p.kills,0);
  sc.appendChild(el('span','big b',bk));
  sc.appendChild(el('span','m','vs'));
  sc.appendChild(el('span','big r',rk));
  if(d.team_gold_diff!==0){{
    const df=el('span','',`${{d.team_gold_diff>=0?'+':''}}${{fG(d.team_gold_diff)}}`);
    df.style.cssText=`padding:2px 8px;border-radius:6px;font-size:12px;font-weight:700;color:${{d.team_gold_diff>=0?'#3b82f6':'#ef4444'}};background:${{d.team_gold_diff>=0?'rgba(59,130,246,.15)':'rgba(239,68,68,.15)'}}`;
    sc.appendChild(df);
  }}
  app.appendChild(sc);

  // Me card
  const me=d.my_team.find(p=>p.is_local_player);
  if(me){{
    const c=el('div','card');
    const r=el('div','row');
    r.appendChild(img(cI(me.champion),'width:36px;height:36px;border-radius:8px'));
    const info=el('div','');info.style.flex='1';
    info.appendChild(el('div','',me.champion+' Lv'+me.level));
    const kda=el('div','');kda.style.cssText='font-size:16px;font-weight:800';
    const k=el('span','g',me.kills);const sl=el('span','m',' / ');const dt=el('span','r',me.deaths);const sl2=el('span','m',' / ');const a=el('span','',me.assists);
    kda.append(k,sl,dt,sl2,a);
    info.appendChild(kda);
    r.appendChild(info);
    const gold=el('div','y',Math.floor(d.active_player?.current_gold||0)+'g');gold.style.cssText='font-size:16px;font-weight:700';
    r.appendChild(gold);
    c.appendChild(r);
    const items=el('div','items');
    me.items.filter(i=>i.item_id>0).forEach(i=>items.appendChild(img(iI(i.item_id))));
    c.appendChild(items);
    app.appendChild(c);
  }}

  // Threats
  if(d.enemy_team.length>0){{
    const c=el('div','card');
    c.appendChild(el('div','ct','Threats'));
    const sorted=[...d.enemy_team].sort((a,b)=>b.items.reduce((s,i)=>s+i.price,0)-a.items.reduce((s,i)=>s+i.price,0));
    sorted.slice(0,5).forEach(e=>{{
      const gold=e.items.reduce((s,i)=>s+i.price,0);
      const fed=e.kills>=4||gold>8000;
      const r=el('div','row');
      r.style.cssText=`padding:6px 8px;border-radius:8px;background:${{fed?'rgba(239,68,68,.08)':'rgba(26,35,50,.5)'}}`;
      r.appendChild(img(cI(e.champion),'width:24px;height:24px;border-radius:6px'));
      const n=el('span','',e.champion);n.style.cssText='font-size:11px;font-weight:600;flex:1';r.appendChild(n);
      r.appendChild(el('span','',`${{e.kills}}/${{e.deaths}}/${{e.assists}}`));
      const g=el('span','y',fG(gold));g.style.fontSize='10px';r.appendChild(g);
      if(fed){{const f=el('span','r','FED');f.style.cssText='font-size:8px;font-weight:700';r.appendChild(f)}}
      c.appendChild(r);
    }});
    app.appendChild(c);
  }}

  // All players
  const all=[...d.my_team,...d.enemy_team];
  if(all.length>0){{
    const c=el('div','card');
    c.appendChild(el('div','ct','All Players'));
    all.forEach(p=>{{
      const isMe=p.is_local_player;
      const isEnemy=d.enemy_team.some(e=>e.champion===p.champion&&e.kills===p.kills);
      const r=el('div','row');
      r.style.padding='4px 0';
      if(isMe)r.style.cssText+='background:rgba(59,130,246,.05);border-radius:6px;padding:4px 6px';
      r.appendChild(img(cI(p.champion),'width:24px;height:24px;border-radius:4px'));
      const n=el('span','',p.champion+(isMe?' (YOU)':''));
      n.style.cssText=`font-size:11px;font-weight:500;flex:1;color:${{isEnemy?'#ef4444':'#3b82f6'}}`;
      r.appendChild(n);
      r.appendChild(el('span','',`${{p.kills}}/${{p.deaths}}/${{p.assists}}`));
      const cs=el('span','m',p.cs+'cs');cs.style.fontSize='10px';r.appendChild(cs);
      c.appendChild(r);
    }});
    app.appendChild(c);
  }}
}}

async function poll(){{
  try{{
    const r=await fetch('/api/state?token='+TOKEN);
    if(r.status===403){{render({{}});return}}
    render(await r.json());
  }}catch(e){{}}
}}
setInterval(poll,2000);
poll();
</script>
</body>
</html>"#)))
}
