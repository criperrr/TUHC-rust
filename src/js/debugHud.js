/**
 * The Unofficial Homestuck Collection (Tauri 2.0 Edition)
 * Diagnostic Monitoring & Performance HUD Overlay
 *
 * Real-time monitoring for:
 * - FPS & Frame rendering time (requestAnimationFrame moving average)
 * - Jank & dropped frame tracker (>33.3ms lag spikes)
 * - Process CPU % & Resident Set Size (RSS MB vs 150MB SLA ceiling)
 * - Active thread count & process uptime
 * - Story page, active theme & enabled mods context
 * - Zero-overhead design: paused when hidden, <0.1% CPU when active
 * - Hotkeys: F3 or Ctrl+Shift+D to toggle, [-] to minimize to micro badge
 */

(function (root) {
  'use strict';

  // State
  let isVisible = false;
  let isMinimized = false;
  let rafId = null;
  let pollIntervalId = null;

  // Frame statistics
  let frameCount = 0;
  let lastFrameTime = performance.now();
  let lastFpsUpdateTime = performance.now();
  let currentFps = 60;
  let currentFrameTime = 16.6;
  let jankCount = 0;

  // Backend system metrics
  let latestMetrics = {
    rss_mb: 0,
    cpu_pct: 0,
    threads: 1,
    uptime_secs: 0
  };

  // DOM Elements
  let hudContainer = null;
  let hudFpsEl = null;
  let hudJankEl = null;
  let hudCpuEl = null;
  let hudRssEl = null;
  let hudThreadsEl = null;
  let hudUptimeEl = null;
  let hudPageEl = null;
  let hudThemeEl = null;
  let hudModsEl = null;
  let hudBodyEl = null;
  let hudMiniContentEl = null;
  let hudMinBtn = null;

  function formatUptime(secs) {
    const s = Math.floor(secs % 60);
    const m = Math.floor((secs / 60) % 60);
    const h = Math.floor(secs / 3600);
    if (h > 0) {
      return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
    }
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  function getFpsColorClass(fps) {
    if (fps >= 50) return 'hudGreen';
    if (fps >= 30) return 'hudYellow';
    return 'hudRed';
  }

  function getRssColorClass(rss) {
    if (rss >= 150) return 'hudRed';
    if (rss >= 120) return 'hudYellow';
    return 'hudGreen';
  }

  function getCpuColorClass(cpu) {
    if (cpu >= 50) return 'hudRed';
    if (cpu >= 25) return 'hudYellow';
    return 'hudGreen';
  }

  function getCurrentContext() {
    const app = root.TUHCApp;
    let pageId = 'HOMEPAGE';
    let theme = 'mspa';
    let mods = 'None';

    if (app && app.state) {
      const s = app.state;
      if (s.currentPageData && (s.currentPageData.page_id || s.currentPageData.pageId)) {
        pageId = s.currentPageData.page_id || s.currentPageData.pageId;
      } else if (s.currentRoute) {
        if (s.currentRoute.pageId) {
          pageId = s.currentRoute.pageId;
        } else if (s.currentRoute.mode) {
          pageId = s.currentRoute.mode;
        }
      }

      // Enabled mods
      const enabledList = s.config?.settings?.mod_list_enabled
        || s.config?.settings?.modListEnabled
        || s.config?.enabledMods;
      if (Array.isArray(enabledList) && enabledList.length > 0) {
        mods = enabledList.join(', ');
      }
    }

    if (typeof document !== 'undefined') {
      const pageContainer = document.getElementById('page-container');
      const appEl = document.getElementById('app');
      theme = pageContainer?.getAttribute('data-theme')
        || appEl?.getAttribute('data-theme')
        || app?.state?.config?.settings?.themeOverride
        || 'mspa';
    }

    return { pageId, theme, mods };
  }

  function updateDisplay() {
    if (!hudContainer || !isVisible) return;

    const ctx = getCurrentContext();
    const fpsClass = getFpsColorClass(currentFps);
    const rssClass = getRssColorClass(latestMetrics.rss_mb);
    const cpuClass = getCpuColorClass(latestMetrics.cpu_pct);

    if (isMinimized) {
      if (hudMiniContentEl) {
        hudMiniContentEl.textContent = `${currentFps} FPS (${currentFrameTime}ms) | ${latestMetrics.rss_mb} MB | CPU ${latestMetrics.cpu_pct}%`;
        hudMiniContentEl.className = fpsClass;
      }
      return;
    }

    // Render expanded details
    if (hudFpsEl) {
      hudFpsEl.textContent = `${currentFps} FPS (${currentFrameTime}ms)`;
      hudFpsEl.className = `hudVal ${fpsClass}`;
    }
    if (hudJankEl) {
      hudJankEl.textContent = String(jankCount);
      hudJankEl.className = `hudVal ${jankCount > 10 ? 'hudYellow' : ''}`;
    }
    if (hudCpuEl) {
      hudCpuEl.textContent = `${latestMetrics.cpu_pct.toFixed(1)}%`;
      hudCpuEl.className = `hudVal ${cpuClass}`;
    }
    if (hudRssEl) {
      hudRssEl.textContent = `${latestMetrics.rss_mb.toFixed(1)} MB`;
      hudRssEl.className = `hudVal ${rssClass}`;
    }
    if (hudThreadsEl) {
      hudThreadsEl.textContent = String(latestMetrics.threads);
    }
    if (hudUptimeEl) {
      hudUptimeEl.textContent = formatUptime(latestMetrics.uptime_secs);
    }
    if (hudPageEl) {
      hudPageEl.textContent = ctx.pageId;
    }
    if (hudThemeEl) {
      hudThemeEl.textContent = ctx.theme;
    }
    if (hudModsEl) {
      hudModsEl.textContent = ctx.mods;
      hudModsEl.title = ctx.mods;
    }
  }

  function onFrame(now) {
    if (!isVisible || isMinimized) {
      if (rafId) {
        cancelAnimationFrame(rafId);
        rafId = null;
      }
      return;
    }

    frameCount++;
    const delta = now - lastFrameTime;
    lastFrameTime = now;

    // Detect frame drop / lag spike (>33.3ms for ~30 FPS threshold)
    if (delta > 33.3) {
      jankCount++;
    }

    const elapsed = now - lastFpsUpdateTime;
    if (elapsed >= 500) {
      currentFps = Math.round((frameCount * 1000) / elapsed);
      currentFrameTime = parseFloat((elapsed / frameCount).toFixed(1));
      frameCount = 0;
      lastFpsUpdateTime = now;
      updateDisplay();
    }

    rafId = requestAnimationFrame(onFrame);
  }

  async function fetchBackendMetrics() {
    if (!isVisible) return;

    try {
      let metrics = null;
      if (root.TUHCApp && typeof root.TUHCApp.ipcInvoke === 'function') {
        metrics = await root.TUHCApp.ipcInvoke('get_debug_metrics');
      } else {
        const resp = await fetch('/ipc/get_debug_metrics');
        if (resp.ok) {
          metrics = await resp.json();
        }
      }

      if (metrics && typeof metrics.rss_mb === 'number') {
        latestMetrics = metrics;
        updateDisplay();
      }
    } catch (_err) {
      // Non-critical diagnostic fetch
    }
  }

  function startMonitoring() {
    lastFrameTime = performance.now();
    lastFpsUpdateTime = performance.now();
    frameCount = 0;

    if (!rafId && isVisible && !isMinimized) {
      rafId = requestAnimationFrame(onFrame);
    }
    if (!pollIntervalId) {
      fetchBackendMetrics();
      pollIntervalId = setInterval(fetchBackendMetrics, 1000);
    }
  }

  function stopMonitoring() {
    if (rafId) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
    if (pollIntervalId) {
      clearInterval(pollIntervalId);
      pollIntervalId = null;
    }
  }

  function setMinimized(minimized) {
    isMinimized = Boolean(minimized);
    if (!hudContainer) return;

    if (isMinimized) {
      if (rafId) {
        cancelAnimationFrame(rafId);
        rafId = null;
      }
      hudContainer.classList.add('minimized');
      if (hudBodyEl) hudBodyEl.style.display = 'none';
      if (hudMiniContentEl) hudMiniContentEl.parentElement.style.display = 'flex';
      if (hudMinBtn) hudMinBtn.textContent = '[+]';
    } else {
      hudContainer.classList.remove('minimized');
      if (hudBodyEl) hudBodyEl.style.display = 'flex';
      if (hudMiniContentEl) hudMiniContentEl.parentElement.style.display = 'none';
      if (hudMinBtn) hudMinBtn.textContent = '[-]';
      if (isVisible && !rafId) {
        lastFrameTime = performance.now();
        lastFpsUpdateTime = performance.now();
        frameCount = 0;
        rafId = requestAnimationFrame(onFrame);
      }
    }
    try {
      localStorage.setItem('tuhc_debug_hud_minimized', isMinimized ? 'true' : 'false');
    } catch (_e) {}
    updateDisplay();
  }

  function show() {
    if (!hudContainer) createHudElements();
    isVisible = true;
    hudContainer.style.display = 'block';
    startMonitoring();
    updateDisplay();
    try {
      localStorage.setItem('tuhc_debug_hud_visible', 'true');
    } catch (_e) {}
  }

  function hide() {
    isVisible = false;
    if (hudContainer) {
      hudContainer.style.display = 'none';
    }
    stopMonitoring();
    try {
      localStorage.setItem('tuhc_debug_hud_visible', 'false');
    } catch (_e) {}
  }

  function toggle() {
    if (isVisible) {
      hide();
    } else {
      show();
    }
  }

  function resetJank() {
    jankCount = 0;
    updateDisplay();
  }

  function createHudElements() {
    if (typeof document === 'undefined' || hudContainer) return;

    hudContainer = document.createElement('div');
    hudContainer.id = 'tuhc-debug-hud';
    hudContainer.className = 'tuhcDebugHud';
    hudContainer.style.display = 'none';

    hudContainer.innerHTML = `
      <div class="hudHeader">
        <span class="hudTitle">TUHC DIAGNOSTICS</span>
        <span class="hudActions">
          <button id="hud-min-btn" class="hudBtn" title="Minimize / Expand">[-]</button>
          <button id="hud-rst-btn" class="hudBtn" title="Reset Jank Counter">[R]</button>
          <button id="hud-close-btn" class="hudBtn" title="Close (F3 or Ctrl+Shift+D to reopen)">[×]</button>
        </span>
      </div>
      <div id="hud-body" class="hudBody">
        <div class="hudRow">
          <span class="hudLabel">RENDER:</span>
          <span id="hud-fps" class="hudVal hudGreen">60 FPS (16.6ms)</span>
          <span class="hudLabel" style="margin-left: 8px;">JANK:</span>
          <span id="hud-jank" class="hudVal">0</span>
        </div>
        <div class="hudRow">
          <span class="hudLabel">CPU:</span>
          <span id="hud-cpu" class="hudVal">0.0%</span>
          <span class="hudLabel" style="margin-left: 8px;">RSS:</span>
          <span id="hud-rss" class="hudVal hudGreen">0.0 MB</span>
          <span class="hudSub">/ 150MB</span>
        </div>
        <div class="hudRow">
          <span class="hudLabel">THREADS:</span>
          <span id="hud-threads" class="hudVal">1</span>
          <span class="hudLabel" style="margin-left: 8px;">UPTIME:</span>
          <span id="hud-uptime" class="hudVal">00:00</span>
        </div>
        <div class="hudRow">
          <span class="hudLabel">PAGE:</span>
          <span id="hud-page" class="hudVal">001901</span>
          <span class="hudLabel" style="margin-left: 8px;">THEME:</span>
          <span id="hud-theme" class="hudVal">mspa</span>
        </div>
        <div class="hudRow" style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 260px;">
          <span class="hudLabel">MODS:</span>
          <span id="hud-mods" class="hudVal" style="font-size: 10px;">None</span>
        </div>
      </div>
      <div id="hud-mini-container" class="hudMinimizedBar" style="display: none;">
        <span id="hud-mini-content" class="hudGreen">60 FPS | 0.0 MB</span>
      </div>
    `;

    document.body.appendChild(hudContainer);

    // Cache elements
    hudFpsEl = hudContainer.querySelector('#hud-fps');
    hudJankEl = hudContainer.querySelector('#hud-jank');
    hudCpuEl = hudContainer.querySelector('#hud-cpu');
    hudRssEl = hudContainer.querySelector('#hud-rss');
    hudThreadsEl = hudContainer.querySelector('#hud-threads');
    hudUptimeEl = hudContainer.querySelector('#hud-uptime');
    hudPageEl = hudContainer.querySelector('#hud-page');
    hudThemeEl = hudContainer.querySelector('#hud-theme');
    hudModsEl = hudContainer.querySelector('#hud-mods');
    hudBodyEl = hudContainer.querySelector('#hud-body');
    hudMiniContentEl = hudContainer.querySelector('#hud-mini-content');
    hudMinBtn = hudContainer.querySelector('#hud-min-btn');

    // Button Listeners
    hudMinBtn?.addEventListener('click', () => setMinimized(!isMinimized));
    hudContainer.querySelector('#hud-rst-btn')?.addEventListener('click', resetJank);
    hudContainer.querySelector('#hud-close-btn')?.addEventListener('click', hide);

    // Initial minimized state restore
    try {
      if (localStorage.getItem('tuhc_debug_hud_minimized') === 'true') {
        setMinimized(true);
      }
    } catch (_e) {}
  }

  function setupGlobalShortcuts() {
    if (typeof window === 'undefined') return;

    window.addEventListener('keydown', (e) => {
      // F3 or Ctrl+Shift+D toggles HUD
      if (e.key === 'F3' || (e.ctrlKey && e.shiftKey && (e.key === 'D' || e.key === 'd'))) {
        e.preventDefault();
        toggle();
      }
    });
  }

  function init() {
    if (typeof document === 'undefined') return;

    createHudElements();
    setupGlobalShortcuts();

    // Determine initial visibility:
    // 1. Explicit user preference in localStorage
    // 2. Or devMode in AppConfig settings
    let shouldShow = false;
    try {
      const stored = localStorage.getItem('tuhc_debug_hud_visible');
      if (stored !== null) {
        shouldShow = (stored === 'true');
      } else if (root.TUHCApp?.state?.config?.settings?.devMode) {
        shouldShow = true;
      }
    } catch (_e) {}

    if (shouldShow) {
      show();
    }
  }

  // Export public API
  const TUHCDebugHud = {
    init,
    show,
    hide,
    toggle,
    setMinimized,
    resetJank,
    updateDisplay,
    getMetrics: () => latestMetrics,
    getCurrentFps: () => currentFps,
    isVisible: () => isVisible,
    isMinimized: () => isMinimized
  };

  root.TUHCDebugHud = TUHCDebugHud;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = TUHCDebugHud;
  }

  // Auto-init on page ready
  if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', init);
    } else {
      init();
    }
  }
})(typeof window !== 'undefined' ? window : globalThis);
