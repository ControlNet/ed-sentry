import React, { useState, useEffect } from 'react';
import { 
  Activity, Shield, Crosshair, AlertTriangle, Settings, 
  List, Terminal, Server, Wifi, Play, CheckCircle2, 
  XCircle, EyeOff, Trash2, Save, Database, Ship, Clock
} from 'lucide-react';

// --- Mock Data (simulated backend state) ---
const MOCK_STATE = {
  connection: { status: 'connected', latency: 42, lastUpdate: new Date().toISOString() },
  session: { active: true, startTime: new Date(Date.now() - 3600000).toISOString(), commander: 'Jameson', ship: 'Anaconda', system: 'LHS 3447', mode: 'Open' },
  status: { hull: 100, shieldsUp: true, fighterAlive: true, fighterHull: 85 },
  combat: { kills: 42, scans: 15, bounty: 12500000, merits: 120, killRateRecent: 12.5, scanRateRecent: 4.2 },
  services: {
    journal: { status: 'ok', folder: 'Default', file: 'Journal.2305101200.01.log' },
    matrix: { status: 'warning', enabled: true, lastDelivery: 'Failed: Timeout' },
    web: { status: 'ok', enabled: true, bind: '127.0.0.1:8080' }
  },
  missions: Array.from({ length: 15 }).map((_, i) => {
    const kinds = ['massacre', 'trade', 'assassination', 'delivery'];
    const factions = ['LHS 3447 Party', 'Federal Unite', 'Silver Boys', 'Eravate Network', 'Empire League'];
    const kind = kinds[i % kinds.length];
    
    let progress = {};
    if (kind === 'massacre') progress = { current: Math.floor(Math.random() * 30), required: 30 };
    else if (kind === 'trade') progress = { commodity: 'Advanced Medicines', collected: 50, delivered: i * 10, required: 120, cargoDepot: true };
    else if (kind === 'assassination') progress = { current: 0, required: 1 };
    
    return {
      id: `m${i + 1}`,
      name: kind === 'massacre' ? 'Massacre Pirate Faction' : kind === 'trade' ? 'Deliver Advanced Medicines' : kind === 'delivery' ? 'Boom Data Delivery' : 'Assassinate Pirate Lord',
      kind: kind,
      state: i === 2 ? 'completed' : 'active',
      issuingFaction: factions[i % factions.length],
      targetFaction: factions[(i + 1) % factions.length],
      destination: i % 2 === 0 ? 'LHS 3447' : 'Sol',
      acceptedTime: new Date(Date.now() - (Math.random() * 10000000)).toISOString(),
      expiryTime: new Date(Date.now() + (Math.random() * 86400000)).toISOString(),
      reward: 1000000 + (i * 250000),
      progress: progress
    };
  }),
  events: [
    { id: 'e1', timestamp: new Date().toISOString(), source: 'Journal', type: 'Bounty', severity: 'success', summary: 'Target destroyed', text: 'Bounty awarded: 250,000 CR (Silver Boys)' },
    { id: 'e2', timestamp: new Date(Date.now() - 5000).toISOString(), source: 'Monitor', type: 'Mission', severity: 'info', summary: 'Mission Updated', text: 'Massacre progress: 15/30' },
    { id: 'e3', timestamp: new Date(Date.now() - 15000).toISOString(), source: 'Journal', type: 'FSDJump', severity: 'info', summary: 'Hyperspace Jump', text: 'System: LHS 3447. Security: High.' },
    { id: 'e4', timestamp: new Date(Date.now() - 45000).toISOString(), source: 'Journal', type: 'FSDTarget', severity: 'warning', summary: 'FSD Charging', text: 'Target: LHS 3447, Class M Star.' },
    { id: 'e5', timestamp: new Date(Date.now() - 60000).toISOString(), source: 'Journal', type: 'ShieldState', severity: 'error', summary: 'Shields Offline', text: 'Generator breached. Recharging in 15s.' },
    { id: 'e6', timestamp: new Date(Date.now() - 65000).toISOString(), source: 'Journal', type: 'UnderAttack', severity: 'error', summary: 'Taking Damage', text: 'Attacker: CMDR Harry Potter (FDL)' },
    { id: 'e7', timestamp: new Date(Date.now() - 120000).toISOString(), source: 'Journal', type: 'DockingGranted', severity: 'success', summary: 'Clearance Granted', text: 'Pad 04. Welcome to Dalton Gateway.' },
    { id: 'e8', timestamp: new Date(Date.now() - 125000).toISOString(), source: 'Journal', type: 'DockingRequested', severity: 'info', summary: 'Docking Request', text: 'Station: Dalton Gateway.' },
    { id: 'e9', timestamp: new Date(Date.now() - 300000).toISOString(), source: 'Journal', type: 'FuelScoop', severity: 'warning', summary: 'Fuel Scooping', text: 'Scooping complete. Tank full.' },
    { id: 'e10', timestamp: new Date(Date.now() - 310000).toISOString(), source: 'Matrix', type: 'Delivery', severity: 'error', summary: 'Delivery Failed', text: 'Timeout connecting to homeserver' },
    { id: 'e11', timestamp: new Date(Date.now() - 360000).toISOString(), source: 'Journal', type: 'ReceiveText', severity: 'info', summary: 'Local Comms', text: 'NPC: "I\'m going to boil you up!"' },
    { id: 'e12', timestamp: new Date(Date.now() - 400000).toISOString(), source: 'Journal', type: 'Scan', severity: 'info', summary: 'Scan Complete', text: 'Target: Imperial Cutter. Wanted.' },
    { id: 'e13', timestamp: new Date(Date.now() - 500000).toISOString(), source: 'System', type: 'Service', severity: 'success', summary: 'Relay Active', text: 'Connected to Elite Dangerous API' },
    { id: 'e14', timestamp: new Date(Date.now() - 550000).toISOString(), source: 'Journal', type: 'Loadout', severity: 'info', summary: 'Ship Ready', text: 'Anaconda (ID: AX-1) initialized.' },
    { id: 'e15', timestamp: new Date(Date.now() - 600000).toISOString(), source: 'System', type: 'Connection', severity: 'info', summary: 'Connected', text: 'WebUI connected to local service' },
  ]
};

