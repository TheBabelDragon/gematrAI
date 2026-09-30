/**
 * gematrAI — calculator + Sefaria corpus tower (static shards only at runtime)
 */
import { SYSTEMS, calculateAll, getSystem } from './gematria/systems.js';
import { CORPUS } from './data/corpus.js';
import { isHebrewLetter } from './gematria/hebrew.js';
import { isGreekLetter } from './gematria/greek.js';
import { isLatinLetter } from './gematria/english.js';

const $ = (s) => document.querySelector(s);
const queryInput = $('#query-input');
const resultsEl = $('#results');
const charCountEl = $('#char-count');
const scriptHintEl = $('#script-hint');
const equivPanel = $('#equivalence-panel');
const equivList = $('#equiv-list');
const aiContent = $('#ai-content');
const reverseValue = $('#reverse-value');
const reverseSystem = $('#reverse-system');
const reverseBtn = $('#reverse-btn');
const matchList = $('#match-list');
const corpusMatchesPanel = $('#corpus-matches-panel');
const corpusMatchesSummary = $('#corpus-matches-summary');
const corpusMatchesList = $('#corpus-matches-list');

let currentText = '', currentResults = [], primarySystemId = null, towerManifest = null;
const shardCache = new Map();
const HEBREW_SYS = ['hebrew-standard','hebrew-gadol','hebrew-katan'];
const UI_SYS_ORDER = ['hebrew-standard','hebrew-gadol','hebrew-katan','greek-isopsephy','english-ordinal','english-reduction','english-pythagorean'];

