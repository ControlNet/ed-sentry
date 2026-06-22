import React, { useState, useEffect } from 'react';
import { Loader2, RefreshCw, EyeOff } from 'lucide-react';

export default function SimpleLoader() {
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState('INITIALIZING...');
  const [isComplete, setIsComplete] = useState(false);

  // Simulate a loading sequence of roughly 3.5 seconds.
  useEffect(() => {
    let currentProgress = 0;
    const duration = 3500; 
    const intervalTime = 35;
    const totalSteps = duration / intervalTime;

    const timer = setInterval(() => {
      currentProgress += (100 / totalSteps);
      
      if (currentProgress >= 100) {
        currentProgress = 100;
        clearInterval(timer);
        setIsComplete(true);
      }

      setProgress(Math.floor(currentProgress));

      // Minimal self-check states tuned for the ED-Sentry context.
      if (currentProgress < 20) {
        setStatus('RESOLVING JOURNAL DIR...');
      } else if (currentProgress < 50) {
        setStatus('PARSING FLIGHT JOURNAL...');
      } else if (currentProgress < 85) {
        setStatus('ESTABLISHING MATRIX RELAY...');
      } else if (currentProgress < 100) {
        setStatus('SYNCING COMMANDER DATA...');
      } else {
        setStatus('SYSTEM READY // FLY SAFE');
      }

    }, intervalTime);

    return () => clearInterval(timer);
  }, []);

  const handleRestart = () => {
    setProgress(0);
    setIsComplete(false);
    setStatus('INITIALIZING...');
    // Simple and efficient refresh path.
    window.location.reload();
  };

  // Circular progress math with a radius of 70.
  const radius = 70;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (progress / 100) * circumference;

  return (
    <div className="flex h-screen w-full flex-col items-center justify-center bg-[#03060a] bg-tactical font-mono text-slate-300 selection:bg-orange-500/30 relative select-none">
      
      {}
      <style>{`
        /* Inject the tactical holographic grid and CRT scanlines. */
        .bg-tactical {
          background-color: #03060a;
          background-image: 
            linear-gradient(rgba(249, 115, 22, 0.03) 1px, transparent 1px),
            linear-gradient(90deg, rgba(249, 115, 22, 0.03) 1px, transparent 1px);
          background-size: 24px 24px;
          position: relative;
        }
        .bg-tactical::after {
          content: "";
          position: absolute;
          top: 0; left: 0; right: 0; bottom: 0;
          background: linear-gradient(rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.2) 50%);
          background-size: 100% 4px;
          pointer-events: none;
          z-index: 100;
          opacity: 0.35;
        }
        .text-glow-orange {
          text-shadow: 0 0 10px rgba(249, 115, 22, 0.6);
        }
        .text-glow-emerald {
          text-shadow: 0 0 10px rgba(16, 185, 129, 0.6);
        }
      `}</style>

      {/* Background holographic soft glow */}
      <div className="absolute inset-0 z-0 flex items-center justify-center overflow-hidden pointer-events-none">
        <div className="h-[350px] w-[350px] rounded-full bg-orange-600/5 blur-[100px]" />
      </div>

      <div className="relative z-10 flex flex-col items-center gap-8">
        
        {}
        {/* Central circular progress indicator with an orange holographic filament effect */}
        <div className="relative flex items-center justify-center">
          {/* Animated SVG ring */}
          <svg className="h-44 w-44 -rotate-90 transform" viewBox="0 0 160 160">
            {/* Dark base ring that suggests an unpowered conduit */}
            <circle 
              cx="80" cy="80" r={radius} 
              className="fill-none stroke-orange-950/20 stroke-[3px]" 
            />
            {/* Bright progress ring that suggests an energized conduit */}
            <circle 
              cx="80" cy="80" r={radius} 
              className={`fill-none stroke-[3px] transition-all duration-75 ease-out ${
                isComplete 
                  ? 'stroke-emerald-500 drop-shadow-[0_0_6px_rgba(16,185,129,0.8)]' 
                  : 'stroke-orange-500 drop-shadow-[0_0_6px_rgba(249,115,22,0.8)]'
              }`}
              strokeLinecap="square"
              strokeDasharray={circumference}
              strokeDashoffset={strokeDashoffset}
            />
          </svg>

          {/* Center percentage text and pulsing core logo */}
          <div className="absolute flex flex-col items-center justify-center">
            {isComplete ? (
              <EyeOff className="w-8 h-8 text-emerald-400 animate-in zoom-in duration-300 drop-shadow-[0_0_6px_rgba(16,185,129,0.8)]" />
            ) : (
              <span className="text-3xl font-bold tracking-tighter text-orange-400 text-glow-orange">
                {progress}<span className="text-sm text-orange-500/70 ml-0.5">%</span>
              </span>
            )}
          </div>
        </div>

        {/* Bottom status text and micro progress bar */}
        <div className="flex flex-col items-center gap-3">
          <div className="flex items-center gap-2">
            {!isComplete && <Loader2 className="h-3 w-3 animate-spin text-orange-500" />}
            <h2 className={`text-[10px] font-bold tracking-[0.2em] uppercase transition-colors duration-300 ${
              isComplete ? 'text-emerald-400 text-glow-emerald' : 'text-orange-500/80 text-glow-orange'
            }`}>
              {status}
            </h2>
          </div>

          {/* Auxiliary linear scale that echoes the main holographic conduit */}
          <div className="h-1 w-40 overflow-hidden rounded-none border border-orange-500/10 bg-slate-950/80 p-0.5">
            <div 
              className={`h-full transition-all duration-75 ease-out ${
                isComplete ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]' : 'bg-orange-500 shadow-[0_0_8px_rgba(249,115,22,0.6)]'
              }`} 
              style={{ width: `${progress}%` }} 
            />
          </div>
        </div>

        {/* Debug restart button shown after completion with holographic button styling */}
        <div className={`transition-all duration-500 ${isComplete ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2 pointer-events-none'}`}>
          <button 
            onClick={handleRestart}
            className="flex items-center gap-2 rounded-sm border border-orange-500/20 bg-orange-950/5 px-4 py-1.5 text-[9px] font-bold tracking-widest uppercase text-orange-400 transition-all hover:bg-orange-500/10 hover:border-orange-500/40"
          >
            <RefreshCw className="h-3 w-3" />
            RE-BOOT LOADER
          </button>
        </div>

      </div>
    </div>
  );
}