// --- Shared UI Components (Tactical HUD style) ---
const HudCard = ({ children, className = "", title, icon: Icon, rightElement }) => (
  <div className={`bg-[#060a11]/80 backdrop-blur-md border border-orange-500/20 shadow-[inset_0_0_20px_rgba(249,115,22,0.05)] rounded-sm relative overflow-hidden flex flex-col ${className}`}>
    {/* Decorative corner marks */}
    <div className="absolute top-0 left-0 w-2 h-2 border-t-2 border-l-2 border-orange-500/50"></div>
    <div className="absolute top-0 right-0 w-2 h-2 border-t-2 border-r-2 border-orange-500/50"></div>
    <div className="absolute bottom-0 left-0 w-2 h-2 border-b-2 border-l-2 border-orange-500/50"></div>
    <div className="absolute bottom-0 right-0 w-2 h-2 border-b-2 border-r-2 border-orange-500/50"></div>
    
    {title && (
      <div className="border-b border-orange-500/20 bg-orange-950/20 px-3 py-2 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-2">
          {Icon && <Icon className="w-3.5 h-3.5 text-orange-500" />}
          <h2 className="text-[10px] font-bold text-orange-500 uppercase tracking-widest">{title}</h2>
        </div>
        {rightElement}
      </div>
    )}
    <div className="p-4 flex-1 overflow-auto custom-scrollbar">
      {children}
    </div>
  </div>
);

const Badge = ({ children, variant = 'default', className = "" }) => {
  const variants = {
    default: 'bg-slate-900/80 text-slate-400 border border-slate-700',
    success: 'bg-emerald-950/80 text-emerald-400 border border-emerald-800 shadow-[0_0_10px_rgba(16,185,129,0.2)]',
    warning: 'bg-amber-950/80 text-amber-400 border border-amber-800 shadow-[0_0_10px_rgba(245,158,11,0.2)]',
    danger: 'bg-rose-950/80 text-rose-400 border border-rose-800 shadow-[0_0_10px_rgba(225,29,72,0.2)]',
    brand: 'bg-orange-950/80 text-orange-400 border border-orange-800 shadow-[0_0_10px_rgba(249,115,22,0.2)]',
  };
  return (
    <span className={`px-2 py-0.5 text-[9px] font-mono uppercase tracking-wider ${variants[variant] || variants.default} ${className}`}>
      {children}
    </span>
  );
};

const ProgressBar = ({ current, total, colorClass = "bg-orange-500", glowClass = "shadow-[0_0_8px_rgba(249,115,22,0.6)]" }) => {
  const percent = Math.min(100, Math.max(0, (current / total) * 100)) || 0;
  return (
    <div className="w-full bg-slate-900/50 border border-slate-800 h-1.5 mt-1 overflow-hidden">
      <div className={`${colorClass} ${glowClass} h-full transition-all duration-500 ease-out`} style={{ width: `${percent}%` }}></div>
    </div>
  );
};

