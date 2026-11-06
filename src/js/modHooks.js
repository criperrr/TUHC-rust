/**
 * The Unofficial Homestuck Collection (Tauri 2.0 Edition)
 * Milestone 4 & 3: Community Mod Engine, VueHooks & Pesterlog Localization (F17, F21)
 *
 * Exports window.TUHCModHooks (browser global) and CommonJS module.exports (Node/test environment).
 */
(function (root) {
  'use strict';

  // ---------------------------------------------------------------------------
  // Mod API Sandbox & Throttled Logger Guard (F17)
  // ---------------------------------------------------------------------------

  /**
   * Creates a safe sandbox `api` object conforming to the Collection mod standard,
   * with a rate-limited/no-op logger and throttled console guard so legacy mods
   * with 7,000-iteration loops (such as Homestuck PT-BR logging every page patch)
   * never saturate IPC channels or freeze WebKitGTK.
   *
   * @param {object} [options={}] - Optional sandbox configuration
   * @param {number} [options.maxLogs=25] - Maximum number of logs before silencing
   * @param {object} [options.jsonStore={}] - Pre-loaded JSON files keyed by relative path
   * @param {object} [options.textStore={}] - Pre-loaded text files keyed by relative path
   * @returns {object} Sandbox environment with { api, installConsoleThrottle, runWithThrottledConsole }
   */
  function createModApiSandbox(options) {
    const opts = options || {};
    const maxLogs = typeof opts.maxLogs === 'number' ? opts.maxLogs : 25;
    const jsonStore = opts.jsonStore || {};
    const textStore = opts.textStore || {};

    let callCount = 0;
    let suppressedCount = 0;

    function safeLog(level, ...args) {
      callCount += 1;
      if (callCount <= maxLogs) {
        if (typeof console !== 'undefined' && typeof console[level] === 'function') {
          console[level](...args);
        }
      } else {
        suppressedCount += 1;
        if (callCount === maxLogs + 1) {
          if (typeof console !== 'undefined' && typeof console.warn === 'function') {
            console.warn(`[TUHCModHooks] Rate-limiting excessive mod logs (> ${maxLogs} calls). Further logs suppressed.`);
          }
        }
      }
    }

    const logger = {
      info: (...args) => safeLog('info', ...args),
      log: (...args) => safeLog('log', ...args),
      warn: (...args) => safeLog('warn', ...args),
      error: (...args) => safeLog('error', ...args),
      debug: (...args) => safeLog('debug', ...args),
      getStats: () => ({ totalCalls: callCount, suppressedCalls: suppressedCount }),
      resetStats: () => {
        callCount = 0;
        suppressedCount = 0;
      }
    };

    function readJson(relPath) {
      if (!relPath || typeof relPath !== 'string') return {};
      const clean = relPath.replace(/^\.\//, '').trim();
      if (jsonStore[clean]) return jsonStore[clean];
      if (jsonStore[relPath]) return jsonStore[relPath];
      return {};
    }

    function readJsonAsync(relPath) {
      return Promise.resolve(readJson(relPath));
    }

    function readText(relPath) {
      if (!relPath || typeof relPath !== 'string') return '';
      const clean = relPath.replace(/^\.\//, '').trim();
      if (textStore[clean]) return textStore[clean];
      if (textStore[relPath]) return textStore[relPath];
      return '';
    }

    function readTextAsync(relPath) {
      return Promise.resolve(readText(relPath));
    }

    const api = {
      logger,
      readJson,
      readJsonAsync,
      readText,
      readTextAsync
    };

    /**
     * Temporarily throttles global console methods.
     * Returns a restore function that reinstates the original console methods.
     */
    function installConsoleThrottle(customLimit = maxLogs) {
      if (typeof console === 'undefined') return () => {};
      const origLog = console.log;
      const origInfo = console.info;
      const origWarn = console.warn;
      const origError = console.error;
      const origDebug = console.debug;

      let loopCount = 0;

      function throttledMethod(orig, ...args) {
        loopCount += 1;
        if (loopCount <= customLimit) {
          orig.apply(console, args);
        } else if (loopCount === customLimit + 1) {
          origWarn.call(console, `[TUHCModHooks] Suppressing console output (> ${customLimit} statements in loop)...`);
        }
      }

      console.log = (...args) => throttledMethod(origLog, ...args);
      console.info = (...args) => throttledMethod(origInfo, ...args);
      console.debug = (...args) => throttledMethod(origDebug, ...args);
      console.warn = (...args) => throttledMethod(origWarn, ...args);
      console.error = (...args) => throttledMethod(origError, ...args);

      return function restore() {
        console.log = origLog;
        console.info = origInfo;
        console.debug = origDebug;
        console.warn = origWarn;
        console.error = origError;
      };
    }

    /**
     * Runs a function while protecting the runtime from console flooding.
     */
    function runWithThrottledConsole(fn, customLimit) {
      const restore = installConsoleThrottle(customLimit);
      try {
        return fn(api);
      } finally {
        restore();
      }
    }

    return {
      api,
      installConsoleThrottle,
      runWithThrottledConsole
    };
  }

  // ---------------------------------------------------------------------------
  // Pesterlog Button Text & VueHooks Localization (F21)
  // ---------------------------------------------------------------------------

  /**
   * Computes the reactive pesterlog toggle button text according to active mods and vueHooks.
   *
   * Canonical English:
   *   "Show pesterlog" / "Hide pesterlog"
   *   "Show tricksterlog" / "Hide tricksterlog"
   *   "Show serious business" / "Hide serious business"
   *   "P4SSWORD H1NT"
   *
   * Homestuck PT-BR (or active pageText vueHook):
   *   "Abrir pesterlog" / "Fechar pesterlog"
   *   "Abrir tricksterlog" / "Fechar tricksterlog"
   *   "Abrir serious business" / "Fechar serious business"
   *   "P4SSWORD H1NT"
   *
   * @param {string} rawContent - Raw story page text content containing `|...|` header
   * @param {boolean} logHidden - True if pesterlog is currently collapsed/hidden, false if expanded
   * @param {Array|Set|object|boolean} [enabledMods] - List of enabled mod IDs/descriptors, or boolean flag
   * @param {Array} [modHooksList] - Structured VueHook descriptors from backend
   * @returns {string} Formatted button label
   */
  function getLogButtonText(rawContent, logHidden, enabledMods, modHooksList) {
    const contentStr = typeof rawContent === 'string' ? rawContent : '';
    const isHidden = Boolean(logHidden);

    // 1. Determine if PT-BR or a localized pageText hook is active
    let isPtBrActive = false;
    let customOpenPrefix = null;
    let customClosePrefix = null;

    if (enabledMods === true) {
      isPtBrActive = true;
    } else if (typeof enabledMods === 'string') {
      isPtBrActive = /pt[-_]?br|homestuck\s*pt/i.test(enabledMods);
    } else if (Array.isArray(enabledMods)) {
      isPtBrActive = enabledMods.some((m) => {
        if (typeof m === 'string') return /pt[-_]?br|homestuck\s*pt/i.test(m);
        if (m && m.enabled) {
          const id = String(m.id || '');
          const title = String(m.title || '');
          return /pt[-_]?br|homestuck\s*pt/i.test(id) || /pt[-_]?br|homestuck\s*pt/i.test(title);
        }
        return false;
      });
    } else if (enabledMods && typeof enabledMods === 'object') {
      isPtBrActive = Boolean(
        enabledMods['Homestuck PT-BR'] ||
        enabledMods['ptbr'] ||
        enabledMods['pt-br'] ||
        enabledMods['Homestuck-PT-BR']
      );
    }

    // Inspect modHooksList for explicit pageText prefixes or computed hooks
    if (Array.isArray(modHooksList)) {
      for (let i = 0; i < modHooksList.length; i++) {
        const hook = modHooksList[i];
        if (!hook) continue;
        const matchName = hook.matchName || hook.match_name;
        if (matchName === 'pageText') {
          const open = hook.logButtonOpenPrefix || hook.log_button_open_prefix;
          const close = hook.logButtonClosePrefix || hook.log_button_close_prefix;
          if (open && close) {
            customOpenPrefix = open;
            customClosePrefix = close;
            isPtBrActive = true;
            break;
          }
          if (typeof hook.logButtonText === 'function') {
            try {
              const res = hook.logButtonText.call({ content: contentStr, logHidden: isHidden });
              if (typeof res === 'string' && res) return res;
            } catch (e) {}
          }
        }
      }
    }

    // 2. Select state prefix
    let statePrefix = '';
    if (isPtBrActive) {
      const open = customOpenPrefix || 'Abrir ';
      const close = customClosePrefix || 'Fechar ';
      statePrefix = isHidden ? open : close;
    } else {
      statePrefix = isHidden ? 'Show ' : 'Hide ';
    }

    // 3. Extract log delimiter tag (e.g. |PESTERLOG|, |TRKSTRLOG|, |SRIOUSBIZ|, |P4SSWORD H1NT|)
    const match = contentStr.match(/\|(.*?)\|/);
    let text = match ? match[1].trim() : 'PESTERLOG';

    // 4. Transform label according to MSPA and PT-BR rules
    if (/P4SSWORD H1NT/i.test(text)) {
      // P4SSWORD H1NT is returned verbatim without Show/Hide or Abrir/Fechar
      return text;
    }
    if (/trkstrlog/i.test(text)) {
      text = 'tricksterlog';
    } else if (/sriousbiz/i.test(text)) {
      text = 'serious business';
    }

    return statePrefix + text.toLowerCase();
  }

  // ---------------------------------------------------------------------------
  // Helper Utilities
  // ---------------------------------------------------------------------------

  /**
   * Strips the leading `|...|` log header delimiter from story page content.
   *
   * @param {string} content - Raw page text
   * @returns {string} Clean page text without the header tag
   */
  function stripLogHeader(content) {
    if (!content || typeof content !== 'string') return '';
    return content.replace(/^\s*\|.*?\|\s*(?:<br\s*\/?>)?/i, '');
  }

  /**
   * Returns true if the page content starts with a pesterlog delimiter.
   *
   * @param {string} content - Raw page text
   * @returns {boolean}
   */
  function isLogContent(content) {
    if (!content || typeof content !== 'string') return false;
    return /\|.*?\|/.test(content);
  }

  /**
   * Extracts the raw tag name from a pesterlog delimiter (e.g. 'PESTERLOG', 'TRKSTRLOG').
   *
   * @param {string} content - Raw page text
   * @returns {string} Tag name or empty string
   */
  function extractLogType(content) {
    if (!content || typeof content !== 'string') return '';
    const match = content.match(/\|(.*?)\|/);
    return match ? match[1].trim() : '';
  }

  // ---------------------------------------------------------------------------
  // Public API Export
  // ---------------------------------------------------------------------------

  const TUHCModHooks = {
    createModApiSandbox,
    getLogButtonText,
    stripLogHeader,
    isLogContent,
    extractLogType
  };

  if (typeof root !== 'undefined') {
    root.TUHCModHooks = TUHCModHooks;
  }
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = TUHCModHooks;
  }
})(typeof window !== 'undefined' ? window : globalThis);
