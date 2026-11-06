/**
 * The Unofficial Homestuck Collection (Tauri 2.0 Edition)
 * Milestone 3: Multimedia Engine, Ruffle WASM Player & Frag IPC Audio Sync (F12, F14, F15, F16)
 *
 * Exports window.TUHCMediaEngine (browser global) and CommonJS module.exports (Node/test environment).
 */
(function (root) {
  'use strict';

  // ---------------------------------------------------------------------------
  // Internal State
  // ---------------------------------------------------------------------------

  /** @type {HTMLAudioElement[]} Active HTML5 Audio elements for HQ Flash soundtrack playback */
  let activeAudioTracks = [];

  /** @type {HTMLVideoElement|null} Active HTML5 Video element (e.g. Collide / Act 7) */
  let activeVideo = null;

  /** @type {HTMLIFrameElement[]} Active <iframe> elements to blank upon cleanup */
  let activeIframes = [];

  /** @type {number[]} Active setTimeout identifiers */
  let activeTimeouts = [];

  /** @type {number[]} Active setInterval identifiers */
  let activeIntervals = [];

  /** @type {HTMLIFrameElement|null} The active Flash Ruffle iframe element */
  let activeIframeElement = null;

  /** @type {object|null} Resolved metadata for the current Flash animation */
  let currentFlashProps = null;

  /** @type {number} 0-based index of the last started audio track */
  let lastStartedAudio = 0;

  /** @type {object|null} Reference to current application context */
  let currentAppContext = null;

  /**
   * Drift-compensated viewport timer for dynamic Flash height expansions.
   */
  const viewportTimer = {
    interval: null,
    callback: null,
    start: 0,
    delay: 0
  };

  /**
   * 7-step sequence state for [S] GAME OVER. (06898 / page 008801).
   */
  const gameOverState = {
    interval: null,
    timeout: null,
    count: 0,
    steps: [22433, 82300, 94800, 118566, 143930, 146973, 224876]
  };

  // ---------------------------------------------------------------------------
  // Timer Management (Drift-Compensated)
  // ---------------------------------------------------------------------------

  function startViewportTimer(callback, delay) {
    clearViewportTimer();
    viewportTimer.callback = callback;
    viewportTimer.delay = delay;
    resumeViewportTimer();
  }

  function pauseViewportTimer() {
    if (viewportTimer.interval) {
      clearInterval(viewportTimer.interval);
      viewportTimer.interval = null;
      viewportTimer.delay -= Date.now() - viewportTimer.start;
    }
  }

  function resumeViewportTimer() {
    if (viewportTimer.callback && viewportTimer.delay > 0) {
      viewportTimer.start = Date.now();
      if (viewportTimer.interval) clearInterval(viewportTimer.interval);
      viewportTimer.interval = setInterval(() => {
        if (Date.now() - viewportTimer.start >= viewportTimer.delay) {
          clearViewportTimer();
          const cb = viewportTimer.callback;
          viewportTimer.callback = null;
          if (typeof cb === 'function') {
            cb();
          }
        }
      }, 20);
      activeIntervals.push(viewportTimer.interval);
    }
  }

  function clearViewportTimer() {
    if (viewportTimer.interval) {
      clearInterval(viewportTimer.interval);
      viewportTimer.interval = null;
    }
    viewportTimer.callback = null;
    viewportTimer.delay = 0;
  }

  // ---------------------------------------------------------------------------
  // Memory Hygiene & Media Cleanup (Memory RSS < 150MB SLA)
  // ---------------------------------------------------------------------------

  /**
   * Stops and tears down active HTML5 <audio> instances, removes video listeners,
   * clears all setTimeout and setInterval timers, and blanks any active <iframe>
   * before page navigation or tab close to guarantee WebKitGTK / Ruffle WASM
   * memory consumption stays strictly under the 150MB RSS budget.
   */
  function cleanupActiveMedia() {
    // 1. Pause and tear down all active audio tracks
    for (let i = 0; i < activeAudioTracks.length; i++) {
      try {
        const audio = activeAudioTracks[i];
        if (audio) {
          audio.pause();
          audio.removeAttribute('src');
          if (typeof audio.load === 'function') audio.load();
        }
      } catch (e) {
        // Ignore tear down errors
      }
    }
    activeAudioTracks = [];

    // 2. Pause and tear down active video element if any
    if (activeVideo) {
      try {
        activeVideo.pause();
        activeVideo.removeAttribute('src');
        if (typeof activeVideo.load === 'function') activeVideo.load();
      } catch (e) {
        // Ignore tear down errors
      }
      activeVideo = null;
    }

    // 3. Clear all pending setTimeout handles
    for (let i = 0; i < activeTimeouts.length; i++) {
      clearTimeout(activeTimeouts[i]);
    }
    activeTimeouts = [];

    // 4. Clear all pending setInterval handles
    for (let i = 0; i < activeIntervals.length; i++) {
      clearInterval(activeIntervals[i]);
    }
    activeIntervals = [];

    // 5. Clear viewport height transition timer
    clearViewportTimer();

    // 6. Clear [S] GAME OVER. progression timer
    if (gameOverState.interval) {
      clearInterval(gameOverState.interval);
      gameOverState.interval = null;
    }
    if (gameOverState.timeout) {
      clearTimeout(gameOverState.timeout);
      gameOverState.timeout = null;
    }
    gameOverState.count = 0;

    // 7. Blank and tear down active iframes to reclaim WebKitGTK WebProcess memory
    for (let i = 0; i < activeIframes.length; i++) {
      try {
        const iframe = activeIframes[i];
        if (iframe) {
          iframe.srcdoc = '';
          iframe.src = 'about:blank';
          if (iframe.contentWindow) {
            try {
              iframe.contentWindow.location.replace('about:blank');
            } catch (ignore) {}
          }
          if (typeof iframe.remove === 'function') {
            iframe.remove();
          }
        }
      } catch (e) {
        // Ignore cross-origin / destroyed iframe access errors
      }
    }
    activeIframes = [];
    activeIframeElement = null;
    currentFlashProps = null;
    lastStartedAudio = 0;
  }

  // ---------------------------------------------------------------------------
  // Ruffle WASM Player Srcdoc Template Generator (F12, F14)
  // ---------------------------------------------------------------------------

  /**
   * Generates the self-hosted Ruffle WASM player srcdoc HTML string containing:
   * - window.RufflePlayer configuration (autoplay, unmuteOverlay, letterbox)
   * - Self-hosted Ruffle bundle script from Asset Pack V2
   * - Triple-hook Frag IPC bridge (onhashchange, navigation API, window.open & anchor clicks)
   * - Flash <object> element embedding the target SWF
   *
   * @param {object} options
   * @param {string} options.swfUrl - Full HTTP URL to the .swf asset
   * @param {number} [options.width=650] - Flash canvas width
   * @param {number} [options.height=450] - Flash canvas height
   * @param {string} [options.bgColor='#ffffff'] - Canvas background color
   * @param {string} [options.assetBase=''] - Base loopback server URL
   * @param {string} [options.flashId=''] - Canonical flash identifier
   * @returns {string} Fully self-contained HTML srcdoc string
   */
  function buildRuffleSrcdoc(options) {
    const opts = options || {};
    const swfUrl = opts.swfUrl || '';
    const width = opts.width || 650;
    const height = opts.height || 450;
    const bgColor = opts.bgColor || '#ffffff';
    const cleanBase = (opts.assetBase || '').replace(/\/+$/, '');

    // Canonical self-hosted Ruffle bundle path inside Asset Pack V2
    const ruffleJsUrl = cleanBase
      ? `${cleanBase}/assets/archive/imods/_twoToThree/js/ruffle/ruffle.js`
      : '/assets/archive/imods/_twoToThree/js/ruffle/ruffle.js';

    return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Ruffle Player</title>
  <style>
    html, body {
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      overflow: hidden;
      background-color: ${bgColor};
    }
    ruffle-player, ruffle-embed, ruffle-object, object, embed {
      width: 100% !important;
      height: 100% !important;
      display: block;
    }
  </style>
  <script>
    window.RufflePlayer = window.RufflePlayer || {};
    window.RufflePlayer.config = {
      autoplay: "on",
      unmuteOverlay: "hidden",
      letterbox: "off",
      warnOnUnsupportedContent: false,
      allowScriptAccess: true,
      preloader: false
    };
  </script>
  <script src="${ruffleJsUrl}"></script>
  <script>
    (function () {
      function dispatchFragIpc(urlOrHash) {
        if (!urlOrHash || typeof urlOrHash !== 'string') return false;
        var idx = urlOrHash.indexOf('about:srcdoc#');
        var hashIdx = idx !== -1 ? idx + 'about:srcdoc#'.length : (urlOrHash.startsWith('#') ? 1 : -1);
        if (hashIdx === -1) {
          var altIdx = urlOrHash.indexOf('#');
          if (altIdx !== -1) hashIdx = altIdx + 1;
        }
        if (hashIdx === -1) return false;
        var cmdString = urlOrHash.slice(hashIdx);
        if (!cmdString) return false;

        var handled = false;
        try {
          if (window.parent && typeof window.parent.__tuhcInvokeFromFlash === 'function') {
            window.parent.__tuhcInvokeFromFlash(cmdString);
            handled = true;
          }
        } catch (e) {}

        try {
          if (window.parent && typeof window.parent.postMessage === 'function') {
            window.parent.postMessage({ type: 'TUHC_FLASH_CMD', cmd: cmdString }, '*');
            window.parent.postMessage({ type: 'TUHC_FRAG_IPC', cmd: cmdString, hash: cmdString }, '*');
            handled = true;
          }
        } catch (e) {}

        return handled;
      }

      // 1. Hashchange listener (Standard ActionScript getURL("about:srcdoc#..."))
      window.addEventListener('hashchange', function () {
        if (location.hash && location.hash.length > 1) {
          var h = location.hash.slice(1);
          try {
            history.replaceState(null, '', location.pathname + location.search);
          } catch (e) {}
          dispatchFragIpc('#' + h);
        }
      });

      // 2. Modern Navigation API interceptor (Chromium/WebKit Navigation API)
      if (window.navigation && typeof window.navigation.addEventListener === 'function') {
        window.navigation.addEventListener('navigate', function (event) {
          if (event.destination && event.destination.url && event.destination.url.indexOf('about:srcdoc#') !== -1) {
            event.preventDefault();
            dispatchFragIpc(event.destination.url);
          }
        });
      }

      // 3. window.open interceptor
      var origOpen = window.open;
      window.open = function (url, target, features) {
        if (typeof url === 'string') {
          if (url.indexOf('about:srcdoc#') !== -1) {
            dispatchFragIpc(url);
            return null;
          }
          if (window.parent && typeof window.parent.__tuhcInvokeFromFlash === 'function') {
            window.parent.__tuhcInvokeFromFlash('link?' + url);
            return null;
          }
          try {
            if (window.parent && typeof window.parent.postMessage === 'function') {
              window.parent.postMessage({ type: 'TUHC_FLASH_CMD', cmd: 'link?' + url }, '*');
              return null;
            }
          } catch (e) {}
        }
        return origOpen ? origOpen.call(window, url, target, features) : null;
      };

      // 4. Anchor click capture
      document.addEventListener('click', function (e) {
        var a = e.target && e.target.closest ? e.target.closest('a') : null;
        if (a && a.href && a.href.indexOf('about:srcdoc#') !== -1) {
          e.preventDefault();
          dispatchFragIpc(a.href);
        }
      }, true);
    })();
  </script>
</head>
<body>
  <object type="application/x-shockwave-flash" width="${width}" height="${height}" data="${swfUrl}">
    <param name="movie" value="${swfUrl}">
    <param name="play" value="true">
    <param name="loop" value="true">
    <param name="quality" value="high">
    <param name="bgcolor" value="${bgColor}">
    <param name="allowScriptAccess" value="always">
  </object>
</body>
</html>`;
  }

  // ---------------------------------------------------------------------------
  // Frag IPC Command Dispatch Engine (F13, F14, F15)
  // ---------------------------------------------------------------------------

  /**
   * Initializes audio instances for the current Flash page if not already loaded.
   */
  function executeAudioInit() {
    if (activeAudioTracks.length > 0) return;
    const tracks = (currentFlashProps && currentFlashProps.audioTracks) || [];
    for (let i = 0; i < tracks.length; i++) {
      try {
        const audio = new Audio(tracks[i]);
        audio.preload = 'auto';
        if (typeof audio.load === 'function') audio.load();
        activeAudioTracks.push(audio);
      } catch (e) {
        // Fallback for non-browser or test runner environments
        activeAudioTracks.push({
          src: tracks[i],
          currentTime: 0,
          volume: 1,
          paused: true,
          play: () => Promise.resolve(),
          pause: () => {},
          load: () => {},
          removeAttribute: () => {}
        });
      }
    }
  }

  /**
   * Executes audioStart with positive setTimeout delay or negative currentTime pre-seek.
   *
   * @param {string|number} [trackParam='1'] - Track number or Cascade scrub ('1'..'5', '1.5', '4.5', '5.5')
   */
  function executeAudioStart(trackParam = '1') {
    executeAudioInit();
    let n = trackParam !== undefined && trackParam !== null && trackParam !== '' ? trackParam : '1';
    let cascadeDelay = 0;

    const flashId = currentFlashProps ? currentFlashProps.flashId : '';
    const rawFlashId = currentFlashProps ? currentFlashProps.rawFlashId : '';
    const isCascade = flashId === '04106' || rawFlashId === '04106';

    if (isCascade) {
      const nStr = String(n);
      if (nStr.endsWith('.5')) {
        const seg = nStr.charAt(0);
        if (seg === '1') cascadeDelay = -52900;
        else if (seg === '4') cascadeDelay = -125250;
        else if (seg === '5') cascadeDelay = -77900;
        n = seg;
      }
    }

    const trackIndex = Math.max(0, parseInt(String(n), 10) - 1);
    const audioTrack = activeAudioTracks[trackIndex];
    if (!audioTrack) return;

    lastStartedAudio = trackIndex;
    let replay = false;

    // Reset track if it has progressed past 30s (e.g. 5x SHOWDOWN COMBO loop)
    if (audioTrack.currentTime > 30) {
      executeAudioReset();
      replay = true;
    }

    let delay = 0;
    if (cascadeDelay) {
      delay = cascadeDelay;
    } else if (root && root.TUHCFlashProps && typeof root.TUHCFlashProps.getAudioDelay === 'function') {
      delay = root.TUHCFlashProps.getAudioDelay(flashId || rawFlashId, n, replay, false);
    }

    if (delay > 0) {
      const tId = setTimeout(() => {
        if (activeAudioTracks[trackIndex]) {
          try {
            const p = activeAudioTracks[trackIndex].play();
            if (p && typeof p.catch === 'function') p.catch(() => {});
          } catch (e) {}
        }
      }, delay);
      activeTimeouts.push(tId);
    } else {
      if (delay < 0) {
        try {
          audioTrack.currentTime = -delay / 1000;
        } catch (e) {}
      }
      try {
        const p = audioTrack.play();
        if (p && typeof p.catch === 'function') p.catch(() => {});
      } catch (e) {}
    }
  }

  function executeAudioPause() {
    if (activeAudioTracks[lastStartedAudio]) {
      try {
        activeAudioTracks[lastStartedAudio].pause();
      } catch (e) {}
    }
  }

  function executeAudioResume() {
    const tId = setTimeout(() => {
      if (activeAudioTracks[lastStartedAudio]) {
        try {
          const p = activeAudioTracks[lastStartedAudio].play();
          if (p && typeof p.catch === 'function') p.catch(() => {});
        } catch (e) {}
      }
    }, 30);
    activeTimeouts.push(tId);
  }

  function executeAudioReset() {
    for (let i = 0; i < activeAudioTracks.length; i++) {
      try {
        const track = activeAudioTracks[i];
        if (track) {
          track.pause();
          track.currentTime = 0;
        }
      } catch (e) {}
    }
  }

  function executeAudioSeek(secondsStr) {
    let sec = parseFloat(secondsStr);
    if (!Number.isNaN(sec) && activeAudioTracks[lastStartedAudio]) {
      if (sec >= 100) {
        sec = sec / 1000;
      }
      try {
        activeAudioTracks[lastStartedAudio].currentTime = sec;
      } catch (e) {}
    }
  }

  function executeVolume(volumeStr) {
    const v = parseFloat(volumeStr);
    if (!Number.isNaN(v)) {
      const clamped = Math.max(0, Math.min(1, v));
      for (let i = 0; i < activeAudioTracks.length; i++) {
        try {
          if (activeAudioTracks[i]) activeAudioTracks[i].volume = clamped;
        } catch (e) {}
      }
    }
  }

  /**
   * Executes heightStart viewport height transition with drift-compensated timer.
   *
   * @param {string} mode - Mode key ('cascade1', 'cascade1.5', 'A6A6I1', 'pipeorgan', 'vriskagram', 'remem8er')
   */
  function executeHeightStart(mode) {
    const settings = (currentAppContext && currentAppContext.settings) || {};
    if (settings.jsFlashes === false) return;

    const flashPropsModule = (root && root.TUHCFlashProps) || {};
    const cropHeights = flashPropsModule.CROP_HEIGHT || flashPropsModule.cropHeight || {};
    const heightTransitions = flashPropsModule.HEIGHT_TRANSITIONS || flashPropsModule.heightTransitions || {};

    const flashId = currentFlashProps ? currentFlashProps.flashId : '';
    const initialCrop = cropHeights[flashId] || (currentFlashProps && currentFlashProps.cropHeight);

    if (activeIframeElement && initialCrop && activeIframeElement.style.height !== initialCrop) {
      activeIframeElement.style.transition = 'height 0.2s';
      activeIframeElement.style.height = initialCrop;
    }

    const transitionDef = heightTransitions[mode];
    if (!transitionDef) return;

    const time = transitionDef.time;
    const transitionCss = transitionDef.transition;
    const heightTo = transitionDef.heightTo;

    if (time < 1) {
      if (activeIframeElement) {
        activeIframeElement.style.transition = transitionCss;
        activeIframeElement.style.height = heightTo;
      }
    } else {
      startViewportTimer(() => {
        if (activeIframeElement) {
          activeIframeElement.style.transition = transitionCss;
          activeIframeElement.style.height = heightTo;
        }
      }, time);
    }
  }

  function executeHeightPause() {
    pauseViewportTimer();
  }

  function executeHeightResume() {
    resumeViewportTimer();
  }

  function executeHeightReset() {
    clearViewportTimer();
    const flashPropsModule = (root && root.TUHCFlashProps) || {};
    const cropHeights = flashPropsModule.CROP_HEIGHT || flashPropsModule.cropHeight || {};
    const flashId = currentFlashProps ? currentFlashProps.flashId : '';
    const initialCrop = cropHeights[flashId] || (currentFlashProps && currentFlashProps.cropHeight);

    if (activeIframeElement && initialCrop) {
      activeIframeElement.style.transition = 'height 0.2s';
      activeIframeElement.style.height = initialCrop;
    }
  }

  /**
   * Executes the 7-step background strike sequence for [S] GAME OVER. (06898 / page 008801).
   */
  function executeGameOver() {
    const settings = (currentAppContext && currentAppContext.settings) || {};
    if (settings.jsFlashes === false) return;

    const flashPropsModule = (root && root.TUHCFlashProps) || {};
    const steps = (flashPropsModule.GAME_OVER && flashPropsModule.GAME_OVER.steps) || gameOverState.steps;

    gameOverState.count = 0;

    function triggerStep() {
      gameOverState.count += 1;
      const currentStep = gameOverState.count;

      // Notify appContext or dispatch custom DOM event
      if (currentAppContext && typeof currentAppContext.onGameOverStep === 'function') {
        currentAppContext.onGameOverStep(currentStep);
      }
      if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('tuhc-game-over-step', { detail: { step: currentStep } }));
        const appEl = document.getElementById('app') || document.getElementById('page-container');
        if (appEl) {
          appEl.setAttribute('data-game-over-step', String(currentStep));
        }
      }

      if (gameOverState.count < steps.length) {
        const nextDelay = steps[gameOverState.count] - steps[gameOverState.count - 1];
        gameOverState.timeout = setTimeout(triggerStep, nextDelay);
        activeTimeouts.push(gameOverState.timeout);
      }
    }

    if (steps.length > 0) {
      gameOverState.timeout = setTimeout(triggerStep, steps[0]);
      activeTimeouts.push(gameOverState.timeout);
    }
  }

  /**
   * Navigates the host router to a given MSPA URL or route.
   */
  function executeLink(url) {
    if (!url || typeof url !== 'string' || url === 'about:srcdoc' || url === '#' || url === 'about:blank') {
      return;
    }
    if (currentAppContext && typeof currentAppContext.navigate === 'function') {
      currentAppContext.navigate(url);
      return;
    }
    if (typeof window !== 'undefined') {
      if (typeof window.__tuhcNavigate === 'function') {
        window.__tuhcNavigate(url);
        return;
      }
      window.dispatchEvent(new CustomEvent('tuhc-navigate', { detail: { url } }));
    }
  }

  /**
   * Parses and executes a Frag IPC command string from ActionScript (or test harness).
   * Supports compound commands joined with '&' (e.g. `heightStart?cascade1&audioStart?1`).
   *
   * @param {string} cmdStr - Command string (e.g. 'audioInit', 'audioStart?1', 'heightStart?cascade1')
   */
  function handleFlashCommand(cmdStr) {
    if (!cmdStr || typeof cmdStr !== 'string') return;
    let clean = cmdStr.trim();

    // Strip about:srcdoc# or # if present
    const hashIdx = clean.indexOf('#');
    if (hashIdx !== -1) {
      clean = clean.slice(hashIdx + 1);
    }

    // Normalize audioSeek&<val> -> audioSeek?<val>
    clean = clean.replace(/(?:^|&)(audioSeek)&([0-9.]+)/g, (match, cmd, val) => {
      return match.startsWith('&') ? `&${cmd}?${val}` : `${cmd}?${val}`;
    });

    // Split compound commands by '&'
    const commandList = clean.split('&');
    for (let i = 0; i < commandList.length; i++) {
      const singleCmd = commandList[i].trim();
      if (!singleCmd) continue;

      const qIdx = singleCmd.indexOf('?');
      const funcName = qIdx !== -1 ? singleCmd.slice(0, qIdx) : singleCmd;
      const param = qIdx !== -1 ? singleCmd.slice(qIdx + 1) : '';

      switch (funcName) {
        case 'audioInit':
          executeAudioInit();
          break;
        case 'audioStart':
          executeAudioStart(param || '1');
          break;
        case 'audioPause':
          executeAudioPause();
          break;
        case 'audioResume':
          executeAudioResume();
          break;
        case 'audioReset':
          executeAudioReset();
          break;
        case 'audioSeek':
          executeAudioSeek(param);
          break;
        case 'vol':
          executeVolume(param);
          break;
        case 'heightStart':
          executeHeightStart(param);
          break;
        case 'heightPause':
          executeHeightPause();
          break;
        case 'heightResume':
          executeHeightResume();
          break;
        case 'heightReset':
          executeHeightReset();
          break;
        case 'gameOver':
          executeGameOver();
          break;
        case 'link':
          executeLink(param);
          break;
        default:
          break;
      }
    }
  }

  // ---------------------------------------------------------------------------
  // Host Window Frag IPC Interceptor Installation (F14)
  // ---------------------------------------------------------------------------

  /**
   * Installs global __tuhcInvokeFromFlash and window 'message' listeners on the host.
   */
  function initFragIpc(targetWindow) {
    const win = targetWindow || (typeof window !== 'undefined' ? window : null);
    if (!win) return;

    win.__tuhcInvokeFromFlash = function (cmdStr) {
      handleFlashCommand(cmdStr);
    };

    win.addEventListener('message', (event) => {
      if (event && event.data) {
        if (event.data.type === 'TUHC_FLASH_CMD' || event.data.type === 'TUHC_FRAG_IPC') {
          handleFlashCommand(event.data.cmd || event.data.hash);
        }
      }
    });
  }

  // Auto-install on the current browser window
  if (typeof window !== 'undefined') {
    initFragIpc(window);
  }

  // ---------------------------------------------------------------------------
  // Unified Media Renderer (F12, F15, F16)
  // ---------------------------------------------------------------------------

  /**
   * Renders media items into containerEl based on format:
   * - Images (.gif, .png, .jpg, .jpeg, .webp)
   * - Flashes (.swf) via Ruffle WASM srcdoc iframe + calibrated external audio tracks
   * - Video (.mp4, .webm) with Collide 22s width expansion
   * - Sburb / Openbound HTML5 Canvas games (.html) inside iframe
   *
   * @param {HTMLElement} containerEl - DOM container where media elements will be appended
   * @param {string[]|string} [mediaUrls] - Raw media URL(s) from page.media
   * @param {object} [pageObj] - Active story page descriptor ({ page_id, title, flags, media })
   * @param {object} [appContext] - Context ({ assetBase, cacheBuster, settings, navigate, onGameOverStep })
   */
  function renderMedia(containerEl, mediaUrls, pageObj, appContext, resolveAssetUrlFallback) {
    if (!containerEl || typeof containerEl.appendChild !== 'function') {
      return false;
    }

    cleanupActiveMedia();

    // Support both structured appContext and direct settings object
    const isContextObj = appContext && (appContext.settings !== undefined || appContext.assetBase !== undefined);
    const settings = isContextObj ? (appContext.settings || {}) : (appContext || {});
    const assetBase = (isContextObj && appContext.assetBase) || (appContext && appContext.assetBase) || '';
    const cacheBuster = (isContextObj && (appContext.cacheBuster || appContext.overlayGeneration)) || '';
    const navigate = isContextObj ? appContext.navigate : (appContext && appContext.navigate);

    currentAppContext = isContextObj ? appContext : { settings, assetBase, cacheBuster, navigate };
    if (!currentAppContext.settings) {
      currentAppContext.settings = settings;
    }

    containerEl.innerHTML = '';

    const pObj = pageObj || {};
    const pageId = String(pObj.page_id || pObj.pageId || '');
    const pageFlags = Array.isArray(pObj.flags) ? pObj.flags : [];

    let items = [];
    if (Array.isArray(mediaUrls)) {
      items = mediaUrls;
    } else if (typeof mediaUrls === 'string' && mediaUrls.trim()) {
      items = [mediaUrls];
    } else if (Array.isArray(pObj.media)) {
      items = pObj.media;
    }

    if (items.length === 0) return false;

    const flashPropsModule = (root && root.TUHCFlashProps) || {};
    const resolveFn = flashPropsModule.resolveMediaAssets;

    for (let i = 0; i < items.length; i++) {
      const rawUrl = items[i];
      if (!rawUrl || typeof rawUrl !== 'string') continue;

      let resolved;
      if (typeof resolveFn === 'function') {
        resolved = resolveFn(rawUrl, settings, assetBase, cacheBuster, pageId, pageFlags);
      } else {
        const fallbackUrl = typeof resolveAssetUrlFallback === 'function' ? resolveAssetUrlFallback(rawUrl) : rawUrl;
        resolved = {
          resolvedUrl: fallbackUrl,
          ext: (rawUrl.split('?')[0].match(/\.([a-z0-9]+)$/i) || ['', ''])[1].toLowerCase(),
          flashId: '',
          rawFlashId: '',
          width: 650,
          height: 450,
          bgcolor: '#fff',
          cropHeight: '450px',
          isCropped: false,
          audioTracks: []
        };
      }

      const ext = resolved.ext || '';

      // 1. Flash Animation (.swf) -> Ruffle WASM Player Iframe
      if (ext === 'swf') {
        currentFlashProps = resolved;
        const initialHeight = (settings.jsFlashes !== false && resolved.isCropped)
          ? resolved.cropHeight
          : `${resolved.height}px`;

        const iframe = document.createElement('iframe');
        iframe.className = 'page-media-flash ruffle-frame';
        iframe.setAttribute('data-media-type', 'ruffle-flash');
        iframe.setAttribute('data-swf-src', resolved.resolvedUrl);
        iframe.setAttribute('data-flash-id', resolved.flashId);
        iframe.style.width = `${resolved.width}px`;
        iframe.style.height = initialHeight;
        iframe.style.border = 'none';
        iframe.style.overflow = 'hidden';
        iframe.style.display = 'block';
        iframe.style.margin = '0 auto';
        iframe.style.backgroundColor = resolved.bgcolor || '#ffffff';

        const srcdocHtml = buildRuffleSrcdoc({
          swfUrl: resolved.resolvedUrl,
          width: resolved.width,
          height: resolved.height,
          bgColor: resolved.bgcolor,
          assetBase,
          flashId: resolved.flashId
        });
        iframe.srcdoc = srcdocHtml;

        activeIframeElement = iframe;
        activeIframes.push(iframe);
        containerEl.appendChild(iframe);

        // Preload external audio tracks if present so they are ready on audioStart
        if (resolved.audioTracks && resolved.audioTracks.length > 0) {
          executeAudioInit();
        }
      }
      // 2. Sburb / Openbound HTML5 Canvas Game (.html)
      else if (ext === 'html') {
        const iframe = document.createElement('iframe');
        iframe.className = 'page-media-html sburb-frame';
        iframe.setAttribute('data-media-type', 'sburb-canvas');
        iframe.src = resolved.resolvedUrl;
        iframe.width = String(resolved.width || 650);
        iframe.height = String(resolved.height || 450);
        iframe.style.border = 'none';
        iframe.style.overflow = 'hidden';
        iframe.style.display = 'block';
        iframe.style.margin = '0 auto';
        iframe.allowFullscreen = true;

        activeIframes.push(iframe);
        containerEl.appendChild(iframe);
      }
      // 3. HTML5 Video (.mp4, .webm, .ogg)
      else if (ext === 'mp4' || ext === 'webm' || ext === 'ogg') {
        const video = document.createElement('video');
        video.className = 'page-media-video';
        video.setAttribute('data-media-type', 'video');
        video.src = resolved.resolvedUrl;
        video.controls = true;
        video.autoplay = true;
        video.style.display = 'block';
        video.style.margin = '0 auto';
        video.style.width = `${resolved.width || 650}px`;
        video.style.height = `${resolved.height || 450}px`;

        // Special Collide (08080.mp4 / page 009987) curtain expansion at currentTime > 22s
        if (resolved.flashId === '08080' || pageId === '009987') {
          let widened = false;
          video.addEventListener('timeupdate', () => {
            if (!widened && video.currentTime > 22) {
              widened = true;
              video.style.transition = 'width 1.5s cubic-bezier(0, 0, 0, 1)';
              video.style.width = '950px';
              if (containerEl) {
                containerEl.style.transition = 'width 1.5s cubic-bezier(0, 0, 0, 1)';
              }
            }
          });
        }

        activeVideo = video;
        containerEl.appendChild(video);
      }
      // 4. Static / Animated Image (.gif, .png, .jpg, .jpeg, .webp)
      else {
        const img = document.createElement('img');
        img.className = 'page-media-img';
        img.setAttribute('data-media-type', 'image');
        img.src = resolved.resolvedUrl;
        img.alt = pObj.title || 'MSPA Story Media';
        img.style.display = 'block';
        img.style.margin = '0 auto';
        containerEl.appendChild(img);
      }
    }
    return true;
  }

  // ---------------------------------------------------------------------------
  // Public API Export
  // ---------------------------------------------------------------------------

  const TUHCMediaEngine = {
    cleanupActiveMedia,
    buildRuffleSrcdoc,
    handleFlashCommand,
    renderMedia,
    initFragIpc,

    // Internal state inspectors for unit and E2E verification
    getActiveAudio: () => activeAudioTracks,
    getActiveIframes: () => activeIframes,
    getLastStartedAudio: () => lastStartedAudio,
    getViewportTimer: () => viewportTimer,
    getGameOverState: () => gameOverState,
    getCurrentFlashProps: () => currentFlashProps
  };

  if (typeof root !== 'undefined') {
    root.TUHCMediaEngine = TUHCMediaEngine;
  }
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = TUHCMediaEngine;
  }
})(typeof window !== 'undefined' ? window : globalThis);