// --- Content View Components ---
const DashboardContent = ({ state }) => {
  const formatUptime = (startStr) => {
    const diff = Date.now() - new Date(startStr).getTime();
    const h = Math.floor(diff / 3600000);
    const m = Math.floor((diff % 3600000) / 60000);
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')} HRS`;
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 pb-8 animate-in fade-in duration-500">
      
      {/* Session Overview */}
      <HudCard title="Telemetry Link" icon={Wifi} rightElement={
        state.session.active ? <Badge variant="success">ACTIVE</Badge> : <Badge variant="warning">STDBY</Badge>
      }>
        <div className="space-y-3 font-mono text-xs">
          <div className="flex justify-between items-end border-b border-orange-500/10 pb-1">
            <span className="text-slate-500 text-[10px]">CMDR</span> 
            <span className="text-orange-400 font-bold">{state.session.commander}</span>
          </div>
          <div className="flex justify-between items-end border-b border-orange-500/10 pb-1">
            <span className="text-slate-500 text-[10px]">SHIP</span> 
            <span className="text-slate-200">{state.session.ship}</span>
          </div>
          <div className="flex justify-between items-end border-b border-orange-500/10 pb-1">
            <span className="text-slate-500 text-[10px]">SYSTEM</span> 
            <span className="text-slate-200">{state.session.system}</span>
          </div>
          <div className="flex justify-between items-end border-b border-orange-500/10 pb-1">
            <span className="text-slate-500 text-[10px]">MODE</span> 
            <span className="text-slate-200">{state.session.mode}</span>
          </div>
          <div className="flex justify-between items-end pt-1">
            <span className="text-slate-500 text-[10px] flex items-center gap-1"><Clock className="w-3 h-3"/> UPTIME</span> 
            <span className="text-emerald-400">{formatUptime(state.session.startTime)}</span>
          </div>
        </div>
      </HudCard>

      {/* Ship Integrity */}
      <HudCard title="Ship Integrity" icon={Shield}>
        <div className="space-y-5">
          <div>
            <div className="flex justify-between text-[10px] font-mono mb-1">
              <span className="text-slate-500 uppercase">Hull Plating</span>
              <span className={state.status.hull > 50 ? 'text-slate-200' : 'text-rose-400 animate-pulse'}>{state.status.hull}%</span>
            </div>
            <ProgressBar current={state.status.hull} total={100} colorClass={state.status.hull > 50 ? "bg-orange-500" : "bg-rose-500"} glowClass={state.status.hull > 50 ? "shadow-[0_0_8px_rgba(249,115,22,0.6)]" : "shadow-[0_0_8px_rgba(225,29,72,0.6)]"} />
          </div>
          <div className="flex justify-between items-center bg-slate-900/30 p-2 border border-slate-800/50 rounded-sm">
            <span className="text-[10px] font-mono text-slate-500 uppercase">Deflector Shield</span>
            {state.status.shieldsUp ? <Badge variant="brand">ONLINE</Badge> : <Badge variant="danger" className="animate-pulse">OFFLINE</Badge>}
          </div>
          <div className="pt-2 border-t border-orange-500/10">
            <div className="flex justify-between text-[10px] font-mono mb-1">
              <span className="text-slate-500 uppercase">SLF Fighter (Deploy)</span>
              <span className={state.status.fighterAlive ? 'text-cyan-400' : 'text-slate-600'}>
                {state.status.fighterAlive ? `${state.status.fighterHull}%` : 'DESTROYED/MIA'}
              </span>
            </div>
            {state.status.fighterAlive && (
              <ProgressBar current={state.status.fighterHull} total={100} colorClass="bg-cyan-500" glowClass="shadow-[0_0_8px_rgba(6,182,212,0.6)]" />
            )}
          </div>
        </div>
      </HudCard>

      {/* Combat Analytics */}
      <HudCard title="Combat Analytics" icon={Crosshair}>
        <div className="grid grid-cols-2 gap-3 mb-4">
          <div className="bg-slate-900/40 border border-slate-800 p-2 text-center">
            <div className="text-2xl font-mono font-bold text-slate-200 text-shadow-glow">{state.combat.kills}</div>
            <div className="text-[9px] font-bold text-orange-500/80 uppercase tracking-widest mt-1">Total Kills</div>
          </div>
          <div className="bg-slate-900/40 border border-slate-800 p-2 text-center">
            <div className="text-2xl font-mono font-bold text-slate-200 text-shadow-glow">{state.combat.scans}</div>
            <div className="text-[9px] font-bold text-orange-500/80 uppercase tracking-widest mt-1">Total Scans</div>
          </div>
        </div>
        <div className="space-y-2 font-mono text-[10px]">
          <div className="flex justify-between border-b border-orange-500/10 pb-1"><span className="text-slate-500">BOUNTY VOUCHERS</span> <span className="text-orange-400">{state.combat.bounty.toLocaleString()} CR</span></div>
          <div className="flex justify-between border-b border-orange-500/10 pb-1"><span className="text-slate-500">FACTION MERITS</span> <span className="text-cyan-400">{state.combat.merits} PTS</span></div>
          <div className="flex justify-between pt-1"><span className="text-slate-500">KILL RATE / HR</span> <span className="text-slate-300">{state.combat.killRateRecent.toFixed(1)}</span></div>
        </div>
      </HudCard>

      {/* Core Services */}
      <HudCard title="Service Nodes" icon={Server}>
        <div className="space-y-4">
          <div className="flex justify-between items-start">
            <div className="flex flex-col gap-1">
              <span className="text-[10px] font-mono text-slate-500 uppercase flex items-center gap-1.5"><Database className="w-3 h-3 text-slate-400"/> Local Journal</span>
              <span className="text-[8px] font-mono text-slate-600 truncate w-32">{state.services.journal.file}</span>
            </div>
            {state.services.journal.status === 'ok' ? <Badge variant="success">SYNCED</Badge> : <Badge variant="danger">ERROR</Badge>}
          </div>
          <div className="flex justify-between items-start">
            <div className="flex flex-col gap-1">
              <span className="text-[10px] font-mono text-slate-500 uppercase flex items-center gap-1.5"><Activity className="w-3 h-3 text-slate-400"/> Matrix Relay</span>
              <span className="text-[8px] font-mono text-rose-500/80 truncate w-32">{state.services.matrix.lastDelivery}</span>
            </div>
            {state.services.matrix.status === 'warning' ? <Badge variant="warning">WARN</Badge> : <Badge variant="success">ONLINE</Badge>}
          </div>
          <div className="flex justify-between items-start">
            <div className="flex flex-col gap-1">
              <span className="text-[10px] font-mono text-slate-500 uppercase flex items-center gap-1.5"><Wifi className="w-3 h-3 text-slate-400"/> Web Interface</span>
              <span className="text-[8px] font-mono text-slate-600 truncate w-32">{state.services.web.bind}</span>
            </div>
            <Badge variant="success">LISTENING</Badge>
          </div>
        </div>
      </HudCard>
      
      {/* Active Missions Summary - Added Fixed Height to allow scrolling */}
      <HudCard className="md:col-span-2 h-[320px]" title="Active Missions" icon={List} rightElement={<Badge variant="default">TOTAL {state.missions.length}</Badge>}>
        <div className="space-y-2">
          {state.missions.map(m => (
              <div key={m.id} className="flex justify-between items-center p-2 bg-slate-900/40 border border-slate-800/50 rounded-sm">
                <div className="flex flex-col">
                  <span className="text-xs font-medium text-slate-200 truncate">{m.name}</span>
                  <span className="text-[9px] font-mono text-orange-500/70 uppercase tracking-widest">{m.kind}</span>
                </div>
                <div className="text-right">
                  <div className="text-[10px] font-mono text-slate-400">
                    {m.progress.current !== undefined ? `${m.progress.current} / ${m.progress.required}` : m.state.toUpperCase()}
                  </div>
                  {m.progress.current !== undefined && (
                    <div className="w-16 mt-1 ml-auto">
                      <ProgressBar current={m.progress.current} total={m.progress.required} colorClass="bg-orange-500" />
                    </div>
                  )}
                </div>
              </div>
          ))}
        </div>
      </HudCard>

      {/* Recent Alerts Summary - Added Fixed Height to allow scrolling */}
      <HudCard className="md:col-span-2 h-[320px]" title="Recent Alerts" icon={AlertTriangle}>
        <div className="space-y-2 font-mono text-[10px]">
          {state.events.map(e => (
              <div key={e.id} className="flex gap-3 p-1.5 border-l-2 border-slate-800 pl-2 bg-slate-900/20">
                <span className="text-slate-600 whitespace-nowrap">{new Date(e.timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit', second:'2-digit'})}</span>
                <span className={`truncate ${e.severity === 'warning' ? 'text-amber-400' : e.severity === 'error' ? 'text-rose-500' : e.severity === 'success' ? 'text-emerald-400' : 'text-slate-300'}`}>
                  [{e.source.toUpperCase()}] {e.summary.toUpperCase()}: {e.text}
                </span>
              </div>
          ))}
        </div>
      </HudCard>
    </div>
  );
};

const MissionsContent = ({ state }) => {
  const [selectedId, setSelectedId] = useState(state.missions[0]?.id || null);
  const selectedMission = state.missions.find(m => m.id === selectedId);

  return (
    <div className="flex h-[calc(100vh-140px)] gap-4 animate-in fade-in duration-500">
      {/* Master List */}
      <HudCard title="Mission Directory" className="w-1/3 shrink-0">
        <div className="space-y-2 pr-1">
          {state.missions.length === 0 ? (
            <div className="text-center py-8 text-slate-600 text-xs font-mono">NO ACTIVE MISSIONS</div>
          ) : (
            state.missions.map(m => (
              <button 
                key={m.id}
                onClick={() => setSelectedId(m.id)}
                className={`w-full text-left p-3 border rounded-sm transition-all text-xs flex flex-col gap-2 ${
                  selectedId === m.id 
                    ? 'bg-orange-950/30 border-orange-500 shadow-[inset_0_0_15px_rgba(249,115,22,0.15)]' 
                    : 'bg-slate-900/30 border-slate-800 hover:border-slate-600'
                }`}
              >
                <div className="flex justify-between items-start">
                  <div className="font-bold text-slate-200 truncate pr-2">{m.name}</div>
                  {m.state === 'active' ? <div className="w-2 h-2 bg-emerald-500 shadow-[0_0_5px_#10b981] mt-1 shrink-0" /> : <CheckCircle2 className="w-3 h-3 text-slate-600 shrink-0" />}
                </div>
                <div className="flex justify-between text-[9px] font-mono text-slate-500">
                  <span className="uppercase">{m.kind}</span>
                  {m.progress.current !== undefined && <span>{m.progress.current}/{m.progress.required}</span>}
                </div>
                {m.progress.current !== undefined && (
                  <ProgressBar current={m.progress.current} total={m.progress.required} colorClass={selectedId === m.id ? "bg-orange-500" : "bg-slate-600"} glowClass={selectedId === m.id ? "shadow-[0_0_8px_rgba(249,115,22,0.6)]" : "none"} />
                )}
              </button>
            ))
          )}
        </div>
      </HudCard>

      {/* Detail View */}
      <HudCard title="Mission Intel" className="flex-1">
        {selectedMission ? (
          <div className="animate-in fade-in h-full">
            <div className="mb-6 pb-4 border-b border-orange-500/20">
              <div className="flex gap-2 mb-3">
                <Badge variant="brand">{selectedMission.kind}</Badge>
                <Badge variant={selectedMission.state === 'active' ? 'success' : 'default'}>{selectedMission.state}</Badge>
              </div>
              <h1 className="text-2xl font-black uppercase tracking-wider text-slate-100 text-shadow-glow">{selectedMission.name}</h1>
              <p className="text-orange-500/50 font-mono text-[9px] mt-1 tracking-widest">ID: {selectedMission.id.toUpperCase()}</p>
            </div>

            <div className="grid grid-cols-2 gap-8 font-mono text-[10px]">
              <div className="space-y-6">
                <section>
                  <h3 className="text-orange-500 mb-2 border-b border-slate-800 pb-1 tracking-widest font-bold">FACTION & DESTINATION</h3>
                  <div className="space-y-1">
                    <div className="flex justify-between"><span className="text-slate-600">ISSUING</span> <span className="text-slate-300">{selectedMission.issuingFaction}</span></div>
                    <div className="flex justify-between"><span className="text-slate-600">TARGET</span> <span className="text-rose-400">{selectedMission.targetFaction}</span></div>
                    <div className="flex justify-between"><span className="text-slate-600">DESTINATION</span> <span className="text-slate-300">{selectedMission.destination}</span></div>
                  </div>
                </section>
                <section>
                  <h3 className="text-orange-500 mb-2 border-b border-slate-800 pb-1 tracking-widest font-bold">TIMETABLE & REWARD</h3>
                  <div className="space-y-1">
                    <div className="flex justify-between"><span className="text-slate-600">ACCEPTED</span> <span className="text-slate-300">{new Date(selectedMission.acceptedTime).toLocaleString()}</span></div>
                    <div className="flex justify-between"><span className="text-slate-600">EXPIRY</span> <span className="text-amber-400">{new Date(selectedMission.expiryTime).toLocaleString()}</span></div>
                    <div className="flex justify-between mt-2 pt-2 border-t border-slate-800/50"><span className="text-slate-600">PAYOUT</span> <span className="text-cyan-400 font-bold text-xs">{selectedMission.reward.toLocaleString()} CR</span></div>
                  </div>
                </section>
              </div>

              <div>
                <div className="bg-slate-900/50 border border-slate-800 p-4 rounded-sm relative">
                   <div className="absolute top-0 left-0 w-8 h-8 border-t border-l border-orange-500/30 opacity-50"></div>
                   <h3 className="text-orange-500 mb-4 tracking-widest font-bold flex items-center gap-2">
                     <Activity className="w-3 h-3" /> TRACKING UPLINK
                   </h3>
                   
                   {selectedMission.kind === 'massacre' && (
                    <div className="space-y-3">
                      <div className="flex justify-between items-end">
                        <span className="text-slate-500">CONFIRMED KILLS</span>
                        <span className="text-lg font-bold text-slate-200">{selectedMission.progress.current} <span className="text-slate-600 text-xs">/ {selectedMission.progress.required}</span></span>
                      </div>
                      <ProgressBar current={selectedMission.progress.current} total={selectedMission.progress.required} />
                      <p className="text-slate-500 text-[9px] mt-4 leading-relaxed">
                        TARGET LOCK REQUIRED: VALID BOUNTIES FOR FACTION "{selectedMission.targetFaction.toUpperCase()}" ONLY.
                      </p>
                    </div>
                  )}

                  {selectedMission.kind === 'trade' && (
                    <div className="space-y-3">
                      <div className="flex justify-between text-orange-400 border-b border-slate-800 pb-2 mb-2 font-bold text-xs">
                        <span>{selectedMission.progress.commodity.toUpperCase()}</span>
                      </div>
                      <div className="flex justify-between"><span className="text-slate-600">REQUIRED UNITS</span> <span>{selectedMission.progress.required}</span></div>
                      <div className="flex justify-between"><span className="text-slate-600">COLLECTED</span> <span className="text-cyan-400">{selectedMission.progress.collected}</span></div>
                      <div className="flex justify-between"><span className="text-slate-600">DELIVERED</span> <span className="text-emerald-400">{selectedMission.progress.delivered}</span></div>
                      
                      <div className="pt-2 mt-2 border-t border-slate-800">
                         <div className="flex justify-between text-[8px] mb-1 text-slate-500"><span>DELIVERY SATISFACTION</span></div>
                         <ProgressBar current={selectedMission.progress.delivered} total={selectedMission.progress.required} colorClass="bg-emerald-500" glowClass="shadow-[0_0_8px_rgba(16,185,129,0.6)]" />
                      </div>
                      {selectedMission.progress.cargoDepot && (
                         <div className="mt-4"><Badge variant="default">CARGO DEPOT AUTHORIZED</Badge></div>
                      )}
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex items-center justify-center h-full text-slate-600 text-xs font-mono uppercase tracking-widest">
            Awaiting selection
          </div>
        )}
      </HudCard>
    </div>
  );
};

const EventsContent = ({ state }) => {
  return (
    <HudCard title="System Telemetry Feed" className="h-[calc(100vh-140px)] animate-in fade-in duration-500 relative">
      {/* Sticky header fixes for smooth scrolling underneath */}
      <div className="sticky -top-4 -mx-4 px-4 pt-4 bg-[#060a11]/95 backdrop-blur-md z-10 border-b border-orange-500/20 pb-2 mb-2 grid grid-cols-12 gap-4 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono shadow-[0_5px_15px_-5px_rgba(0,0,0,0.5)]">
        <div className="col-span-2">TIMESTAMP</div>
        <div className="col-span-2">ORIGIN</div>
        <div className="col-span-8">PAYLOAD DECODE</div>
      </div>
      <div className="space-y-1 pb-4">
        {state.events.map(event => (
          <div key={event.id} className="grid grid-cols-12 gap-4 text-[10px] font-mono p-2 hover:bg-slate-900/60 border-b border-slate-800/30 transition-colors">
            <div className="col-span-2 text-slate-500 pt-0.5">
              {new Date(event.timestamp).toLocaleTimeString([], { hour12: false })}
            </div>
            <div className="col-span-2">
              <Badge variant={event.source === 'Matrix' ? 'brand' : 'default'} className="!text-[8px]">{event.source}</Badge>
            </div>
            <div className="col-span-8">
              <div className={`font-bold mb-0.5 flex items-center gap-2 ${
                event.severity === 'warning' ? 'text-amber-400' :
                event.severity === 'error' ? 'text-rose-500' :
                event.severity === 'success' ? 'text-emerald-400' : 'text-slate-300'
              }`}>
                {event.severity === 'warning' && <AlertTriangle className="w-3 h-3" />}
                {event.severity === 'error' && <XCircle className="w-3 h-3" />}
                {event.severity === 'success' && <CheckCircle2 className="w-3 h-3" />}
                {event.summary.toUpperCase()}
              </div>
              <div className="text-slate-500">{event.text}</div>
            </div>
          </div>
        ))}
      </div>
    </HudCard>
  );
};

const ConfigContent = ({ state }) => {
  return (
    <div className="max-w-4xl mx-auto pb-12 animate-in fade-in duration-500">
      <HudCard title="System Configuration" icon={Settings} className="overflow-visible">
        <div className="space-y-6 p-2">
          {/* Section 1: Data Source */}
          <section className="border border-slate-800 bg-slate-900/20 p-5 rounded-sm relative">
            <h3 className="absolute -top-3 left-4 bg-[#060a11] px-2 text-[10px] font-bold text-orange-500 tracking-widest flex items-center gap-2">
              <Database className="w-3 h-3" /> LOCAL INGESTION
            </h3>
            <div className="mt-2">
              <label className="block text-[10px] font-mono text-slate-500 uppercase mb-2">Override Journal Path (Optional)</label>
              <div className="flex gap-2">
                <input 
                  type="text" 
                  placeholder="LEAVE BLANK FOR SYSTEM DEFAULT" 
                  defaultValue=""
                  className="flex-1 bg-black/50 border border-slate-800 rounded-sm px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-orange-500/50 transition-colors"
                />
                <button className="bg-slate-800 hover:bg-slate-700 px-6 py-2 rounded-sm text-[10px] font-bold uppercase tracking-wider text-slate-300 transition-colors">
                  BROWSE
                </button>
              </div>
            </div>
          </section>

          {/* Section 2: Matrix Push */}
          <section className="border border-slate-800 bg-slate-900/20 p-5 rounded-sm relative">
             <h3 className="absolute -top-3 left-4 bg-[#060a11] px-2 text-[10px] font-bold text-orange-500 tracking-widest flex items-center gap-2">
              <Activity className="w-3 h-3" /> MATRIX RELAY PROTOCOL
            </h3>
            <div className="mt-2 grid grid-cols-2 gap-5">
              <div className="col-span-2 flex items-center gap-3 bg-slate-900/50 p-2 border border-slate-800/50">
                <input type="checkbox" id="matrix_enabled" defaultChecked={state.services.matrix.enabled} className="w-4 h-4 accent-orange-500 bg-slate-900 border-slate-700" />
                <label htmlFor="matrix_enabled" className="text-xs font-mono text-slate-200 uppercase tracking-widest">Enable Matrix Broadcasting</label>
              </div>
              
              <div>
                <label className="block text-[10px] font-mono text-slate-500 uppercase mb-2">Homeserver URI</label>
                <input type="text" defaultValue="https://matrix.org" className="w-full bg-black/50 border border-slate-800 rounded-sm px-3 py-2 text-xs font-mono text-slate-200 focus:border-orange-500/50 outline-none" />
              </div>
              <div>
                <label className="block text-[10px] font-mono text-slate-500 uppercase mb-2">Target Room ID</label>
                <input type="text" defaultValue="!room_id:matrix.org" className="w-full bg-black/50 border border-slate-800 rounded-sm px-3 py-2 text-xs font-mono text-slate-200 focus:border-orange-500/50 outline-none" />
              </div>

              <div className="col-span-2 pt-4 border-t border-slate-800/50">
                <label className="flex justify-between text-[10px] font-mono uppercase mb-2">
                  <span className="text-slate-500">Access Token <span className="text-rose-500/80 ml-1">(WRITE-ONLY)</span></span>
                  <span className="text-emerald-500 font-bold">TOKEN PRESENT IN VAULT</span>
                </label>
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <input type="password" placeholder="••••••••••••••••••••••••" className="w-full bg-black/50 border border-slate-800 rounded-sm pl-3 pr-10 py-2 text-xs font-mono text-slate-200 focus:border-orange-500/50 outline-none" />
                    <EyeOff className="w-4 h-4 text-slate-600 absolute right-3 top-2" />
                  </div>
                  <button className="flex items-center gap-2 border border-rose-900/50 text-rose-500 bg-rose-950/20 hover:bg-rose-900/40 px-4 py-2 rounded-sm text-[10px] font-bold uppercase tracking-wider transition-colors">
                    <Trash2 className="w-3.5 h-3.5" /> PURGE
                  </button>
                </div>
              </div>
            </div>
          </section>

          {/* Section 3: Web UI/API */}
          <section className="border border-slate-800 bg-slate-900/20 p-5 rounded-sm relative">
             <h3 className="absolute -top-3 left-4 bg-[#060a11] px-2 text-[10px] font-bold text-orange-500 tracking-widest flex items-center gap-2">
              <Server className="w-3 h-3" /> LOCAL API GATEWAY
            </h3>
            <div className="mt-2 grid grid-cols-2 gap-5">
               <div>
                <label className="block text-[10px] font-mono text-slate-500 uppercase mb-2">Bind Address</label>
                <input type="text" defaultValue="127.0.0.1" className="w-full bg-black/50 border border-slate-800 rounded-sm px-3 py-2 text-xs font-mono text-slate-200 focus:border-orange-500/50 outline-none" />
              </div>
              <div>
                <label className="block text-[10px] font-mono text-slate-500 uppercase mb-2">TCP Port</label>
                <input type="number" defaultValue={8080} className="w-full bg-black/50 border border-slate-800 rounded-sm px-3 py-2 text-xs font-mono text-slate-200 focus:border-orange-500/50 outline-none" />
              </div>
            </div>
          </section>
        </div>

        {/* Global Save */}
        <div className="mt-6 flex justify-end border-t border-orange-500/10 pt-4 px-2">
          <button className="flex items-center gap-2 bg-orange-600 hover:bg-orange-500 text-slate-100 px-5 py-2 rounded-sm text-[10px] font-bold uppercase tracking-widest transition-all shadow-[0_0_10px_rgba(234,88,12,0.4)]">
            <Save className="w-3.5 h-3.5" /> Commit Modifications
          </button>
        </div>
      </HudCard>
    </div>
  );
};

// --- Main Layout Wrapper ---
const TopTabLayout = ({ activeTab, setActiveTab, tabs, state, children }) => (
  <div className="flex flex-col h-screen w-full relative z-10 text-slate-300 font-sans">
    <header className="h-14 bg-[#04070d]/90 backdrop-blur-md border-b border-orange-500/20 flex items-center justify-between px-6 shrink-0 shadow-lg z-20">
      {/* Brand */}
      <div className="flex items-center gap-3 w-48">
        <div className="w-6 h-6 border border-orange-500/40 rounded-sm flex items-center justify-center bg-orange-950/20 shadow-[inset_0_0_8px_rgba(249,115,22,0.3)]">
          <EyeOff className="w-3 h-3 text-orange-400" />
        </div>
        <h1 className="font-black text-slate-200 tracking-wider text-sm uppercase text-shadow-glow">ed-sentry</h1>
      </div>

      {/* Tabs */}
      <div className="flex gap-2">
        {tabs.map(tab => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-4 py-2 rounded-sm text-[10px] font-bold uppercase tracking-widest transition-all duration-200 ${
                isActive 
                  ? 'bg-orange-500/10 text-orange-400 border-b-2 border-orange-500 shadow-[inset_0_-10px_10px_-10px_rgba(249,115,22,0.5)]' 
                  : 'text-slate-400 hover:bg-slate-900/60 hover:text-slate-200 border-b-2 border-transparent'
              }`}
            >
              <Icon className={`w-3.5 h-3.5 ${isActive ? 'text-orange-500 drop-shadow-[0_0_5px_rgba(249,115,22,0.8)]' : 'opacity-60'}`} />
              {tab.label}
            </button>
          )
        })}
      </div>

      {/* Status */}
      <div className="flex items-center justify-end gap-3 w-48">
         <span className="text-emerald-500 flex items-center gap-1.5 text-[9px] font-mono"><span className="w-1.5 h-1.5 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_5px_#10b981]"></span> SYNCED</span>
      </div>
    </header>
    <main className="flex-1 overflow-y-auto p-6 relative custom-scrollbar bg-gradient-to-b from-[#04070d]/50 to-transparent">
      <div className="flex justify-between items-center mb-6 border-b border-orange-500/10 pb-4">
        <div>
          <h1 className="text-xl font-bold uppercase tracking-widest text-slate-200 text-shadow-glow">{activeTab} Interface</h1>
          <span className="text-[9px] font-mono text-slate-500 uppercase">SYS_RELAY: ONLINE</span>
        </div>
      </div>
      {children}
    </main>
  </div>
);

// --- Root App Component ---
export default function App() {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [liveState, setLiveState] = useState(MOCK_STATE);
  const [pulseActive, setPulseActive] = useState(false);

  const tabs = [
    { id: 'dashboard', label: 'Telemetry', icon: Activity },
    { id: 'missions', label: 'Missions', icon: List }, // Restore the Missions label.
    { id: 'events', label: 'Comms Feed', icon: Terminal },
    { id: 'config', label: 'Systems', icon: Settings },
  ];

  // Data simulator effect.
  useEffect(() => {
    if (!pulseActive) return;
    const interval = setInterval(() => {
      setLiveState(prev => {
        const nextHull = Math.max(10, prev.status.hull - (Math.random() > 0.8 ? Math.floor(Math.random() * 5) : 0));
        const nextBounty = prev.combat.bounty + (Math.random() > 0.5 ? Math.floor(Math.random() * 500000) : 0);
        return {
          ...prev,
          status: { ...prev.status, hull: nextHull },
          combat: { ...prev.combat, bounty: nextBounty, kills: prev.combat.kills + (Math.random() > 0.9 ? 1 : 0) }
        };
      });
    }, 2500);
    return () => clearInterval(interval);
  }, [pulseActive]);

  const renderWorkspaceContent = () => {
    switch (activeTab) {
      case 'dashboard': return <DashboardContent state={liveState} />;
      case 'missions': return <MissionsContent state={liveState} />;
      case 'events': return <EventsContent state={liveState} />;
      case 'config': return <ConfigContent state={liveState} />;
      default: return <DashboardContent state={liveState} />;
    }
  };

  return (
    <div className="h-screen w-full bg-[#03060a] relative select-none overflow-hidden bg-tactical font-sans">
      
      {/* Tactical Style Core - inject background grid and scanline effects */}
      <style>{`
        .custom-scrollbar::-webkit-scrollbar { width: 4px; }
        .custom-scrollbar::-webkit-scrollbar-track { background: rgba(15, 23, 42, 0.5); }
        .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(249, 115, 22, 0.3); border-radius: 0px; }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #f97316; }
        
        .text-shadow-glow { text-shadow: 0 0 10px rgba(249, 115, 22, 0.4); }

        .bg-tactical {
          background-color: #03060a;
          background-image: 
            linear-gradient(rgba(30, 41, 59, 0.2) 1px, transparent 1px),
            linear-gradient(90deg, rgba(30, 41, 59, 0.2) 1px, transparent 1px);
          background-size: 24px 24px;
          position: relative;
        }
        .bg-tactical::after {
          content: "";
          position: absolute;
          top: 0; left: 0; right: 0; bottom: 0;
          background: linear-gradient(rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.15) 50%);
          background-size: 100% 4px;
          pointer-events: none;
          z-index: 100;
          opacity: 0.35;
        }
      `}</style>

      {/* Floating Data Controls (Simulator) */}
      <div className="absolute bottom-4 right-4 z-[110] flex flex-col gap-2 pointer-events-none">
        {/* Data Simulator Toggle */}
        <button
          onClick={() => setPulseActive(!pulseActive)}
          className={`flex items-center justify-center gap-1 px-3 py-2 rounded-sm text-[8px] font-bold uppercase tracking-wider font-mono transition-all pointer-events-auto ${
            pulseActive 
              ? 'bg-emerald-950/40 text-emerald-400 border border-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.3)]' 
              : 'bg-[#04070d]/90 backdrop-blur border border-orange-500/30 text-orange-400 hover:bg-orange-500/10'
          }`}
        >
          <Play className={`w-3 h-3 ${pulseActive ? 'animate-spin' : ''}`} />
          {pulseActive ? 'PULSING' : 'SIMULATE'}
        </button>
      </div>

      <TopTabLayout activeTab={activeTab} setActiveTab={setActiveTab} tabs={tabs} state={liveState}>
        {renderWorkspaceContent()}
      </TopTabLayout>

    </div>
  );
}