function escapeHtml(s){return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');}
function scriptClass(ch){if(isHebrewLetter(ch))return'hebrew';if(isGreekLetter(ch))return'greek';if(isLatinLetter(ch))return'latin';return'';}
function detectScripts(text){let h=0,g=0,l=0;for(const ch of text){if(isHebrewLetter(ch))h++;else if(isGreekLetter(ch))g++;else if(isLatinLetter(ch))l++;}const p=[];if(h)p.push('Hebrewx'+h);if(g)p.push('Greekx'+g);if(l)p.push('Latinx'+l);return p.length?p.join(' · '):'-';}
function copyText(text){navigator.clipboard.writeText(text).catch(()=>{const ta=document.createElement('textarea');ta.value=text;document.body.appendChild(ta);ta.select();document.execCommand('copy');document.body.removeChild(ta);});}
function encodeState(){const params=new URLSearchParams();if(currentText)params.set('q',currentText);if(primarySystemId)params.set('sys',primarySystemId);const hash=params.toString();history.replaceState(null,'',hash?'#'+hash:location.pathname);}
function restoreState(){const hash=location.hash.slice(1);if(!hash)return;const params=new URLSearchParams(hash);const q=params.get('q'),sys=params.get('sys');if(q){queryInput.value=q;if(sys&&getSystem(sys))primarySystemId=sys;runCalculation(q);}}

async function loadTowerManifest(){
  try{
    const res=await fetch('./corpus/generated/manifest.json');
    if(!res.ok)throw new Error('no manifest');
    towerManifest=await res.json();
    const st=towerManifest.stats||{};
    const set=(id,v)=>{const el=$(id);if(el)el.textContent=v;};
    set('#tower-name',towerManifest.name||'Sefaria Hebrew Corpus');
    set('#tower-ver',towerManifest.tower||'0.2');
    set('#tower-build',towerManifest.buildId||'-');
    set('#tower-segments',st.segments!=null?String(st.segments):'-');
    set('#tower-tokens',st.unique_terms!=null?String(st.unique_terms):'-');
    set('#tower-books',st.n_books!=null?String(st.n_books):'-');
    set('#tower-scope',(towerManifest.systems||[]).join(', ')||'hebrew');
  }catch(e){
    const set=(id,v)=>{const el=$(id);if(el)el.textContent=v;};
    set('#tower-name','not loaded');set('#tower-build','run corpus build');towerManifest=null;
  }
}

async function fetchShard(systemId,value){
  if(!towerManifest)return[];
  const size=towerManifest.bucketSize||100;
  const b=String(Math.floor(value/size)*size).padStart(4,'0')+'-'+String(Math.floor(value/size)*size+size-1).padStart(4,'0');
  const key=systemId+'|'+b;
  if(shardCache.has(key)){const payload=shardCache.get(key);return (payload&&(payload[String(value)]||payload[value]))||[];}
  if(b==='0000-0099'&&towerManifest.lowValuesUrl){
    const packKey='__low__';
    if(!shardCache.has(packKey)){
      try{const pr=await fetch(towerManifest.lowValuesUrl);if(pr.ok)shardCache.set(packKey,await pr.json());else shardCache.set(packKey,null);}
      catch(e){shardCache.set(packKey,null);}
    }
    const pack=shardCache.get(packKey);
    if(pack&&pack[systemId]){shardCache.set(key,pack[systemId]);return (pack[systemId][String(value)]||pack[systemId][value])||[];}
  }
  const base=towerManifest.valuesBase||'./corpus/generated/values/';
  const url=base+systemId+'/'+b+'.json';
  try{
    const res=await fetch(url);
    if(!res.ok){shardCache.set(key,{});return[];}
    const payload=await res.json();
    shardCache.set(key,payload);
    return (payload[String(value)]||payload[value])||[];
  }catch(e){shardCache.set(key,{});return[];}
}

function getSeedMatches(systemId,value){
  const sys=getSystem(systemId);if(!sys)return[];
  return CORPUS.filter(e=>sys.calculate(e.term).total===value);
}

function runCalculation(text){
  currentText=text;
  charCountEl.textContent=[...text].length+' chars';
  scriptHintEl.textContent=detectScripts(text);
  if(!text.trim()){
    resultsEl.innerHTML='<div class="empty-state">Enter text to begin calculation</div>';
    equivPanel.style.display='none';
    if(corpusMatchesPanel)corpusMatchesPanel.style.display='none';
    aiContent.innerHTML='<p class="empty-state" style="padding:0">Awaiting input...</p>';
    encodeState();return;
  }
  currentResults=calculateAll(text);
  if(!primarySystemId||!currentResults.find(r=>r.system.id===primarySystemId))primarySystemId=currentResults[0]?.system.id||null;
  renderResults();renderEquivalence();renderCorpusMatches();renderAI();encodeState();
}

function renderResults(){
  if(!currentResults.length){resultsEl.innerHTML='<div class="empty-state">No applicable systems</div>';return;}
  resultsEl.innerHTML=currentResults.map(({system,result})=>{
    const expanded=system.id===primarySystemId?'expanded':'';
    const breakdownHtml=result.breakdown.map(b=>{
      const cls=scriptClass(b.char);const valStr=b.value===null?'-':b.value;
      return '<span class="bd-item"><span class="bd-char '+cls+'">'+escapeHtml(b.char)+'</span><span class="bd-val '+(b.value===null?'null':'')+'">'+valStr+'</span></span>';
    }).join('');
    return '<div class="result-card '+expanded+'" data-sys="'+system.id+'"><div class="result-header" data-toggle="'+system.id+'"><div class="result-sys">'+escapeHtml(system.name)+'</div><div class="result-total">'+result.total+'</div><div class="result-actions"><button class="btn-icon" data-copy="'+system.id+'" title="Copy">C</button><button class="btn-icon" data-expand="'+system.id+'">'+(expanded?'v':'>')+'</button></div></div><div class="result-body"><div class="breakdown">'+breakdownHtml+'</div><p style="margin-top:0.6rem;font-family:var(--mono);font-size:0.65rem;color:var(--text-dim)">'+escapeHtml(system.description)+'</p></div></div>';
  }).join('');
  resultsEl.querySelectorAll('[data-toggle]').forEach(el=>{
    el.addEventListener('click',e=>{
      if(e.target.closest('[data-copy]'))return;
      primarySystemId=el.dataset.toggle;
      resultsEl.querySelectorAll('.result-card').forEach(c=>{
        const isThis=c.dataset.sys===primarySystemId;c.classList.toggle('expanded',isThis);
        const btn=c.querySelector('[data-expand]');if(btn)btn.textContent=isThis?'v':'>';
      });
      renderEquivalence();renderCorpusMatches();renderAI();encodeState();
    });
  });
  resultsEl.querySelectorAll('[data-copy]').forEach(btn=>{
    btn.addEventListener('click',e=>{e.stopPropagation();const r=currentResults.find(x=>x.system.id===btn.dataset.copy);if(r){copyText(String(r.result.total));btn.textContent='OK';setTimeout(()=>btn.textContent='C',900);}});
  });
}

function renderEquivalence(){
  if(!currentText.trim()||!primarySystemId){equivPanel.style.display='none';return;}
  const primary=currentResults.find(r=>r.system.id===primarySystemId);if(!primary){equivPanel.style.display='none';return;}
  const target=primary.result.total;
  const sys=getSystem(primarySystemId);
  const matches=CORPUS.map(e=>({...e,total:sys.calculate(e.term).total})).filter(e=>e.total===target&&e.term!==currentText.trim());
  equivPanel.style.display='block';
  if(!matches.length){equivList.innerHTML='<span style="font-family:var(--mono);font-size:0.75rem;color:var(--text-dim)">No other seed-corpus matches.</span>';return;}
  equivList.innerHTML=matches.slice(0,24).map(m=>'<button class="equiv-chip" data-term="'+escapeHtml(m.term)+'">'+escapeHtml(m.term)+'</button>').join('');
  equivList.querySelectorAll('[data-term]').forEach(chip=>chip.addEventListener('click',()=>{queryInput.value=chip.dataset.term;runCalculation(chip.dataset.term);}));
}

async function renderCorpusMatches(){
  if(!corpusMatchesPanel||!currentText.trim()||!primarySystemId){if(corpusMatchesPanel)corpusMatchesPanel.style.display='none';return;}
  const primary=currentResults.find(r=>r.system.id===primarySystemId);if(!primary){corpusMatchesPanel.style.display='none';return;}
  const val=primary.result.total,sid=primary.system.id;
  if(!HEBREW_SYS.includes(sid)||!towerManifest){corpusMatchesPanel.style.display='none';return;}
  const hits=await fetchShard(sid,val);
  corpusMatchesPanel.style.display='block';
  const occ=hits.reduce((s,h)=>s+(h.c||0),0);
  const works=new Set();hits.forEach(h=>(h.b||[]).forEach(b=>works.add(b)));
  corpusMatchesSummary.textContent=val+' · '+hits.length+' unique · '+occ+' occ · '+works.size+' works · '+sid;
  if(!hits.length){corpusMatchesList.innerHTML='<div class="empty-state">No tower matches for this value</div>';return;}
  corpusMatchesList.innerHTML=hits.slice(0,30).map(h=>
    '<div class="match-item" data-term="'+escapeHtml(h.s)+'"><div class="match-term" dir="rtl">'+escapeHtml(h.s)+'</div><div class="match-meta">'+h.c+' occ · '+escapeHtml((h.b||[]).slice(0,4).join(', '))+(h.lic?' · '+escapeHtml(h.lic):'')+'</div>'+((h.r&&h.r[0])?'<div class="match-refs">'+escapeHtml(h.r.slice(0,3).join(', '))+'</div>':'')+'</div>'
  ).join('');
  corpusMatchesList.querySelectorAll('[data-term]').forEach(el=>el.addEventListener('click',()=>{queryInput.value=el.dataset.term;runCalculation(el.dataset.term);}));
}

function renderAI(){
  if(!currentText.trim()||!currentResults.length){aiContent.innerHTML='<p class="empty-state" style="padding:0">Awaiting input...</p>';return;}
  const primary=currentResults.find(r=>r.system.id===primarySystemId)||currentResults[0];
  const total=primary.result.total;
  const obs=[];
  obs.push('Under <strong>'+escapeHtml(primary.system.name)+'</strong> the total is <strong>'+total+'</strong>.');
  obs.push(towerManifest?'Corpus tower is online (Sefaria Tanakh Hebrew indexes).':'Corpus tower not loaded; seed corpus only.');
  obs.push('Numerical equivalence is a property of the chosen mapping — not semantic, theological, or historical relationship.');
  aiContent.innerHTML=obs.map(o=>'<p>'+o+'</p>').join('')+'<p class="disclaimer">Deterministic client-side · not an oracle</p>';
}

function populateReverseSystems(){
  reverseSystem.innerHTML=SYSTEMS.map(s=>'<option value="'+s.id+'">'+escapeHtml(s.short)+' - '+escapeHtml(s.name)+'</option>').join('');
}

async function runReverseLookup(){
  const val=parseInt(reverseValue.value,10);
  if(isNaN(val)||val<0){matchList.innerHTML='<div class="empty-state">Enter a non-negative integer</div>';return;}
  let html='';
  for(const sid of UI_SYS_ORDER){
    const sys=getSystem(sid);if(!sys)continue;
    const label=sys.short+' · '+sys.name;
    let towerHits=[];
    if(towerManifest&&(towerManifest.systems||[]).includes(sid))towerHits=await fetchShard(sid,val);
    const seed=getSeedMatches(sid,val);
    const occ=towerHits.reduce((s,h)=>s+(h.c||0),0);
    const works=new Set();towerHits.forEach(h=>(h.b||[]).forEach(b=>works.add(b)));
    html+='<div class="sys-block"><div class="sys-block-title">'+escapeHtml(label)+'</div>';
    if(towerHits.length){
      html+='<div class="sys-block-meta">'+towerHits.length+' unique · '+occ+' occ · '+works.size+' works · tower</div>';
      html+=towerHits.slice(0,20).map(h=>'<div class="match-item" data-term="'+escapeHtml(h.s)+'" data-sys="'+sid+'"><div class="match-term" dir="rtl">'+escapeHtml(h.s)+'</div><div class="match-meta">'+h.c+' occ · '+escapeHtml((h.b||[]).slice(0,3).join(', '))+'</div></div>').join('');
    }else html+='<div class="sys-block-meta">0 tower matches</div>';
    if(seed.length){
      html+='<div class="sys-block-meta">Seed · '+seed.length+'</div>';
      html+=seed.map(m=>'<div class="match-item" data-term="'+escapeHtml(m.term)+'" data-sys="'+sid+'"><div class="match-term">'+escapeHtml(m.term)+'</div><div class="match-meta">seed</div></div>').join('');
    }
    html+='</div>';
  }
  matchList.innerHTML=html;
  matchList.querySelectorAll('[data-term]').forEach(el=>el.addEventListener('click',()=>{queryInput.value=el.dataset.term;if(el.dataset.sys)primarySystemId=el.dataset.sys;runCalculation(el.dataset.term);window.scrollTo({top:0,behavior:'smooth'});}));
}

function init(){
  populateReverseSystems();loadTowerManifest();
  let debounce;
  queryInput.addEventListener('input',()=>{clearTimeout(debounce);debounce=setTimeout(()=>runCalculation(queryInput.value),60);});
  reverseBtn.addEventListener('click',runReverseLookup);
  reverseValue.addEventListener('keydown',e=>{if(e.key==='Enter')runReverseLookup();});
  restoreState();
  if(!queryInput.value)resultsEl.innerHTML='<div class="empty-state">Enter Latin, Hebrew, or Greek text</div>';
}
init();
