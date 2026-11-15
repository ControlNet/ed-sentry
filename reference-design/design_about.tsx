import React, { useState } from 'react';
import { 
  EyeOff, Info, Github, ExternalLink, ShieldAlert
} from 'lucide-react';

// --- 通用 UI 组件 (Tactical HUD 风格) ---
const HudCard = ({ children, className = "", title, icon: Icon, rightElement }) => (
  <div className={`bg-[#060a11]/80 backdrop-blur-md border border-orange-500/20 shadow-[inset_0_0_20px_rgba(249,115,22,0.05)] rounded-sm relative overflow-hidden flex flex-col ${className}`}>
    {/* 装饰性角标 */}
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
    brand: 'bg-orange-950/80 text-orange-400 border border-orange-800 shadow-[0_0_10px_rgba(249,115,22,0.2)]',
  };
  return (
    <span className={`px-2 py-0.5 text-[9px] font-mono uppercase tracking-wider ${variants[variant] || variants.default} ${className}`}>
      {children}
    </span>
  );
};

// --- About 页面组件 ---
const AboutContent = () => {
  return (
    <div className="max-w-2xl mx-auto pb-12 animate-in fade-in duration-500">
      
      {/* 顶部身份牌 (无 Logo，高质感文字排版) */}
      <div className="mb-8 flex flex-col items-center justify-center border-b border-orange-500/20 pb-8 text-center relative">
        <div className="absolute inset-0 bg-orange-500/5 blur-[80px] pointer-events-none rounded-full"></div>
        <h1 className="text-4xl font-black text-slate-100 tracking-[0.3em] uppercase text-shadow-glow mb-2">
          ed-sentry
        </h1>
        <p className="text-xs text-orange-500/80 font-mono tracking-[0.2em] uppercase max-w-lg">
          Elite Dangerous Automated AFK Monitoring & Telemetry Uplink System
        </p>
      </div>

      <div className="space-y-6">
        
        {/* 项目主要元数据 */}
        <HudCard title="System Information" icon={Info}>
          <div className="space-y-3 font-mono text-xs uppercase">
            <div className="flex justify-between border-b border-slate-800/60 pb-2.5">
              <span className="text-slate-500">Current Version</span>
              <span className="text-orange-400 font-bold">v0.1.0-ALPHA</span>
            </div>
            
            <div className="flex justify-between border-b border-slate-800/60 pb-2.5">
              <span className="text-slate-500">Project Author</span>
              <span className="text-slate-200">CMDR OpenSource</span>
            </div>

            <div className="flex justify-between border-b border-slate-800/60 pb-2.5">
              <span className="text-slate-500">Software License</span>
              <span className="text-slate-200">MIT License</span>
            </div>

            <div className="flex justify-between items-center pt-1.5">
              <span className="text-slate-500">Git Repository</span>
              <a 
                href="https://github.com/your-repo/ed-sentry" 
                target="_blank" 
                rel="noopener noreferrer" 
                className="flex items-center gap-1.5 text-orange-400 hover:text-orange-300 transition-colors"
              >
                <Github className="w-4 h-4" />
                <span>Source Repository</span>
                <ExternalLink className="w-3 h-3" />
              </a>
            </div>
          </div>
        </HudCard>

        {/* 免责声明 */}
        <HudCard className="border-rose-900/30" title="Legal & Compliance" icon={ShieldAlert}>
          <div className="p-4 bg-rose-950/10 border border-rose-900/30 text-rose-500/80 font-mono text-[9px] leading-loose text-justify rounded-sm">
            <strong className="text-rose-500 text-[10px] tracking-widest uppercase block mb-2">WARNING: Unofficial Software</strong>
            "ed-sentry" is a third-party application and is not affiliated with, endorsed, or sponsored by Frontier Developments plc. 
            All in-game assets, names, and concepts are the property of Frontier Developments. 
            This tool strictly adheres to the read-only constraints of the local Journal files and does not inject, modify, or automate any client-side game processes. 
            Use at your own risk. Fly Dangerously, Commander.
          </div>
        </HudCard>

      </div>
    </div>
  );
};

// --- 主要布局包装器 ---
const SinglePageLayout = ({ children }) => (
  <div className="flex flex-col h-screen w-full relative z-10 text-slate-300 font-sans">
    <header className="h-14 bg-[#04070d]/90 backdrop-blur-md border-b border-orange-500/20 flex items-center justify-between px-6 shrink-0 shadow-lg z-20">
      <div className="flex items-center gap-3 w-48">
        <div className="w-6 h-6 border border-orange-500/40 rounded-sm flex items-center justify-center bg-orange-950/20 shadow-[inset_0_0_8px_rgba(249,115,22,0.3)]">
          <EyeOff className="w-3 h-3 text-orange-400" />
        </div>
        <h1 className="font-black text-slate-200 tracking-wider text-sm uppercase text-shadow-glow">ed-sentry</h1>
      </div>

      <div className="flex items-center gap-2">
        <div className="flex items-center gap-2 px-4 py-2 rounded-sm text-[10px] font-bold uppercase tracking-widest text-orange-400 bg-orange-500/10 border-b-2 border-orange-500 shadow-[inset_0_-10px_10px_-10px_rgba(249,115,22,0.5)]">
          <Info className="w-3.5 h-3.5 text-orange-500 drop-shadow-[0_0_5px_rgba(249,115,22,0.8)]" />
          About
        </div>
      </div>

      <div className="flex items-center justify-end gap-3 w-48">
         <span className="text-emerald-500 flex items-center gap-1.5 text-[9px] font-mono"><span className="w-1.5 h-1.5 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_5px_#10b981]"></span> ONLINE</span>
      </div>
    </header>
    <main className="flex-1 overflow-y-auto p-6 relative custom-scrollbar bg-gradient-to-b from-[#04070d]/50 to-transparent">
      <div className="flex justify-between items-center mb-6 border-b border-orange-500/10 pb-4">
        <div>
          <h1 className="text-xl font-bold uppercase tracking-widest text-slate-200 text-shadow-glow">
            About Interface
          </h1>
          <span className="text-[9px] font-mono text-slate-500 uppercase">SYS_RELAY: ACTIVE</span>
        </div>
      </div>
      {children}
    </main>
  </div>
);

// --- 根应用组件 ---
export default function App() {
  return (
    <div className="h-screen w-full bg-[#03060a] relative select-none overflow-hidden bg-tactical font-sans">
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

      <SinglePageLayout>
        <AboutContent />
      </SinglePageLayout>
    </div>
  );
}
