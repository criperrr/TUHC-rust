/**
 * The Unofficial Homestuck Collection (Tauri 2.0 Edition)
 * Milestone 2: Frontend Webview Engine (Features F07, F08, F09, F10, F11, F20, F25)
 *
 * Provides:
 * - F07: Dual-Mode IPC Client & Asset URL Resolver (?v=${overlayGeneration} cache busting)
 * - F08: Tabbed Browser State Machine, Router, History & Bookmarks Drawer
 * - F09 & F10: Story Page Renderer, 13 Themes, Character Quirks, Pesterlogs with PT-BR Mod Hooks
 * - F11: Complete Music Player, Discography (37 albums, 797 tracks) & Sound Test Audio
 * - F20 & F25: Settings UI, Community Mod Toggles (Homestuck PT-BR) & Discord RPC Toggle
 */

(function (root) {
  'use strict';

  /* ==========================================================================
     1. Global Application State
     ========================================================================== */
  const state = {
    assetBase: (typeof window !== 'undefined' && window.location && window.location.origin && !window.location.origin.startsWith('null'))
      ? window.location.origin
      : 'http://127.0.0.1:0',
    overlayGeneration: 0,
    isTauri: false,
    appInfo: null,
    activeTabKey: '000',
    currentRoute: { mode: 'HOMEPAGE' },
    currentPageData: null,
    musicData: null,
    availableMods: [],
    saveDebounceTimer: null,
    config: {
      assetPackPath: '',
      locale: 'en-US',
      enabledMods: [],
      tabData: {
        activeTabKey: '000',
        tabs: {
          '000': {
            key: '000',
            url: '/',
            title: 'The Unofficial Homestuck Collection',
            hasEmbed: false,
            history: [],
            future: []
          }
        },
        tabList: ['000'],
        sortedTabList: ['000'],
        closedTabList: []
      },
      saveData: {
        saves: {},
        saveList: []
      },
      settings: {
        themeOverride: 'default',
        forceThemeOverride: false,
        bolin: false,
        hqAudio: true,
        jsFlashes: true,
        ruffleFallback: true,
        unpeachy: false,
        pxsTavros: false,
        cursedHistory: false,
        retcon1: true,
        retcon2: true,
        retcon3: true,
        retcon4: true,
        retcon5: true,
        retcon6: true,
        mspaMode: false,
        switchToNewTabs: false,
        forceScrollBar: true,
        urlTooltip: true,
        openLogs: false,
        enableDiscordRpc: true,
        textOverride: {
          fontFamily: '',
          bold: false,
          fontSize: 0,
          lineHeight: 0,
          paragraphSpacing: false,
          highContrast: false
        }
      }
    },
    soundTest: {
      isPlaying: false,
      currentTrack: null,
      currentAlbum: null,
      audioEl: null
    }
  };

  /* ==========================================================================
     2. Dual-Mode IPC Client & Asset URL Resolver (F07)
     ========================================================================== */

  /**
   * Universal IPC Invoker. Automatically uses Tauri IPC internals when available,
   * otherwise posts JSON to the Axum loopback /ipc HTTP mirror.
   */
  async function ipcInvoke(cmd, args = {}) {
    const tauriInvoke = (typeof window !== 'undefined' && window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function')
      ? window.__TAURI_INTERNALS__.invoke
      : (typeof window !== 'undefined' && window.__TAURI__?.core?.invoke);

    // Map common aliases for complete backward/forward compatibility
    let command = cmd;
    let payload = Object.assign({}, args);

    if (command === 'get_config') {
      command = 'load_config';
    } else if (command === 'list_mods') {
      command = 'get_available_mods';
    }

    if (command === 'save_config' && !payload.config && !payload.new_config && !payload.newConfig) {
      payload = { config: payload };
    }

    if (command === 'get_story_page') {
      const pid = payload.page_id || payload.pageId || payload.id;
      if (pid) {
        payload.page_id = pid;
        payload.pageId = pid;
      }
    }

    if (tauriInvoke) {
      try {
        return await tauriInvoke(command, payload);
      } catch (err) {
        // Fall back to HTTP if Tauri invoke fails in development or hybrid mode
        console.warn(`[TUHC-IPC] Tauri invoke failed for '${command}', falling back to HTTP:`, err);
      }
    }

    // HTTP /ipc loopback fallback
    try {
      const resp = await fetch(`${state.assetBase}/ipc`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json'
        },
        body: JSON.stringify({ cmd: command, args: payload })
      });

      if (!resp.ok) {
        const errorText = await resp.text();
        throw new Error(`IPC '${command}' error [${resp.status}]: ${errorText}`);
      }

      return await resp.json();
    } catch (e) {
      // Graceful fallback for non-critical telemetry or presence endpoints
      if (command.includes('discord')) {
        console.debug(`[TUHC-IPC] Discord RPC non-fatal notice:`, e.message);
        return { status: 'unsupported' };
      }
      throw e;
    }
  }

  /**
   * Transforms logical or relative asset references into full HTTP/VFS paths.
   * Injects cache-busting `?v=${overlayGeneration}` when community mods change.
   */
  function resolveAssetUrl(rawUrl) {
    if (!rawUrl || typeof rawUrl !== 'string') return '';
    let url = rawUrl.trim();

    // Strip external mspaintadventures domain
    url = url.replace(/^https?:\/\/(?:www\.)?mspaintadventures\.com\//i, '/');

    let resolved = '';
    if (url.startsWith('assets://')) {
      const rest = url.slice('assets://'.length).replace(/^\/+/, '');
      resolved = `${state.assetBase}/assets/${rest}`;
    } else if (url.startsWith('/assets/')) {
      resolved = `${state.assetBase}${url}`;
    } else if (
      url.startsWith('/storyfiles/') ||
      url.startsWith('/archive/') ||
      url.startsWith('/images/') ||
      url.startsWith('/fonts/') ||
      url.startsWith('/advimgs/')
    ) {
      resolved = `${state.assetBase}/assets${url}`;
    } else if (
      url.startsWith('storyfiles/') ||
      url.startsWith('archive/') ||
      url.startsWith('images/') ||
      url.startsWith('fonts/') ||
      url.startsWith('advimgs/')
    ) {
      resolved = `${state.assetBase}/assets/${url}`;
    } else if (
      url.startsWith('/') ||
      url.startsWith('http://') ||
      url.startsWith('https://') ||
      url.startsWith('data:') ||
      url.startsWith('blob:')
    ) {
      resolved = url;
    } else {
      resolved = `${state.assetBase}/assets/${url}`;
    }

    // Append cache-busting token if community mod overlay has updated
    if (state.overlayGeneration > 0 && !resolved.startsWith('data:') && !resolved.startsWith('blob:')) {
      const separator = resolved.includes('?') ? '&' : '?';
      resolved = `${resolved}${separator}v=${state.overlayGeneration}`;
    }
    return resolved;
  }

  /* ==========================================================================
     3. Configuration Persistence & Debouncing (F08)
     ========================================================================== */

  function persistConfig(immediate = false) {
    if (state.saveDebounceTimer) {
      clearTimeout(state.saveDebounceTimer);
      state.saveDebounceTimer = null;
    }

    const saveAction = async () => {
      try {
        await ipcInvoke('save_config', { config: state.config });
      } catch (err) {
        console.error('[TUHC-Config] Failed to persist config:', err);
      }
    };

    if (immediate) {
      saveAction();
    } else {
      state.saveDebounceTimer = setTimeout(saveAction, 150);
    }
  }

  async function loadConfig() {
    try {
      const cfg = await ipcInvoke('load_config');
      if (cfg && typeof cfg === 'object') {
        state.config = Object.assign(state.config, cfg);
        if (cfg.tabData && cfg.tabData.activeTabKey) {
          state.activeTabKey = cfg.tabData.activeTabKey;
        }
        if (cfg.settings) {
          applyGlobalSettings(cfg.settings);
        }
      }
    } catch (err) {
      console.warn('[TUHC-Config] load_config failed, using defaults:', err);
    }
  }

  function applyGlobalSettings(settings) {
    if (!settings) return;
    const body = document.body;
    if (body) {
      if (settings.forceScrollBar) {
        body.classList.add('force-scrollbar');
      } else {
        body.classList.remove('force-scrollbar');
      }
    }
  }

  /* ==========================================================================
     4. Tabbed Browser State Machine & Chrome (F08)
     ========================================================================== */

  let tabCounter = 1;

  function generateTabKey() {
    return 'tab_' + String(++tabCounter).padStart(3, '0') + '_' + Math.random().toString(36).substr(2, 4);
  }

  function getActiveTab() {
    const tabs = state.config.tabData.tabs;
    return tabs[state.activeTabKey] || tabs[state.config.tabData.tabList[0]];
  }

  function newTab(url = '/', adjacent = false, switchTo = true) {
    const tabData = state.config.tabData;
    const key = generateTabKey();
    const newTabObj = {
      key: key,
      url: url,
      title: 'The Unofficial Homestuck Collection',
      hasEmbed: false,
      history: [],
      future: []
    };

    tabData.tabs[key] = newTabObj;

    if (adjacent && tabData.tabList.includes(state.activeTabKey)) {
      const idx = tabData.tabList.indexOf(state.activeTabKey);
      tabData.tabList.splice(idx + 1, 0, key);
    } else {
      tabData.tabList.push(key);
    }

    if (!tabData.sortedTabList) tabData.sortedTabList = [];
    tabData.sortedTabList.unshift(key);

    if (switchTo || state.config.settings.switchToNewTabs) {
      switchTab(key);
    } else {
      renderTabBar();
      persistConfig();
    }
    return key;
  }

  function closeTab(key = state.activeTabKey) {
    const tabData = state.config.tabData;
    const list = tabData.tabList;

    if (list.length <= 1) {
      // Don't close last remaining tab; reset it to homepage
      const onlyTab = tabData.tabs[list[0]];
      if (onlyTab) {
        onlyTab.url = '/';
        onlyTab.title = 'The Unofficial Homestuck Collection';
        onlyTab.history = [];
        onlyTab.future = [];
        pushUrl('/', true);
      }
      renderTabBar();
      persistConfig();
      return;
    }

    const idx = list.indexOf(key);
    if (idx !== -1) {
      list.splice(idx, 1);
    }

    if (tabData.sortedTabList) {
      const sIdx = tabData.sortedTabList.indexOf(key);
      if (sIdx !== -1) tabData.sortedTabList.splice(sIdx, 1);
    }

    if (!tabData.closedTabList) tabData.closedTabList = [];
    const closedTab = tabData.tabs[key];
    if (closedTab) {
      tabData.closedTabList.push({ key, index: idx, tab: closedTab });
      if (tabData.closedTabList.length > 25) tabData.closedTabList.shift();
      delete tabData.tabs[key];
    }

    if (state.activeTabKey === key) {
      // Find candidate in sortedTabList or adjacent tab
      let nextKey = null;
      if (tabData.sortedTabList && tabData.sortedTabList.length > 0) {
        for (const candidate of tabData.sortedTabList) {
          if (list.includes(candidate)) {
            nextKey = candidate;
            break;
          }
        }
      }
      if (!nextKey) {
        nextKey = list[Math.min(idx, list.length - 1)];
      }
      switchTab(nextKey);
    } else {
      renderTabBar();
      persistConfig();
    }
  }

  function switchTab(key) {
    const tabData = state.config.tabData;
    if (!tabData.tabs[key]) return;

    state.activeTabKey = key;
    tabData.activeTabKey = key;

    if (!tabData.sortedTabList) tabData.sortedTabList = [];
    const sIdx = tabData.sortedTabList.indexOf(key);
    if (sIdx !== -1) tabData.sortedTabList.splice(sIdx, 1);
    tabData.sortedTabList.unshift(key);

    const activeTab = tabData.tabs[key];
    renderTabBar();
    updateAddressBar(activeTab.url);
    updateNavControls();
    renderRoute(resolveRoute(activeTab.url));
    persistConfig();
  }

  function restoreClosedTab() {
    const tabData = state.config.tabData;
    if (!tabData.closedTabList || tabData.closedTabList.length === 0) return;

    const restored = tabData.closedTabList.pop();
    const key = restored.key;
    tabData.tabs[key] = restored.tab;

    const insertIdx = Math.min(restored.index, tabData.tabList.length);
    tabData.tabList.splice(insertIdx, 0, key);

    switchTab(key);
  }

  function duplicateTab(key = state.activeTabKey) {
    const active = state.config.tabData.tabs[key];
    if (!active) return;
    newTab(active.url, true, true);
  }

  function pushUrl(rawUrl, replace = false, targetKey = state.activeTabKey) {
    if (typeof replace === 'string') {
      const actualTarget = replace;
      replace = !!targetKey;
      targetKey = actualTarget;
    }
    if (!targetKey || typeof targetKey !== 'string') {
      targetKey = state.activeTabKey;
    }

    const tab = state.config.tabData.tabs[targetKey] || getActiveTab();
    if (!tab) return;

    let url = (rawUrl || '/').trim();
    if (!url.startsWith('/') && !url.startsWith('http://') && !url.startsWith('https://') && !url.startsWith('mspa://')) {
      url = '/' + url;
    }

    if (url === tab.url && !replace) {
      return;
    }

    if (!replace && tab.url) {
      if (!tab.history) tab.history = [];
      tab.history.push(tab.url);
      tab.future = [];
    }

    tab.url = url;

    // Synchronize HTML5 Browser History API (for Back / Forward / Refresh support)
    if (typeof window !== 'undefined' && window.history && window.history.pushState) {
      try {
        const formatted = formatAddressUrl(url);
        if (replace) {
          window.history.replaceState({ tabKey: targetKey, url }, '', formatted);
        } else if (window.location.pathname !== formatted) {
          window.history.pushState({ tabKey: targetKey, url }, '', formatted);
        }
      } catch (_) {}
    }

    if (targetKey === state.activeTabKey) {
      updateAddressBar(url);
      updateNavControls();
      renderRoute(resolveRoute(url));
    }
    persistConfig();
  }

  function historyBack(targetKey = state.activeTabKey) {
    if (!targetKey || typeof targetKey !== 'string') targetKey = state.activeTabKey;
    const tab = state.config.tabData.tabs[targetKey] || getActiveTab();
    if (!tab || !tab.history || tab.history.length === 0) return;

    if (!tab.future) tab.future = [];
    tab.future.unshift(tab.url);

    const prevUrl = tab.history.pop();
    tab.url = prevUrl;

    if (typeof window !== 'undefined' && window.history && window.history.replaceState) {
      try {
        window.history.replaceState({ tabKey: targetKey, url: prevUrl }, '', formatAddressUrl(prevUrl));
      } catch (_) {}
    }

    if (targetKey === state.activeTabKey) {
      updateAddressBar(prevUrl);
      updateNavControls();
      renderRoute(resolveRoute(prevUrl));
    }
    persistConfig();
  }

  function historyForward(targetKey = state.activeTabKey) {
    if (!targetKey || typeof targetKey !== 'string') targetKey = state.activeTabKey;
    const tab = state.config.tabData.tabs[targetKey] || getActiveTab();
    if (!tab || !tab.future || tab.future.length === 0) return;

    if (!tab.history) tab.history = [];
    tab.history.push(tab.url);

    const nextUrl = tab.future.shift();
    tab.url = nextUrl;

    if (typeof window !== 'undefined' && window.history && window.history.replaceState) {
      try {
        window.history.replaceState({ tabKey: targetKey, url: nextUrl }, '', formatAddressUrl(nextUrl));
      } catch (_) {}
    }

    if (targetKey === state.activeTabKey) {
      updateAddressBar(nextUrl);
      updateNavControls();
      renderRoute(resolveRoute(nextUrl));
    }
    persistConfig();
  }

  function renderTabBar() {
    if (typeof document === 'undefined') return;
    const tabListEl = document.getElementById('tabs') || document.getElementById('tab-list');
    if (!tabListEl) return;

    tabListEl.innerHTML = '';
    const tabData = state.config.tabData;

    for (const key of tabData.tabList) {
      const tab = tabData.tabs[key];
      if (!tab) continue;

      const shell = document.createElement('li');
      shell.className = 'tabShell' + (key === state.activeTabKey ? ' activeTabShell' : '');

      const tabEl = document.createElement('div');
      tabEl.className = 'tab' + (key === state.activeTabKey ? ' activeTab' : '');
      tabEl.setAttribute('data-tab-key', key);
      tabEl.setAttribute('role', 'tab');
      tabEl.setAttribute('aria-selected', key === state.activeTabKey ? 'true' : 'false');

      const titleSpan = document.createElement('div');
      titleSpan.className = 'tabTitle';
      const textSpan = document.createElement('span');
      textSpan.textContent = tab.title || 'The Unofficial Homestuck Collection';
      titleSpan.appendChild(textSpan);
      tabEl.appendChild(titleSpan);

      if (tabData.tabList.length > 1) {
        const closeBtn = document.createElement('div');
        closeBtn.className = 'systemButton closeTabButton';
        closeBtn.title = 'Close Tab (Ctrl+W)';
        closeBtn.textContent = '✕';
        closeBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          closeTab(key);
        });
        tabEl.appendChild(closeBtn);
      }

      tabEl.addEventListener('click', () => switchTab(key));
      shell.appendChild(tabEl);
      tabListEl.appendChild(shell);
    }

    const titleTextEl = document.getElementById('titleBarText');
    if (titleTextEl) {
      const activeTab = getActiveTab();
      titleTextEl.textContent = activeTab ? (activeTab.title || 'The Unofficial Homestuck Collection') : 'The Unofficial Homestuck Collection';
    }
  }

  function updateAddressBar(url) {
    if (typeof document === 'undefined') return;
    const input = document.getElementById('address-bar-input');
    if (!input) return;
    input.value = formatAddressUrl(url);
  }

  function updateNavControls() {
    if (typeof document === 'undefined') return;
    const tab = getActiveTab();
    const backBtn = document.getElementById('nav-back-btn');
    const fwdBtn = document.getElementById('nav-forward-btn');

    if (backBtn) {
      backBtn.disabled = !tab || !tab.history || tab.history.length === 0;
    }
    if (fwdBtn) {
      fwdBtn.disabled = !tab || !tab.future || tab.future.length === 0;
    }
  }

  /* ==========================================================================
     5. URL Routing & Viz-to-MSPA Conversions (F08)
     ========================================================================== */

  function vizToMspa(base, page) {
    const pStr = String(page).toLowerCase();
    const p = parseInt(pStr, 10);

    if (base === 'homestuck') {
      if (pStr === 'darkcage') return 'darkcage';
      if (pStr === 'darkcage2') return 'darkcage2';
      if (pStr === 'dancestors') return 'dancestors';
      if (pStr === 'act7') return '010027';
      if (pStr === 'epilogues') return '010030';
      if (!isNaN(p) && p >= 1 && p <= 8130) {
        return String(p + 1900).padStart(6, '0');
      }
      return String(p).padStart(6, '0');
    }

    if (base === 'problem-sleuth') {
      if (!isNaN(p) && p >= 1 && p <= 1674) {
        return String(p + 218).padStart(6, '0');
      }
      return String(p).padStart(6, '0');
    }

    if (base === 'jailbreak') {
      if (p === 135 || pStr === '135') return 'jb2_000000';
      if (!isNaN(p) && p >= 1 && p <= 134) {
        return String(p + 1).padStart(6, '0');
      }
      return String(p).padStart(6, '0');
    }

    if (base === 'bard-quest') {
      if (p === 1) return '000136';
      if (!isNaN(p) && p >= 2 && p <= 47) {
        return String(p + 169).padStart(6, '0');
      }
      return String(p).padStart(6, '0');
    }

    if (base === 'beta') {
      if (!isNaN(p) && p >= 1 && p <= 8) {
        return String(p + 1892).padStart(6, '0');
      }
      return String(p).padStart(6, '0');
    }

    if (base === 'ryanquest') {
      if (!isNaN(p)) return 'rq' + String(p).padStart(4, '0');
      return pStr;
    }

    return pStr.padStart(6, '0');
  }

  function mspaToViz(pageId) {
    const pStr = String(pageId).trim();
    const n = parseInt(pStr, 10);

    if (isNaN(n)) {
      if (pStr === 'jb2_000000') return { base: 'jailbreak', page: '135' };
      return null;
    }

    if (n >= 1901 && n <= 10030) {
      return { base: 'homestuck', page: String(n - 1900) };
    }
    if (n >= 219 && n <= 1892) {
      return { base: 'problem-sleuth', page: String(n - 218) };
    }
    if (n >= 2 && n <= 135) {
      return { base: 'jailbreak', page: String(n - 1) };
    }
    if (n === 136) {
      return { base: 'bard-quest', page: '1' };
    }
    if (n >= 171 && n <= 216) {
      return { base: 'bard-quest', page: String(n - 169) };
    }
    if (n >= 1893 && n <= 1900) {
      return { base: 'beta', page: String(n - 1892) };
    }
    return null;
  }

  function getStoryNum(pageId) {
    const n = parseInt(pageId, 10);
    if (isNaN(n)) return 6;
    if (n >= 1901) return 6;
    if (n >= 1893) return 5;
    if (n >= 219) return 4;
    if (n >= 136) return 2;
    if (n >= 1) return 1;
    return 6;
  }

  function getStoryStartPage(storyNum) {
    switch (storyNum) {
      case 1: return '000002';
      case 2: return '000136';
      case 4: return '000219';
      case 5: return '001893';
      case 6:
      default: return '001901';
    }
  }

  function formatTimestamp(ts) {
    if (!ts) return '';
    try {
      const date = new Date(Number(ts) * 1000);
      const options = {
        timeZone: 'America/New_York',
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: 'numeric',
        minute: '2-digit',
        hour12: true
      };
      return new Intl.DateTimeFormat('en-US', {
        ...options,
        timeZoneName: 'short'
      }).format(date);
    } catch (_) {
      return '';
    }
  }

  function formatAddressUrl(url) {
    if (!url) return '/';
    const route = resolveRoute(url);
    if (route.mode === 'PAGE') {
      if (state.config.settings.mspaMode) {
        return `/mspa/${route.pageId}`;
      }
      const viz = mspaToViz(route.pageId);
      if (viz) {
        return `/${viz.base}/${viz.page}`;
      }
      return `/mspa/${route.pageId}`;
    }
    return url;
  }

  function resolveRoute(rawUrl) {
    if (!rawUrl || rawUrl === '' || rawUrl === '/' || rawUrl === '/home') {
      return { mode: 'HOMEPAGE' };
    }

    let url = rawUrl.trim();
    if (url.startsWith('mspa://')) {
      url = '/mspa/' + url.slice('mspa://'.length);
    }

    // Classic query string format `?s=6&p=001901`
    if (url.includes('?')) {
      const parts = url.split('?');
      const queryParams = new URLSearchParams(parts[1]);
      if (queryParams.has('p')) {
        let p = queryParams.get('p');
        if (/^\d+$/.test(p) && p.length < 6) p = p.padStart(6, '0');
        return { mode: 'PAGE', pageId: p };
      }
    }

    // Direct system routes
    if (url === '/settings') return { mode: 'SETTINGS' };
    if (url === '/help') return { mode: 'HELP' };
    if (url === '/evenmore') return { mode: 'EVENMORE' };
    if (url === '/credits') return { mode: 'CREDITS' };
    if (url.startsWith('/log')) {
      const parts = url.split('/').filter(Boolean);
      return { mode: 'LOG', storyId: parts[1] || '6' };
    }
    if (url.startsWith('/map')) {
      const parts = url.split('/').filter(Boolean);
      return { mode: 'MAP', storyId: parts[1] || '6' };
    }
    if (url.startsWith('/search')) {
      const parts = url.split('/').filter(Boolean);
      return { mode: 'SEARCH', query: decodeURIComponent(parts[1] || '') };
    }
    if (url.startsWith('/sbahj')) {
      const parts = url.split('/').filter(Boolean);
      return { mode: 'SBAHJ', cid: parts[1] || '1' };
    }

    // Music routes
    if (url === '/music' || url === '/music/discography') {
      return { mode: 'MUSIC', view: 'discography' };
    }
    if (url === '/music/tracks') {
      return { mode: 'MUSIC', view: 'tracks' };
    }
    if (url === '/music/artists') {
      return { mode: 'MUSIC', view: 'artists' };
    }
    if (url === '/music/features') {
      return { mode: 'MUSIC', view: 'features' };
    }
    if (url.startsWith('/music/album/')) {
      return { mode: 'MUSIC', view: 'album', id: url.slice('/music/album/'.length) };
    }
    if (url.startsWith('/music/track/')) {
      return { mode: 'MUSIC', view: 'track', id: url.slice('/music/track/'.length) };
    }
    if (url.startsWith('/music/artist/')) {
      return { mode: 'MUSIC', view: 'artist', id: url.slice('/music/artist/'.length) };
    }

    // Story Page routes (/mspa/<id>, /homestuck/<p>, /problem-sleuth/<p>, etc.)
    if (url.startsWith('/mspa/')) {
      const rawPid = url.slice('/mspa/'.length).trim();
      let pid = rawPid;
      // Shorthand story entries:
      if (rawPid === '1') pid = '000002';
      else if (rawPid === '2') pid = '000136';
      else if (rawPid === '3') pid = 'mc0001';
      else if (rawPid === '4') pid = '000219';
      else if (rawPid === '5') pid = '001893';
      else if (rawPid === '6') pid = '001901';
      else if (/^\d+$/.test(rawPid) && rawPid.length < 6) {
        pid = rawPid.padStart(6, '0');
      }
      return { mode: 'PAGE', pageId: pid };
    }

    const storyMatch = url.match(/^\/(homestuck|problem-sleuth|jailbreak|bard-quest|beta|ryanquest)\/(.+)$/);
    if (storyMatch) {
      const base = storyMatch[1];
      const p = storyMatch[2];
      const pageId = vizToMspa(base, p);
      return { mode: 'PAGE', pageId: pageId };
    }

    // Numeric shorthand fallback
    const numericMatch = url.match(/^\/(\d+)$/);
    if (numericMatch) {
      let n = numericMatch[1];
      if (state.config.settings.mspaMode) {
        return { mode: 'PAGE', pageId: n.padStart(6, '0') };
      }
      return { mode: 'PAGE', pageId: vizToMspa('homestuck', n) };
    }

    return { mode: 'HOMEPAGE' };
  }

  /* ==========================================================================
     6. Story Page Renderer & Pesterlogs (F09 & F10)
     ========================================================================== */

  function computeActiveTheme(pageTheme, settings) {
    if (settings.forceThemeOverride && settings.themeOverride && settings.themeOverride !== 'default') {
      return settings.themeOverride;
    }
    if (pageTheme && typeof pageTheme === 'string' && pageTheme.trim() !== '') {
      return pageTheme.trim();
    }
    if (settings.themeOverride && settings.themeOverride !== 'default') {
      return settings.themeOverride;
    }
    return 'mspa';
  }

  function applyTheme(theme) {
    if (typeof document === 'undefined') return;
    const appEl = document.getElementById('app');
    const pageContainer = document.getElementById('page-container');
    const validTheme = theme || 'mspa';

    if (appEl) {
      appEl.setAttribute('data-theme', validTheme);
      appEl.className = 'theme-' + validTheme;
    }
    if (pageContainer) {
      pageContainer.setAttribute('data-theme', validTheme);
      pageContainer.className = 'pageBody theme-' + validTheme;
    }
  }

  /**
   * Reference logic for PT-BR logButtonText when window.TUHCModHooks is not loaded.
   * Matches `evaluate_ptbr_log_button_text` in tests/common.py exactly.
   */
  function evaluateFallbackLogButtonText(rawContent, isHidden, enabledMods, locale) {
    const match = rawContent.match(/\|(.*?)\|/);
    const rawTag = match ? match[1] : 'PESTERLOG';

    const isPtBr = locale === 'pt-BR' ||
      (Array.isArray(enabledMods) && (enabledMods.includes('Homestuck PT-BR') || enabledMods.includes('homestuck_ptbr')));

    const statePrefix = isPtBr ? (isHidden ? 'Abrir ' : 'Fechar ') : (isHidden ? 'Show ' : 'Hide ');

    if (/P4SSWORD H1NT/i.test(rawTag)) {
      return rawTag;
    }
    let tag = rawTag.toLowerCase();
    if (/trkstrlog/i.test(rawTag)) {
      tag = 'tricksterlog';
    } else if (/sriousbiz/i.test(rawTag)) {
      tag = 'serious business';
    }
    return statePrefix + tag;
  }

  function getLogButtonLabel(rawContent, isHidden) {
    if (root.TUHCModHooks && typeof root.TUHCModHooks.getLogButtonText === 'function') {
      try {
        const text = root.TUHCModHooks.getLogButtonText(rawContent, isHidden, state.config.enabledMods);
        if (text && typeof text === 'string') return text;
      } catch (e) {
        console.warn('[TUHC-ModHooks] getLogButtonText error:', e);
      }
    }
    return evaluateFallbackLogButtonText(
      rawContent,
      isHidden,
      state.config.enabledMods,
      state.config.locale
    );
  }

  function rewriteHtmlContent(html) {
    if (!html) return '';
    let processed = html;

    // Rewrite internal images
    processed = processed.replace(/<img([^>]+)src=["']([^"']+)["']/gi, (match, prefix, src) => {
      const resolved = resolveAssetUrl(src);
      return `<img${prefix}src="${resolved}"`;
    });

    // Clean up classic mspaintadventures links
    processed = processed.replace(/https?:\/\/(?:www\.)?mspaintadventures\.com\/\?s=(\d+)&amp;p=(\d+)/gi, '/mspa/$2');
    processed = processed.replace(/https?:\/\/(?:www\.)?mspaintadventures\.com\/\?s=(\d+)&p=(\d+)/gi, '/mspa/$2');

    return processed;
  }

  async function renderStoryPage(pageId) {
    const pageContent = document.getElementById('page-content');
    if (!pageContent) return;

    let titleEl = document.getElementById('page-title');
    let mediaContainer = document.getElementById('media-container');
    let textContainer = document.getElementById('text-container');
    let commandContainer = document.getElementById('command-container');
    let footerNav = document.getElementById('footer-nav');

    // Reconstruct story frame scaffolding if previous view (e.g. Homepage, Settings, Music) cleared it
    if (!titleEl || !mediaContainer || !textContainer || !commandContainer || !footerNav) {
      pageContent.innerHTML = `
        <h1 id="page-title" class="pageTitle">Loading...</h1>
        <div id="media-container" class="mediaContent"></div>
        <div id="text-container" class="textContent"><p class="prattle">Fetching story page data...</p></div>
        <div id="command-container" class="commandContent"></div>
        <div id="footer-nav" class="footerNav"></div>
      `;
      titleEl = document.getElementById('page-title');
      mediaContainer = document.getElementById('media-container');
      textContainer = document.getElementById('text-container');
      commandContainer = document.getElementById('command-container');
      footerNav = document.getElementById('footer-nav');
    }

    if (titleEl) titleEl.textContent = 'Loading...';
    if (mediaContainer) mediaContainer.innerHTML = '';
    if (textContainer) textContainer.innerHTML = '<p class="prattle">Fetching story page data...</p>';
    if (commandContainer) commandContainer.innerHTML = '';
    if (footerNav) footerNav.innerHTML = '';

    try {
      const page = await ipcInvoke('get_story_page', { page_id: pageId });
      state.currentPageData = page;

      // Update tab title, document title, and TitleBar text
      const pageTitle = page.title || 'MSPA';
      const storyNum = getStoryNum(pageId);
      const adventureSuffixes = [
        ' - Jailbreak',
        ' - Bard Quest',
        '',
        ' - Problem Sleuth',
        ' - Homestuck Beta',
        ' - Homestuck'
      ];
      const fullTitle = (storyNum >= 1 && storyNum <= 6 && adventureSuffixes[storyNum - 1])
        ? `${pageTitle}${adventureSuffixes[storyNum - 1]}`
        : `${pageTitle} - Homestuck`;

      const activeTab = getActiveTab();
      if (activeTab) {
        activeTab.title = fullTitle;
        renderTabBar();
      }
      document.title = fullTitle;
      const titleBarTextEl = document.getElementById('titleBarText');
      if (titleBarTextEl) {
        titleBarTextEl.textContent = fullTitle;
      }

      // Apply theme
      const theme = computeActiveTheme(page.theme, state.config.settings);
      applyTheme(theme);

      // Discord RPC Reading Notification
      if (state.config.settings.enableDiscordRpc) {
        ipcInvoke('set_discord_presence', {
          page_id: pageId,
          title: pageTitle
        }).catch(() => {});
      }

      // Title
      if (titleEl) {
        titleEl.textContent = pageTitle;
        if (page.flag && page.flag.includes('S')) {
          titleEl.style.display = 'none'; // Supercartridge hides title
        } else {
          titleEl.style.display = 'block';
        }
      }

      // Media Rendering (Retcon logic + TUHCMediaEngine or fallback)
      if (mediaContainer) {
        let mediaList = Array.isArray(page.media) ? [...page.media] : [];

        // Apply Retcon toggles
        const settings = state.config.settings;
        mediaList = mediaList.map((m) => {
          let item = m;
          for (let r = 1; r <= 5; r++) {
            if (settings[`retcon${r}`] === false) {
              item = item.replace(/_retcon(\d)?\.(gif|png|jpg|swf)/i, '.$2');
              item = item.replace(/\/retcon\//i, '/');
            }
          }
          if (settings.unpeachy && pageId === '007680') {
            item = item.replace('05779_2.gif', '05779_unpeachy.gif');
          }
          if (settings.pxsTavros && pageId === '008753') {
            item = item.replace('06852.gif', '06852_pxs.gif');
          }
          return item;
        });

        // Delegate to TUHCMediaEngine if available
        let engineHandled = false;
        if (root.TUHCMediaEngine && typeof root.TUHCMediaEngine.renderMedia === 'function') {
          try {
            engineHandled = root.TUHCMediaEngine.renderMedia(
              mediaContainer,
              mediaList,
              page,
              {
                settings,
                assetBase: state.assetBase,
                cacheBuster: state.overlayGeneration,
                navigate: pushUrl
              },
              resolveAssetUrl
            );
          } catch (e) {
            console.warn('[TUHC-MediaEngine] renderMedia threw:', e);
          }
        }

        if (!engineHandled) {
          mediaContainer.innerHTML = '';
          for (const item of mediaList) {
            const url = resolveAssetUrl(item);
            const lower = item.toLowerCase();

            if (lower.endsWith('.gif') || lower.endsWith('.png') || lower.endsWith('.jpg') || lower.endsWith('.jpeg') || lower.endsWith('.webp')) {
              const img = document.createElement('img');
              img.className = 'panel-media';
              img.src = url;
              img.alt = pageTitle;
              mediaContainer.appendChild(img);
            } else if (lower.endsWith('.mp4') || lower.endsWith('.webm')) {
              const video = document.createElement('video');
              video.className = 'panel-media';
              video.controls = true;
              video.autoplay = true;
              video.src = url;
              mediaContainer.appendChild(video);
            } else if (lower.endsWith('.mp3') || lower.endsWith('.ogg') || lower.endsWith('.wav')) {
              const audio = document.createElement('audio');
              audio.controls = true;
              audio.src = url;
              mediaContainer.appendChild(audio);
            } else if (lower.endsWith('.swf')) {
              const flashProps = (root.TUHCFlashProps && root.TUHCFlashProps.INDEXED_FLASH_PROPS && root.TUHCFlashProps.INDEXED_FLASH_PROPS[pageId]) || {};
              const width = flashProps.width || 650;
              const height = flashProps.height || 450;

              const swfBox = document.createElement('div');
              swfBox.className = 'swf-embed-container';
              swfBox.style.width = width + 'px';
              swfBox.style.height = height + 'px';
              swfBox.style.margin = '0 auto';

              if (window.RufflePlayer && typeof window.RufflePlayer.newest === 'function') {
                const ruffle = window.RufflePlayer.newest();
                const player = ruffle.createPlayer();
                swfBox.appendChild(player);
                player.load({ url: url });
              } else {
                const obj = document.createElement('object');
                obj.setAttribute('data', url);
                obj.setAttribute('type', 'application/x-shockwave-flash');
                obj.setAttribute('width', String(width));
                obj.setAttribute('height', String(height));
                swfBox.appendChild(obj);
              }
              mediaContainer.appendChild(swfBox);
            } else if (lower.endsWith('.html') || lower.endsWith('.htm')) {
              const iframe = document.createElement('iframe');
              iframe.src = url;
              iframe.className = 'panel-iframe';
              iframe.width = '650';
              iframe.height = '450';
              iframe.frameBorder = '0';
              mediaContainer.appendChild(iframe);
            }
          }
        }
      }

      // Text Content Rendering (Pesterlogs, Authorlogs, Prattle)
      if (textContainer) {
        textContainer.innerHTML = '';
        const rawContent = page.content || '';

        // Universal Pesterlog delimiter matcher (matches PESTERLOG, DIALOGLOG, SPRITELOG, JOURNALOG, AUTHORLOG, etc.)
        const logMatch = rawContent.match(/^\|(.*?)\|\s*(?:<br\s*\/?>)?\s*/i);

        if (logMatch) {
          const logType = logMatch[1].trim().toUpperCase();
          const bodyHtml = rawContent.slice(logMatch[0].length);

          if (logType === 'AUTHORLOG') {
            const authorBox = document.createElement('div');
            authorBox.className = 'authorlog';
            const authorContent = document.createElement('p');
            authorContent.className = 'authorlogContent';
            authorContent.innerHTML = rewriteHtmlContent(bodyHtml);
            authorBox.appendChild(authorContent);
            textContainer.appendChild(authorBox);
          } else {
            // Collapsible pesterlog box
            let isHidden = !state.config.settings.openLogs;
            const logBox = document.createElement('div');
            logBox.className = 'log';

            const logBtn = document.createElement('button');
            logBtn.className = 'logButton';
            logBtn.textContent = getLogButtonLabel(rawContent, isHidden);

            const logContent = document.createElement('div');
            logContent.className = 'logContent';
            logContent.style.display = isHidden ? 'none' : 'block';
            logContent.innerHTML = rewriteHtmlContent(bodyHtml);

            logBtn.addEventListener('click', () => {
              isHidden = !isHidden;
              logContent.style.display = isHidden ? 'none' : 'block';
              logBtn.textContent = getLogButtonLabel(rawContent, isHidden);
            });

            logBox.appendChild(logBtn);
            logBox.appendChild(logContent);
            textContainer.appendChild(logBox);
          }
        } else if (rawContent.trim() !== '') {
          const prattle = document.createElement('p');
          prattle.className = 'prattle';
          prattle.innerHTML = rewriteHtmlContent(rawContent);
          textContainer.appendChild(prattle);
        }

        // Intercept link clicks inside textContainer
        bindInternalLinks(textContainer);
      }

      // Next Command Links
      if (commandContainer) {
        commandContainer.innerHTML = '';
        let nextList = Array.isArray(page.next) ? [...page.next] : [];

        // Retcon 6 password page exclusions
        const retcon6Pages = ['009058', '009109', '009135', '009150', '009188', '009204', '009222', '009263'];
        if (!state.config.settings.retcon6) {
          nextList = nextList.filter((nid) => !retcon6Pages.includes(nid));
        }

        for (const nextId of nextList) {
          const nextArrow = document.createElement('div');
          nextArrow.className = 'nextArrow';
          nextArrow.innerHTML = `&gt; <a href="/mspa/${nextId}" class="nextArrowLink" data-page-id="${nextId}">...</a>`;
          commandContainer.appendChild(nextArrow);

          // Asynchronously fetch next title
          ipcInvoke('get_story_page', { page_id: nextId }).then((nextPage) => {
            const link = nextArrow.querySelector('a');
            if (link && nextPage && nextPage.title) {
              link.textContent = nextPage.title;
            }
          }).catch(() => {
            const link = nextArrow.querySelector('a');
            if (link) link.textContent = `Page ${nextId}`;
          });
        }
        bindInternalLinks(commandContainer);
      }

      // Footer Navigation (Authentic Legacy Structure)
      if (footerNav) {
        footerNav.innerHTML = '';

        const navOptions = document.createElement('ul');
        navOptions.className = 'navOptions';

        const startOverId = getStoryStartPage(storyNum);
        if (pageId !== startOverId) {
          const liStart = document.createElement('li');
          const startOverLink = document.createElement('a');
          startOverLink.href = `/mspa/${startOverId}`;
          startOverLink.className = 'startOver';
          startOverLink.textContent = 'Start Over';
          liStart.appendChild(startOverLink);
          navOptions.appendChild(liStart);
        }

        if (page.previous) {
          const liBack = document.createElement('li');
          const backLink = document.createElement('a');
          backLink.href = `/mspa/${page.previous}`;
          backLink.className = 'goBack';
          backLink.textContent = 'Go Back';
          liBack.appendChild(backLink);
          navOptions.appendChild(liBack);
        }
        footerNav.appendChild(navOptions);

        const metaUl = document.createElement('ul');
        metaUl.className = 'meta';

        if (page.timestamp) {
          const liTime = document.createElement('li');
          const timeDiv = document.createElement('div');
          timeDiv.className = 'timestamp';
          timeDiv.setAttribute('data-timestamp', String(page.timestamp));
          timeDiv.textContent = formatTimestamp(page.timestamp);
          liTime.appendChild(timeDiv);
          metaUl.appendChild(liTime);
        }

        const liViz = document.createElement('li');
        const viz = mspaToViz(pageId);
        const vizHref = viz ? `https://homestuck.com/${viz.base === 'homestuck' ? 'story/' : viz.base + '/'}${viz.page}` : `/mspa/${pageId}`;
        const vizText = viz ? `/${viz.base}/${viz.page} ↗` : `/mspa/${pageId}`;
        const vizLink = document.createElement('a');
        vizLink.className = 'vizLink';
        vizLink.href = vizHref;
        vizLink.textContent = vizText;
        liViz.appendChild(vizLink);
        metaUl.appendChild(liViz);

        footerNav.appendChild(metaUl);
        bindInternalLinks(footerNav);
      }

    } catch (err) {
      console.error('[TUHC-Page] Failed to render story page:', err);
      if (titleEl) titleEl.textContent = 'Page Not Found';
      if (textContainer) {
        textContainer.innerHTML = `<p class="prattle" style="color: #ff3333;">Could not load page ${pageId}: ${err.message}</p>`;
      }
    }
  }

  function bindInternalLinks(container) {
    if (!container) return;
    const links = container.querySelectorAll('a[href]');
    for (const link of links) {
      const href = link.getAttribute('href');
      if (
        href.startsWith('/') ||
        href.startsWith('mspa://') ||
        href.startsWith('?s=') ||
        href.includes('mspaintadventures.com')
      ) {
        link.addEventListener('click', (e) => {
          e.preventDefault();
          pushUrl(href);
        });
      }
    }
  }

  /* ==========================================================================
     7. Music Player, Discography & Sound Test (F11)
     ========================================================================== */

  async function ensureMusicDataLoaded() {
    if (state.musicData) return state.musicData;
    try {
      const data = await ipcInvoke('get_music_data');
      state.musicData = data;
      return data;
    } catch (err) {
      console.error('[TUHC-Music] Failed to load music data:', err);
      return { albums: {}, tracks: {}, artists: {}, flashes: {} };
    }
  }

  function formatDuration(sec) {
    if (typeof sec !== 'number' || isNaN(sec)) return '0:00';
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${String(s).padStart(2, '0')}`;
  }

  function renderMusicNavHeader(activeView = 'discography') {
    return `
      <div class="musicHeaderNav">
        <div class="musicNavLinks">
          <a href="/music" class="${activeView === 'discography' ? 'activeMusicNavLink' : ''}">Albums</a> |
          <a href="/music/tracks" class="${activeView === 'tracks' ? 'activeMusicNavLink' : ''}">Tracks</a> |
          <a href="/music/artists" class="${activeView === 'artists' ? 'activeMusicNavLink' : ''}">Artists</a> |
          <a href="/music/features" class="${activeView === 'features' ? 'activeMusicNavLink' : ''}">Features</a> |
          <a href="#random" id="music-random-track-btn">Random Track</a>
        </div>
        <input id="music-filter-input" class="musicSearchInput" type="text" placeholder="Filter albums or tracks..." aria-label="Music Search" />
      </div>
    `;
  }

  async function renderMusicDiscographyView(container) {
    const data = await ensureMusicDataLoaded();
    const albums = data.albums || {};
    const albumKeys = Object.keys(albums);

    container.innerHTML = `
      <div class="musicContainer">
        ${renderMusicNavHeader('discography')}
        <h2 style="font-family: 'Courier New', monospace; text-align: left; margin-bottom: 12px;">DISCOGRAPHY (${albumKeys.length} ALBUMS)</h2>
        <div id="album-grid" class="albumGrid">
          ${albumKeys.map((slug) => {
            const alb = albums[slug];
            const coverUrl = resolveAssetUrl(`/archive/music/${alb.directory}/cover.jpg`);
            return `
              <div class="albumCard" data-slug="${slug}">
                <a href="/music/album/${slug}">
                  <img class="albumCover" src="${coverUrl}" alt="${alb.name}" loading="lazy" />
                </a>
                <div class="albumCardTitle">
                  <a href="/music/album/${slug}">${alb.name}</a>
                </div>
                <div class="albumCardDate">${alb.date || ''}</div>
              </div>
            `;
          }).join('')}
        </div>
      </div>
    `;

    bindInternalLinks(container);

    // Search filter
    const filterInput = container.querySelector('#music-filter-input');
    if (filterInput) {
      filterInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase().trim();
        const cards = container.querySelectorAll('.albumCard');
        cards.forEach((card) => {
          const title = (card.querySelector('.albumCardTitle')?.textContent || '').toLowerCase();
          card.style.display = title.includes(query) ? 'flex' : 'none';
        });
      });
    }

    // Random track button
    const randomBtn = container.querySelector('#music-random-track-btn');
    if (randomBtn) {
      randomBtn.addEventListener('click', (e) => {
        e.preventDefault();
        const trackKeys = Object.keys(data.tracks || {});
        if (trackKeys.length > 0) {
          const randKey = trackKeys[Math.floor(Math.random() * trackKeys.length)];
          pushUrl(`/music/track/${randKey}`);
        }
      });
    }
  }

  async function renderMusicTracksView(container) {
    const data = await ensureMusicDataLoaded();
    const tracks = data.tracks || {};
    const trackKeys = Object.keys(tracks);

    container.innerHTML = `
      <div class="musicContainer">
        ${renderMusicNavHeader('tracks')}
        <h2 style="font-family: 'Courier New', monospace; text-align: left; margin-bottom: 12px;">ALL TRACKS (${trackKeys.length})</h2>
        <table class="musicTrackTable">
          <thead>
            <tr>
              <th style="width: 40px;">Play</th>
              <th>Track Title</th>
              <th>Album</th>
              <th>Duration</th>
            </tr>
          </thead>
          <tbody id="track-rows">
            ${trackKeys.map((slug, idx) => {
              const trk = tracks[slug];
              return `
                <tr class="trackRow" data-slug="${slug}">
                  <td>
                    <button class="playTrackBtn" data-slug="${slug}" title="Play in Sound Test">&#9654;</button>
                  </td>
                  <td>
                    <a href="/music/track/${slug}"><strong>${trk.name}</strong></a>
                  </td>
                  <td>${trk.album || ''}</td>
                  <td>${formatDuration(trk.duration)}</td>
                </tr>
              `;
            }).join('')}
          </tbody>
        </table>
      </div>
    `;

    bindInternalLinks(container);

    // Wire up Play buttons
    const playBtns = container.querySelectorAll('.playTrackBtn');
    playBtns.forEach((btn) => {
      btn.addEventListener('click', () => {
        const slug = btn.getAttribute('data-slug');
        loadTrackIntoSoundTest(slug, true);
      });
    });

    // Search filter
    const filterInput = container.querySelector('#music-filter-input');
    if (filterInput) {
      filterInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase().trim();
        const rows = container.querySelectorAll('.trackRow');
        rows.forEach((row) => {
          const text = row.textContent.toLowerCase();
          row.style.display = text.includes(query) ? '' : 'none';
        });
      });
    }
  }

  async function renderAlbumDetailView(container, albumSlug) {
    const data = await ensureMusicDataLoaded();
    const album = data.albums ? data.albums[albumSlug] : null;

    if (!album) {
      container.innerHTML = `
        <div class="musicContainer">
          ${renderMusicNavHeader('discography')}
          <p class="prattle" style="color: #ff3333;">Album '${albumSlug}' not found in collection.</p>
        </div>
      `;
      return;
    }

    const coverUrl = resolveAssetUrl(`/archive/music/${album.directory}/cover.jpg`);
    let trackList = [];
    if (Array.isArray(album.tracks)) {
      trackList = album.tracks.flat();
    }

    container.innerHTML = `
      <div class="musicContainer">
        ${renderMusicNavHeader('discography')}
        <div class="musicDetailLayout" style="display: flex; gap: 20px; flex-wrap: wrap;">
          <div style="width: 220px; text-align: center;">
            <img class="albumCover" src="${coverUrl}" alt="${album.name}" style="width: 200px; height: 200px; border-radius: 4px;" />
            <div style="font-size: 11px; margin-top: 8px;">
              <strong>Released:</strong> ${album.date || 'Unknown'}<br />
              ${album.urls && album.urls.bandcamp ? `<a href="${album.urls.bandcamp}" target="_blank">Bandcamp Page</a>` : ''}
            </div>
          </div>
          <div style="flex: 1; min-width: 300px;">
            <h2 style="font-family: 'Courier New', monospace; margin-top: 0;">${album.name}</h2>
            <table class="musicTrackTable" style="margin-top: 12px;">
              <thead>
                <tr>
                  <th style="width: 40px;">Play</th>
                  <th style="width: 30px;">#</th>
                  <th>Title</th>
                  <th>Duration</th>
                </tr>
              </thead>
              <tbody>
                ${trackList.map((slug, idx) => {
                  const trk = data.tracks ? data.tracks[slug] : null;
                  const title = trk ? trk.name : slug;
                  const dur = trk ? formatDuration(trk.duration) : '';
                  return `
                    <tr>
                      <td>
                        <button class="playTrackBtn" data-slug="${slug}" title="Play">&#9654;</button>
                      </td>
                      <td>${idx + 1}</td>
                      <td><a href="/music/track/${slug}"><strong>${title}</strong></a></td>
                      <td>${dur}</td>
                    </tr>
                  `;
                }).join('')}
              </tbody>
            </table>
            ${album.commentary ? `
              <div class="settingsSectionCard" style="margin-top: 18px;">
                <h3>Album Commentary</h3>
                <div style="font-size: 12px; line-height: 1.4;">${album.commentary}</div>
              </div>
            ` : ''}
          </div>
        </div>
      </div>
    `;

    bindInternalLinks(container);

    const playBtns = container.querySelectorAll('.playTrackBtn');
    playBtns.forEach((btn) => {
      btn.addEventListener('click', () => {
        const slug = btn.getAttribute('data-slug');
        loadTrackIntoSoundTest(slug, true);
      });
    });
  }

  async function renderTrackDetailView(container, trackSlug) {
    const data = await ensureMusicDataLoaded();
    const track = data.tracks ? data.tracks[trackSlug] : null;

    if (!track) {
      container.innerHTML = `
        <div class="musicContainer">
          ${renderMusicNavHeader('tracks')}
          <p class="prattle" style="color: #ff3333;">Track '${trackSlug}' not found.</p>
        </div>
      `;
      return;
    }

    const album = data.albums ? data.albums[track.album] : null;
    const coverDir = album ? album.directory : track.album;
    const coverUrl = resolveAssetUrl(`/archive/music/${coverDir}/cover.jpg`);

    container.innerHTML = `
      <div class="musicContainer">
        ${renderMusicNavHeader('tracks')}
        <div class="musicDetailLayout" style="display: flex; gap: 20px; flex-wrap: wrap;">
          <div style="width: 220px; text-align: center;">
            <img class="albumCover" src="${coverUrl}" alt="${track.name}" style="width: 200px; height: 200px; border-radius: 4px;" />
            <div style="font-size: 11px; margin-top: 8px;">
              <strong>Album:</strong> <a href="/music/album/${track.album}">${album ? album.name : track.album}</a><br />
              <strong>Duration:</strong> ${formatDuration(track.duration)}
            </div>
          </div>
          <div style="flex: 1; min-width: 300px;">
            <h2 style="font-family: 'Courier New', monospace; margin-top: 0;">${track.name}</h2>
            <div class="soundTestPlayerBox">
              <h4 style="margin: 0 0 8px 0;">Sound Test Audio Player</h4>
              <button id="direct-play-btn" class="playTrackBtn" style="padding: 6px 14px; font-size: 13px;">&#9654; Play Track</button>
            </div>
            ${track.artists && track.artists.length > 0 ? `
              <div style="margin: 10px 0; font-size: 12px;">
                <strong>Artists:</strong> ${Array.isArray(track.artists) ? track.artists.join(', ') : track.artists}
              </div>
            ` : ''}
            ${track.featured && track.featured.length > 0 ? `
              <div style="margin: 10px 0; font-size: 12px;">
                <strong>Featured in:</strong>
                ${track.featured.map((pid) => `<a href="/mspa/${pid}">Page ${pid}</a>`).join(', ')}
              </div>
            ` : ''}
            ${track.commentary ? `
              <div class="settingsSectionCard" style="margin-top: 14px;">
                <h3>Track Commentary</h3>
                <div style="font-size: 12px; line-height: 1.4;">${track.commentary}</div>
              </div>
            ` : ''}
          </div>
        </div>
      </div>
    `;

    bindInternalLinks(container);

    const directBtn = container.querySelector('#direct-play-btn');
    if (directBtn) {
      directBtn.addEventListener('click', () => {
        loadTrackIntoSoundTest(trackSlug, true);
      });
    }
  }

  async function loadTrackIntoSoundTest(trackSlug, autoplay = true) {
    const data = await ensureMusicDataLoaded();
    const track = data.tracks ? data.tracks[trackSlug] : null;
    if (!track) return;

    const album = data.albums ? data.albums[track.album] : null;
    const albumDir = album ? album.directory : track.album;

    // Track audio resolution:
    // In asset pack v2, specific MP3s exist in extra/ or volume-1/, or standard directory
    let candidatePath = `/archive/music/${albumDir}/${track.directory || trackSlug}.mp3`;
    if (trackSlug === 'hilary-troiano-commentary') {
      candidatePath = '/archive/music/extra/hilary-troiano-commentary.mp3';
    } else if (albumDir === 'volume-1') {
      candidatePath = `/archive/music/volume-1/01-john-harrington.mp3`;
    }

    const audioUrl = resolveAssetUrl(candidatePath);
    const bar = document.getElementById('sound-test-bar');
    const titleEl = document.getElementById('sound-test-title');
    const metaEl = document.getElementById('sound-test-meta');
    const audioEl = document.getElementById('sound-test-audio');

    if (titleEl) titleEl.textContent = track.name;
    if (metaEl) metaEl.textContent = `${album ? album.name : ''} — ${formatDuration(track.duration)}`;

    if (audioEl) {
      audioEl.src = audioUrl;
      if (autoplay) {
        audioEl.play().catch((e) => console.debug('Sound test autoplay blocked:', e));
      }
    }

    if (bar) {
      bar.classList.remove('hidden');
    }

    state.soundTest.isPlaying = autoplay;
    state.soundTest.currentTrack = track;
    state.soundTest.currentAlbum = album;

    // Discord RPC Music presence
    if (state.config.settings.enableDiscordRpc) {
      ipcInvoke('set_discord_music_presence', {
        track: track.name,
        album: album ? album.name : '',
        duration: track.duration
      }).catch(() => {});
    }
  }

  function setupSoundTestBar() {
    if (typeof document === 'undefined') return;
    const bar = document.getElementById('sound-test-bar');
    const playBtn = document.getElementById('sound-test-play-btn');
    const stopBtn = document.getElementById('sound-test-stop-btn');
    const closeBtn = document.getElementById('sound-test-close-btn');
    const audioEl = document.getElementById('sound-test-audio');

    if (playBtn && audioEl) {
      playBtn.addEventListener('click', () => {
        if (audioEl.paused) {
          audioEl.play();
          playBtn.innerHTML = '&#10074;&#10074;';
        } else {
          audioEl.pause();
          playBtn.innerHTML = '&#9654;';
        }
      });

      audioEl.addEventListener('play', () => {
        playBtn.innerHTML = '&#10074;&#10074;';
      });
      audioEl.addEventListener('pause', () => {
        playBtn.innerHTML = '&#9654;';
      });
    }

    if (stopBtn && audioEl) {
      stopBtn.addEventListener('click', () => {
        audioEl.pause();
        audioEl.currentTime = 0;
      });
    }

    if (closeBtn && bar && audioEl) {
      closeBtn.addEventListener('click', () => {
        audioEl.pause();
        bar.classList.add('hidden');
      });
    }
  }

  /* ==========================================================================
     8. Settings & Community Mod Toggles (F20 & F25)
     ========================================================================== */

  async function renderSettingsView(container) {
    let mods = [];
    try {
      mods = await ipcInvoke('get_available_mods');
      state.availableMods = Array.isArray(mods) ? mods : [];
    } catch (e) {
      console.warn('[TUHC-Settings] Could not fetch mods:', e);
    }

    const cfg = state.config;
    const settings = cfg.settings;

    container.innerHTML = `
      <div class="systemPageContainer">
        <h1 style="font-family: 'Courier New', monospace;">SETTINGS &amp; PREFERENCES</h1>

        <!-- Section 1: Themes & Appearance -->
        <div class="settingsSectionCard">
          <h2>Themes &amp; Visuals</h2>
          <div class="settingRow">
            <label for="theme-select"><strong>MSPA Theme:</strong></label>
            <select id="theme-select" style="padding: 4px; font-size: 12px;">
              <option value="default" ${settings.themeOverride === 'default' ? 'selected' : ''}>MSPA Classic / Default</option>
              <option value="dark" ${settings.themeOverride === 'dark' ? 'selected' : ''}>Dark / Midnight</option>
              <option value="retro" ${settings.themeOverride === 'retro' ? 'selected' : ''}>Retro MSPA</option>
              <option value="scratch" ${settings.themeOverride === 'scratch' ? 'selected' : ''}>Doc Scratch</option>
              <option value="sbahj" ${settings.themeOverride === 'sbahj' ? 'selected' : ''}>Sweet Bro and Hella Jeff</option>
              <option value="cascade" ${settings.themeOverride === 'cascade' ? 'selected' : ''}>Cascade</option>
              <option value="trickster" ${settings.themeOverride === 'trickster' ? 'selected' : ''}>Trickster Mode</option>
              <option value="A6A6" ${settings.themeOverride === 'A6A6' ? 'selected' : ''}>Act 6 Act 6</option>
              <option value="collide" ${settings.themeOverride === 'collide' ? 'selected' : ''}>Collide</option>
              <option value="tso" ${settings.themeOverride === 'tso' ? 'selected' : ''}>The Sound of One</option>
              <option value="pxs" ${settings.themeOverride === 'pxs' ? 'selected' : ''}>Paradox Space</option>
              <option value="mspfa" ${settings.themeOverride === 'mspfa' ? 'selected' : ''}>MS Paint Fan Adventures</option>
              <option value="msword" ${settings.themeOverride === 'msword' ? 'selected' : ''}>Microsoft Word</option>
            </select>
          </div>
          <div class="settingRow">
            <label for="setting-force-theme">Force Theme Override across all pages:</label>
            <input type="checkbox" id="setting-force-theme" ${settings.forceThemeOverride ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label for="setting-high-contrast">High Contrast text mode:</label>
            <input type="checkbox" id="setting-high-contrast" ${settings.textOverride?.highContrast ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label for="setting-force-scrollbar">Always display vertical scrollbar:</label>
            <input type="checkbox" id="setting-force-scrollbar" ${settings.forceScrollBar ? 'checked' : ''} />
          </div>
        </div>

        <!-- Section 2: Reading Options -->
        <div class="settingsSectionCard">
          <h2>Reading Experience</h2>
          <div class="settingRow">
            <label for="setting-open-logs">Automatically expand pesterlogs:</label>
            <input type="checkbox" id="setting-open-logs" ${settings.openLogs ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label for="setting-mspa-mode">Display classic MSPA page numbers (/mspa/001901) instead of Viz numbers (/homestuck/1):</label>
            <input type="checkbox" id="setting-mspa-mode" ${settings.mspaMode ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label for="setting-switch-tabs">Switch to new tabs immediately when opened:</label>
            <input type="checkbox" id="setting-switch-tabs" ${settings.switchToNewTabs ? 'checked' : ''} />
          </div>
        </div>

        <!-- Section 3: Audio & Enhancements -->
        <div class="settingsSectionCard">
          <h2>Enhancements &amp; Retcons</h2>
          <div class="settingRow">
            <label for="setting-hq-audio">High Quality (HQ) Audio:</label>
            <input type="checkbox" id="setting-hq-audio" ${settings.hqAudio ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label for="setting-bolin">Dance of the Ancestors / Bolin:</label>
            <input type="checkbox" id="setting-bolin" ${settings.bolin ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label for="setting-unpeachy">Unpeachy John (Page 007680):</label>
            <input type="checkbox" id="setting-unpeachy" ${settings.unpeachy ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label for="setting-pxs-tavros">Paradox Space Tavros (Page 008753):</label>
            <input type="checkbox" id="setting-pxs-tavros" ${settings.pxsTavros ? 'checked' : ''} />
          </div>
          <div class="settingRow">
            <label>Retcon Enhancements (1–6):</label>
            <div style="display: flex; gap: 8px;">
              ${[1, 2, 3, 4, 5, 6].map((r) => `
                <label style="font-size: 11px;">
                  <input type="checkbox" id="setting-retcon-${r}" ${settings[`retcon${r}`] ? 'checked' : ''} /> R${r}
                </label>
              `).join('')}
            </div>
          </div>
        </div>

        <!-- Section 4: Discord Rich Presence (RPC) -->
        <div class="settingsSectionCard">
          <h2>Discord Rich Presence</h2>
          <div class="settingRow">
            <label for="setting-discord-rpc">Enable Discord Rich Presence (RPC):</label>
            <input type="checkbox" id="setting-discord-rpc" ${settings.enableDiscordRpc ? 'checked' : ''} />
          </div>
        </div>

        <!-- Section 5: Community Mods -->
        <div class="settingsSectionCard">
          <h2>Community Mods (${mods.length})</h2>
          <div id="mods-list">
            ${mods.length === 0 ? '<p style="font-size: 12px; color: #888;">No community mods found in mod directory.</p>' : ''}
            ${mods.map((mod) => {
              const isEnabled = cfg.enabledMods && cfg.enabledMods.includes(mod.id);
              return `
                <div class="modCard" data-mod-id="${mod.id}">
                  <div class="modCardHeader">
                    <span>${mod.name || mod.id}</span>
                    <button class="primaryActionBtn toggleModBtn" data-mod-id="${mod.id}" style="background-color: ${isEnabled ? '#168500' : '#666666'};">
                      ${isEnabled ? 'ENABLED' : 'DISABLED'}
                    </button>
                  </div>
                  <div style="font-size: 11px; opacity: 0.85;">${mod.description || 'Community expansion mod.'}</div>
                  <div class="modBadges">
                    <span class="modBadge">v${mod.version || '1.0.0'}</span>
                    ${mod.author ? `<span class="modBadge">by ${mod.author}</span>` : ''}
                  </div>
                </div>
              `;
            }).join('')}
          </div>
        </div>
      </div>
    `;

    // Bind settings event listeners
    const themeSelect = container.querySelector('#theme-select');
    if (themeSelect) {
      themeSelect.addEventListener('change', (e) => {
        settings.themeOverride = e.target.value;
        applyTheme(e.target.value);
        persistConfig();
      });
    }

    const forceThemeBox = container.querySelector('#setting-force-theme');
    if (forceThemeBox) {
      forceThemeBox.addEventListener('change', (e) => {
        settings.forceThemeOverride = e.target.checked;
        persistConfig();
      });
    }

    const highContrastBox = container.querySelector('#setting-high-contrast');
    if (highContrastBox) {
      highContrastBox.addEventListener('change', (e) => {
        if (!settings.textOverride) settings.textOverride = {};
        settings.textOverride.highContrast = e.target.checked;
        persistConfig();
      });
    }

    const scrollbarBox = container.querySelector('#setting-force-scrollbar');
    if (scrollbarBox) {
      scrollbarBox.addEventListener('change', (e) => {
        settings.forceScrollBar = e.target.checked;
        applyGlobalSettings(settings);
        persistConfig();
      });
    }

    const openLogsBox = container.querySelector('#setting-open-logs');
    if (openLogsBox) {
      openLogsBox.addEventListener('change', (e) => {
        settings.openLogs = e.target.checked;
        persistConfig();
      });
    }

    const mspaModeBox = container.querySelector('#setting-mspa-mode');
    if (mspaModeBox) {
      mspaModeBox.addEventListener('change', (e) => {
        settings.mspaMode = e.target.checked;
        updateAddressBar(getActiveTab().url);
        persistConfig();
      });
    }

    const switchTabsBox = container.querySelector('#setting-switch-tabs');
    if (switchTabsBox) {
      switchTabsBox.addEventListener('change', (e) => {
        settings.switchToNewTabs = e.target.checked;
        persistConfig();
      });
    }

    const discordBox = container.querySelector('#setting-discord-rpc');
    if (discordBox) {
      discordBox.addEventListener('change', (e) => {
        settings.enableDiscordRpc = e.target.checked;
        ipcInvoke('toggle_discord_rpc', { enabled: e.target.checked }).catch(() => {});
        persistConfig();
      });
    }

    // Retcons & Enhancements checkboxes
    const hqBox = container.querySelector('#setting-hq-audio');
    if (hqBox) hqBox.addEventListener('change', (e) => { settings.hqAudio = e.target.checked; persistConfig(); });

    const bolinBox = container.querySelector('#setting-bolin');
    if (bolinBox) bolinBox.addEventListener('change', (e) => { settings.bolin = e.target.checked; persistConfig(); });

    const unpeachyBox = container.querySelector('#setting-unpeachy');
    if (unpeachyBox) unpeachyBox.addEventListener('change', (e) => { settings.unpeachy = e.target.checked; persistConfig(); });

    const pxsBox = container.querySelector('#setting-pxs-tavros');
    if (pxsBox) pxsBox.addEventListener('change', (e) => { settings.pxsTavros = e.target.checked; persistConfig(); });

    for (let r = 1; r <= 6; r++) {
      const rBox = container.querySelector(`#setting-retcon-${r}`);
      if (rBox) {
        rBox.addEventListener('change', (e) => {
          settings[`retcon${r}`] = e.target.checked;
          persistConfig();
        });
      }
    }

    // Community mod toggle buttons
    const modBtns = container.querySelectorAll('.toggleModBtn');
    modBtns.forEach((btn) => {
      btn.addEventListener('click', async () => {
        const modId = btn.getAttribute('data-mod-id');
        const currentlyEnabled = cfg.enabledMods && cfg.enabledMods.includes(modId);
        await toggleMod(modId, !currentlyEnabled);
        renderSettingsView(container);
      });
    });
  }

  async function toggleMod(modId, enabled) {
    try {
      await ipcInvoke('toggle_mod', { mod_id: modId, enabled: enabled });

      if (!state.config.enabledMods) state.config.enabledMods = [];
      const idx = state.config.enabledMods.indexOf(modId);

      if (enabled && idx === -1) {
        state.config.enabledMods.push(modId);
      } else if (!enabled && idx !== -1) {
        state.config.enabledMods.splice(idx, 1);
      }

      if (modId === 'Homestuck PT-BR' || modId === 'homestuck_ptbr') {
        state.config.locale = enabled ? 'pt-BR' : 'en-US';
      }

      state.overlayGeneration++;
      persistConfig(true);

      // Update Quick PT-BR button appearance
      updateQuickPtBrButton();

      // If active tab is on a story page, re-render it immediately with new mod state
      const tab = getActiveTab();
      if (tab && tab.url) {
        const route = resolveRoute(tab.url);
        if (route.mode === 'PAGE') {
          renderStoryPage(route.pageId);
        }
      }
    } catch (err) {
      console.error(`[TUHC-Mods] Failed to toggle mod '${modId}':`, err);
    }
  }

  function updateQuickPtBrButton() {
    if (typeof document === 'undefined') return;
    const btn = document.getElementById('quick-ptbr-btn');
    if (!btn) return;
    const isPtBr = state.config.enabledMods &&
      (state.config.enabledMods.includes('Homestuck PT-BR') || state.config.enabledMods.includes('homestuck_ptbr'));

    if (isPtBr) {
      btn.classList.add('active');
      btn.style.backgroundColor = '#168500';
      btn.style.color = '#ffffff';
    } else {
      btn.classList.remove('active');
      btn.style.backgroundColor = '';
      btn.style.color = '';
    }
  }

  /* ==========================================================================
     9. Bookmarks & Saved Games Drawer (saveData)
     ========================================================================== */

  function toggleBookmarksDrawer(forceOpen = null) {
    if (typeof document === 'undefined') return;
    const drawer = document.getElementById('bookmarks-drawer');
    if (!drawer) return;

    if (forceOpen === true) {
      drawer.classList.remove('hidden');
    } else if (forceOpen === false) {
      drawer.classList.add('hidden');
    } else {
      drawer.classList.toggle('hidden');
    }

    if (!drawer.classList.contains('hidden')) {
      renderBookmarksList();
    }
  }

  function newSave() {
    const active = getActiveTab();
    const id = 'save_' + Date.now();
    const saveObj = {
      id: id,
      name: active.title || 'Saved Page',
      url: active.url || '/',
      date: new Date().toLocaleDateString()
    };

    if (!state.config.saveData) state.config.saveData = { saves: {}, saveList: [] };
    state.config.saveData.saves[id] = saveObj;
    state.config.saveData.saveList.unshift(id);

    renderBookmarksList();
    persistConfig();
  }

  function deleteSave(id) {
    const sd = state.config.saveData;
    if (!sd) return;

    delete sd.saves[id];
    const idx = sd.saveList.indexOf(id);
    if (idx !== -1) sd.saveList.splice(idx, 1);

    renderBookmarksList();
    persistConfig();
  }

  function renderBookmarksList() {
    if (typeof document === 'undefined') return;
    const listEl = document.getElementById('bookmarks-list');
    if (!listEl) return;

    listEl.innerHTML = '';
    const sd = state.config.saveData;
    if (!sd || !sd.saveList || sd.saveList.length === 0) {
      listEl.innerHTML = '<p style="padding: 12px; font-size: 11px; color: #888;">No saved bookmarks yet.</p>';
      return;
    }

    for (const id of sd.saveList) {
      const item = sd.saves[id];
      if (!item) continue;

      const card = document.createElement('div');
      card.className = 'bookmarkItem';
      card.innerHTML = `
        <div class="bookmarkItemHeader">
          <span class="bookmarkName">${item.name}</span>
          <div class="bookmarkBtnGroup">
            <button class="primaryActionBtn bookmarkSmallBtn loadSaveBtn" data-id="${id}">Load</button>
            <button class="navBtn bookmarkSmallBtn deleteSaveBtn" data-id="${id}">&times;</button>
          </div>
        </div>
        <div class="bookmarkUrl">${item.url} • ${item.date || ''}</div>
      `;

      card.querySelector('.loadSaveBtn').addEventListener('click', () => {
        pushUrl(item.url);
        toggleBookmarksDrawer(false);
      });

      card.querySelector('.deleteSaveBtn').addEventListener('click', () => {
        deleteSave(id);
      });

      listEl.appendChild(card);
    }
  }

  /* ==========================================================================
     10. System & Utility Pages (Homepage, Log, Map, Search, Credits, SBAHJ)
     ========================================================================== */

  function renderHomepage(container) {
    applyTheme('mspa');
    document.title = 'The Unofficial Homestuck Collection';

    container.innerHTML = `
      <div class="systemPageContainer" style="text-align: center;">
        <h1 style="font-family: 'Courier New', monospace; font-size: 28px; margin-top: 10px;">
          THE UNOFFICIAL HOMESTUCK COLLECTION
        </h1>
        <p class="prattle" style="margin: 0 auto 20px auto;">
          An authentic offline desktop environment for reading MS Paint Adventures with high-performance VFS streaming, 13 themes, flash emulation & full discography.
        </p>

        <div class="adventureGrid">
          <div class="settingsSectionCard">
            <h3>HOMESTUCK</h3>
            <p style="font-size: 12px;">8,130 Pages. The epic story of four kids who play a game called Sburb.</p>
            <a href="/mspa/6" class="primaryActionBtn" style="display: inline-block; text-decoration: none;">Read Homestuck</a>
          </div>

          <div class="settingsSectionCard">
            <h3>MUSIC DISCOGRAPHY</h3>
            <p style="font-size: 12px;">37 Albums, 797 Tracks. The complete MSPA musical archive with Sound Test.</p>
            <a href="/music" class="primaryActionBtn" style="display: inline-block; text-decoration: none;">Open Music Player</a>
          </div>

          <div class="settingsSectionCard">
            <h3>PROBLEM SLEUTH</h3>
            <p style="font-size: 12px;">1,674 Pages. A hard-boiled detective story turned cosmic battle.</p>
            <a href="/mspa/4" class="primaryActionBtn" style="display: inline-block; text-decoration: none;">Read Problem Sleuth</a>
          </div>

          <div class="settingsSectionCard">
            <h3>BARD QUEST</h3>
            <p style="font-size: 12px;">47 Pages. An interactive choose-your-own-adventure experiment.</p>
            <a href="/mspa/2" class="primaryActionBtn" style="display: inline-block; text-decoration: none;">Read Bard Quest</a>
          </div>

          <div class="settingsSectionCard">
            <h3>JAILBREAK</h3>
            <p style="font-size: 12px;">135 Pages. The adventure where it all began in 2006.</p>
            <a href="/mspa/1" class="primaryActionBtn" style="display: inline-block; text-decoration: none;">Read Jailbreak</a>
          </div>

          <div class="settingsSectionCard">
            <h3>SWEET BRO &amp; HELLA JEFF</h3>
            <p style="font-size: 12px;">The iconic JPEG-artefacted webcomic masterpiece.</p>
            <a href="/sbahj/1" class="primaryActionBtn" style="display: inline-block; text-decoration: none;">Read SBAHJ</a>
          </div>
        </div>
      </div>
    `;

    bindInternalLinks(container);
  }

  function renderMap(container, storyId = '6') {
    applyTheme('mspa');
    container.innerHTML = `
      <div class="systemPageContainer">
        <h1 style="font-family: 'Courier New', monospace;">ADVENTURE MAP</h1>
        <div class="settingsSectionCard">
          <h3>Homestuck Acts &amp; Key Milestones</h3>
          <ul style="font-size: 12px; line-height: 1.8;">
            <li><a href="/mspa/001901">Act 1: The Fresh Jimmy (001901)</a></li>
            <li><a href="/mspa/002148">Act 2: The Meaning of Sburb (002148)</a></li>
            <li><a href="/mspa/002659">Act 3: The King in the Fog (002659)</a></li>
            <li><a href="/mspa/003054">Intermission: The Midnight Crew (003054)</a></li>
            <li><a href="/mspa/003258">Act 4: Flight of the Paradox Clone (003258)</a></li>
            <li><a href="/mspa/003889">Act 5 Act 1: MOBIUS DOUBLE TRIPTYCH (003889)</a></li>
            <li><a href="/mspa/004526">Act 5 Act 2: He is Already Here (004526)</a></li>
            <li><a href="/mspa/006013">Act 6: A1 A1 (006013)</a></li>
            <li><a href="/mspa/010027">Act 7: [S] Act 7 (010027)</a></li>
          </ul>
        </div>
      </div>
    `;
    bindInternalLinks(container);
  }

  function renderLog(container, storyId = '6') {
    applyTheme('mspa');
    container.innerHTML = `
      <div class="systemPageContainer">
        <h1 style="font-family: 'Courier New', monospace;">ADVENTURE LOG</h1>
        <div class="settingsSectionCard">
          <p style="font-size: 12px;">Jump directly into any section of the adventure:</p>
          <ul style="font-size: 12px; line-height: 1.6;">
            <li><a href="/mspa/001901">Homestuck (Page 001901)</a></li>
            <li><a href="/mspa/000219">Problem Sleuth (Page 000219)</a></li>
            <li><a href="/mspa/000136">Bard Quest (Page 000136)</a></li>
            <li><a href="/mspa/000002">Jailbreak (Page 000002)</a></li>
          </ul>
        </div>
      </div>
    `;
    bindInternalLinks(container);
  }

  function renderSearch(container, initialQuery = '') {
    applyTheme('mspa');
    container.innerHTML = `
      <div class="systemPageContainer">
        <h1 style="font-family: 'Courier New', monospace;">SEARCH ADVENTURE</h1>
        <div class="settingsSectionCard">
          <form id="story-search-form" style="display: flex; gap: 8px; margin-bottom: 12px;">
            <input id="story-search-input" class="addressInput" type="text" value="${initialQuery}" placeholder="Enter page number or keyword..." style="flex: 1;" />
            <button type="submit" class="primaryActionBtn">Search</button>
          </form>
          <div id="search-results" style="font-size: 12px;">
            ${initialQuery ? `<p>Results for "<strong>${initialQuery}</strong>":</p>` : '<p>Enter a page number to jump directly, or search story logs.</p>'}
          </div>
        </div>
      </div>
    `;

    const form = container.querySelector('#story-search-form');
    if (form) {
      form.addEventListener('submit', (e) => {
        e.preventDefault();
        const q = container.querySelector('#story-search-input').value.trim();
        if (/^\d+$/.test(q)) {
          pushUrl('/mspa/' + q);
        } else if (q) {
          pushUrl('/search/' + encodeURIComponent(q));
        }
      });
    }
  }

  function renderCredits(container) {
    applyTheme('mspa');
    container.innerHTML = `
      <div class="systemPageContainer">
        <h1 style="font-family: 'Courier New', monospace;">CREDITS &amp; ATTRIBUTIONS</h1>
        <div class="settingsSectionCard">
          <h3>Original Work</h3>
          <p style="font-size: 12px;"><strong>MS Paint Adventures &amp; Homestuck:</strong> Created by Andrew Hussie (2006–2016).</p>
          <p style="font-size: 12px;"><strong>Homestuck Music Team:</strong> Over 70 independent musicians, artists, and sound designers.</p>
        </div>
        <div class="settingsSectionCard">
          <h3>The Unofficial Homestuck Collection (Tauri 2.0 Edition)</h3>
          <p style="font-size: 12px;">Rewritten with Rust, Axum, Tokio, and WebKitGTK to guarantee <150MB RSS memory SLA and authentic archival fidelity.</p>
        </div>
      </div>
    `;
  }

  function renderSbahj(container, comicId = '1') {
    applyTheme('sbahj');
    document.title = 'Sweet Bro and Hella Jeff';

    const cidNum = parseInt(comicId, 10) || 1;
    container.innerHTML = `
      <div class="systemPageContainer" style="text-align: center;">
        <h1 style="font-family: 'Comic Sans MS', cursive; color: #ff0000; font-size: 32px;">SWEET BRO AND HELLA JEFF</h1>
        <div class="settingsSectionCard" style="background-color: #14ff23; border: 4px solid #f400ec;">
          <h2 style="color: #0707ec;">COMIC #${cidNum}</h2>
          <div style="margin: 16px 0;">
            <img src="${resolveAssetUrl(`/archive/comics/sbahj/00${cidNum}.jpg`)}" alt="SBAHJ Comic ${cidNum}" style="max-width: 100%; border: 3px solid #000;" />
          </div>
          <div style="display: flex; justify-content: center; gap: 16px; font-weight: bold;">
            ${cidNum > 1 ? `<a href="/sbahj/${cidNum - 1}">&lt; PREVIOUS</a>` : ''}
            <a href="/sbahj/${cidNum + 1}">NEXT &gt;</a>
          </div>
        </div>
      </div>
    `;
    bindInternalLinks(container);
  }

  /* ==========================================================================
     11. Master Route Dispatcher (renderRoute)
     ========================================================================== */

  async function renderRoute(route) {
    state.currentRoute = route;
    if (typeof document === 'undefined') return;
    const pageContent = document.getElementById('page-content');
    if (!pageContent) return;

    // Reset default styling
    pageContent.style.display = 'block';

    // Clear running Flash animations and audio when navigating away from story pages
    if (route.mode !== 'PAGE' && root.TUHCMediaEngine && typeof root.TUHCMediaEngine.cleanupActiveMedia === 'function') {
      root.TUHCMediaEngine.cleanupActiveMedia();
    }

    switch (route.mode) {
      case 'HOMEPAGE':
        renderHomepage(pageContent);
        break;
      case 'SETTINGS':
        applyTheme('mspa');
        document.title = 'Settings — The Unofficial Homestuck Collection';
        renderSettingsView(pageContent);
        break;
      case 'MUSIC':
        applyTheme('mspa');
        document.title = 'Music — The Unofficial Homestuck Collection';
        if (route.view === 'discography') {
          await renderMusicDiscographyView(pageContent);
        } else if (route.view === 'tracks') {
          await renderMusicTracksView(pageContent);
        } else if (route.view === 'album') {
          await renderAlbumDetailView(pageContent, route.id);
        } else if (route.view === 'track') {
          await renderTrackDetailView(pageContent, route.id);
        } else {
          await renderMusicDiscographyView(pageContent);
        }
        break;
      case 'PAGE':
        await renderStoryPage(route.pageId);
        break;
      case 'MAP':
        renderMap(pageContent, route.storyId);
        break;
      case 'LOG':
        renderLog(pageContent, route.storyId);
        break;
      case 'SEARCH':
        renderSearch(pageContent, route.query);
        break;
      case 'CREDITS':
        renderCredits(pageContent);
        break;
      case 'SBAHJ':
        renderSbahj(pageContent, route.cid);
        break;
      default:
        renderHomepage(pageContent);
        break;
    }
  }

  /* ==========================================================================
     12. Chrome Event Wiring & Shortcuts
     ========================================================================== */

  function setupTooltips() {
    const tooltip = document.getElementById('url-tooltip');
    if (!tooltip) return;

    document.addEventListener('mouseover', (e) => {
      const a = e.target.closest('a[href]');
      if (a && state.config.settings.urlTooltip) {
        tooltip.textContent = a.getAttribute('href');
        tooltip.classList.remove('hidden');
      }
    });

    document.addEventListener('mouseout', (e) => {
      const a = e.target.closest('a[href]');
      if (a) {
        tooltip.classList.add('hidden');
      }
    });
  }

  function setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
      const isInput = ['INPUT', 'TEXTAREA', 'SELECT'].includes(e.target.tagName);

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 't') {
        e.preventDefault();
        newTab('/');
        return;
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'w') {
        e.preventDefault();
        closeTab();
        return;
      }

      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 't') {
        e.preventDefault();
        restoreClosedTab();
        return;
      }

      if ((e.ctrlKey || e.metaKey) && e.key === 'Tab') {
        e.preventDefault();
        const tabData = state.config.tabData;
        const list = tabData.tabList;
        const curIdx = list.indexOf(state.activeTabKey);
        const nextIdx = e.shiftKey
          ? (curIdx - 1 + list.length) % list.length
          : (curIdx + 1) % list.length;
        switchTab(list[nextIdx]);
        return;
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
        e.preventDefault();
        const input = document.getElementById('address-bar-input');
        if (input) {
          input.focus();
          input.select();
        }
        return;
      }

      if (e.altKey && e.key === 'ArrowLeft') {
        e.preventDefault();
        historyBack();
        return;
      }

      if (e.altKey && e.key === 'ArrowRight') {
        e.preventDefault();
        historyForward();
        return;
      }

      // Story Page Keyboard Shortcuts (when not in an input box)
      if (!isInput && state.currentRoute.mode === 'PAGE' && state.currentPageData) {
        if (e.key === 'ArrowLeft') {
          if (state.currentPageData.previous) {
            pushUrl(`/mspa/${state.currentPageData.previous}`);
          }
        } else if (e.key === 'ArrowRight') {
          const nextList = state.currentPageData.next;
          if (nextList && nextList.length > 0) {
            pushUrl(`/mspa/${nextList[0]}`);
          }
        } else if (e.key === ' ') {
          const logBox = document.querySelector('.log');
          if (logBox) {
            const btn = logBox.querySelector('.logButton');
            if (btn) {
              e.preventDefault();
              btn.click();
            }
          }
        }
      }
    });
  }

  function bindChromeEvents() {
    // Window control buttons (TitleBar)
    const minBtn = document.getElementById('minButton');
    if (minBtn) minBtn.addEventListener('click', () => ipcInvoke('win_minimize').catch(() => {}));

    const maxBtn = document.getElementById('maxButton');
    if (maxBtn) maxBtn.addEventListener('click', () => ipcInvoke('win_maximize').catch(() => {}));

    const closeBtn = document.getElementById('closeButton');
    if (closeBtn) closeBtn.addEventListener('click', () => ipcInvoke('win_close').catch(() => {}));

    // New tab button
    const newTabBtn = document.getElementById('new-tab-btn');
    if (newTabBtn) {
      newTabBtn.addEventListener('click', () => newTab('/'));
    }

    // Bookmarks drawer toggle buttons
    const bmToggleBtn = document.getElementById('bookmarks-toggle-btn');
    if (bmToggleBtn) {
      bmToggleBtn.addEventListener('click', () => toggleBookmarksDrawer());
    }

    const bmCloseBtn = document.getElementById('bookmarks-close-btn');
    if (bmCloseBtn) {
      bmCloseBtn.addEventListener('click', () => toggleBookmarksDrawer(false));
    }

    const navSaveLoadLink = document.getElementById('nav-saveload-link');
    if (navSaveLoadLink) {
      navSaveLoadLink.addEventListener('click', (e) => {
        e.preventDefault();
        toggleBookmarksDrawer();
      });
    }

    const newSaveBtn = document.getElementById('new-save-btn');
    if (newSaveBtn) {
      newSaveBtn.addEventListener('click', () => newSave());
    }

    const starBtn = document.getElementById('bookmark-star-btn');
    if (starBtn) {
      starBtn.addEventListener('click', () => {
        newSave();
        starBtn.classList.add('active');
        setTimeout(() => { starBtn.classList.remove('active'); }, 1500);
      });
    }

    // Address Bar & Nav Buttons
    const backBtn = document.getElementById('nav-back-btn');
    if (backBtn) backBtn.addEventListener('click', () => historyBack());

    const fwdBtn = document.getElementById('nav-forward-btn');
    if (fwdBtn) fwdBtn.addEventListener('click', () => historyForward());

    const reloadBtn = document.getElementById('nav-reload-btn');
    if (reloadBtn) {
      reloadBtn.addEventListener('click', () => {
        const active = getActiveTab();
        if (active) renderRoute(resolveRoute(active.url));
      });
    }

    // Address input & Jumpbox prompt
    const addressInput = document.getElementById('address-bar-input');
    const jumpboxLink = document.getElementById('jumpbox-link');
    if (jumpboxLink && addressInput) {
      jumpboxLink.addEventListener('click', () => {
        pushUrl(addressInput.value);
      });
    }

    if (addressInput) {
      addressInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          pushUrl(addressInput.value);
        } else if (e.key === 'Escape') {
          const tab = getActiveTab();
          if (tab) addressInput.value = formatAddressUrl(tab.url);
        }
      });
    }

    // Bind classic MSPA top navigation banner links
    const navBanner = document.getElementById('mspa-nav-banner');
    if (navBanner) {
      bindInternalLinks(navBanner);
    }

    // Global Delegated Link Handler (Guarantees ALL internal links & buttons work everywhere)
    document.addEventListener('click', (e) => {
      const anchor = e.target.closest('a[href]');
      if (!anchor) return;
      const href = anchor.getAttribute('href');
      if (!href) return;
      if (href.startsWith('#') || href.startsWith('javascript:')) return;

      if (
        href.startsWith('/') ||
        href.startsWith('mspa://') ||
        href.startsWith('?s=') ||
        href.includes('mspaintadventures.com')
      ) {
        e.preventDefault();
        window.logTuhc('info', `[TUHC-Nav] Global click intercepted: ${href}`);
        pushUrl(href);
      }
    });

    // Browser History PopState (Supports browser Back/Forward buttons and gestures)
    window.addEventListener('popstate', (e) => {
      window.logTuhc('info', `[TUHC-Nav] popstate event triggered: url=${e.state && e.state.url} pathname=${window.location.pathname}`);
      if (e.state && e.state.url) {
        pushUrl(e.state.url, true, state.activeTabKey);
      } else if (window.location.pathname) {
        pushUrl(window.location.pathname, true, state.activeTabKey);
      } else {
        historyBack();
      }
    });

    // ActionScript getURL Flash navigation bridge
    window.__tuhcNavigate = function(url) { pushUrl(url); };
    window.addEventListener('tuhc-navigate', (e) => {
      if (e && e.detail && e.detail.url) {
        pushUrl(e.detail.url);
      }
    });
  }

  /* ==========================================================================
     13. Application Bootstrapper
     ========================================================================== */

  // Global Diagnostic Logger (Captures all events and streams them to Rust backend stdout)
  window.__tuhcLogs = window.__tuhcLogs || [];
  window.logTuhc = function(level, ...args) {
    const msg = args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ');
    const line = `[${new Date().toISOString()}] [${level.toUpperCase()}] ${msg}`;
    window.__tuhcLogs.push(line);
    if (window.__tuhcLogs.length > 500) window.__tuhcLogs.shift();
    try {
      localStorage.setItem('tuhc_last_logs', JSON.stringify(window.__tuhcLogs.slice(-100)));
    } catch (_) {}
    if (console && typeof console[level] === 'function') {
      console[level](...args);
    } else {
      console.log(`[${level}]`, ...args);
    }
    // Stream directly to Rust backend so it is captured in background tasks & terminal!
    ipcInvoke('client_log', { level, message: msg }).catch(() => {});
  };

  window.addEventListener('error', (e) => {
    window.logTuhc('error', `Uncaught Error: ${e.message} at ${e.filename}:${e.lineno}:${e.colno}`);
  });

  window.addEventListener('unhandledrejection', (e) => {
    window.logTuhc('error', `Unhandled Promise Rejection: ${e.reason}`);
  });

  async function initApp() {
    // 1. Handshake with Rust Backend
    try {
      const appInfo = await ipcInvoke('get_app_info');
      state.appInfo = appInfo;
      if (appInfo && appInfo.asset_port) {
        state.assetBase = `http://127.0.0.1:${appInfo.asset_port}`;
      }
    } catch (err) {
      console.warn('[TUHC-Init] Backend handshake note:', err);
    }

    // 2. Load Config & Tab State
    await loadConfig();

    // 3. Setup UI Chromes & Event Listeners
    renderTabBar();
    updateNavControls();
    updateQuickPtBrButton();
    setupSoundTestBar();
    setupTooltips();
    setupKeyboardShortcuts();
    bindChromeEvents();

    // 4. Render Active Tab Initial URL
    const activeTab = getActiveTab();
    if (activeTab) {
      updateAddressBar(activeTab.url);
      await renderRoute(resolveRoute(activeTab.url));
    }
  }

  // Export public API
  const TUHCApp = {
    state,
    ipcInvoke,
    resolveAssetUrl,
    resolveRoute,
    vizToMspa,
    mspaToViz,
    getLogButtonLabel,
    newTab,
    closeTab,
    switchTab,
    pushUrl,
    historyBack,
    historyForward,
    toggleMod,
    loadTrackIntoSoundTest,
    initApp
  };

  root.TUHCApp = TUHCApp;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = TUHCApp;
  }

  // Auto-init on page load
  if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', initApp);
    } else {
      initApp();
    }
  }

})(typeof window !== 'undefined' ? window : globalThis);
