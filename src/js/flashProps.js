/**
 * The Unofficial Homestuck Collection (Tauri 2.0 Edition)
 * Milestone 3: Flash Metadata, Audio Calibration & Viewport Timing Tables (F13, F15)
 *
 * Exports window.TUHCFlashProps (browser global) and CommonJS module.exports (Node/test environment).
 */
(function (root) {
  'use strict';

  /**
   * Flash viewport dimension and background color overrides keyed by flashId.
   * Default dimensions when not present: width = 650, height = 450, bgcolor = '#fff'.
   */
  const INDEXED_FLASH_PROPS = {
    // Crisp text boxes on early small flashes
    '00003': { width: 550 },
    '00007': { width: 550 },
    '00008': { width: 550 },
    '00018': { width: 550 },
    '00021': { width: 550 },
    '00022': { width: 550 },

    // Big flashes
    '00980': { height: 650 },
    '01070': { height: 650 },
    '01073': { height: 650 },
    '01267': { height: 650 },
    '01407': { height: 650 },
    '01641': { height: 650 },
    '01668': { height: 650 },
    '01801': { height: 650 },
    '01931': { height: 650 },
    '02577': { height: 650 },
    '02625': { height: 650 },
    '02786': { height: 650 },
    '02847': { height: 650 },
    '02926': { height: 650 },
    '02985': { height: 650 },
    '03085': { height: 650 },
    '03676': { height: 650 },
    '03692': { height: 650 },
    '03741': { height: 650 },
    '04106': { width: 950, height: 650, bgcolor: '#000' },
    '04108': { height: 650 },
    '04110': { height: 650 },
    '04272': { height: 650 },
    '04370': { width: 950, height: 650 },
    '04388': { height: 650 },
    '04483': { height: 650 },
    '04569': { height: 650 },
    '04614': { height: 650 },
    '04662': { height: 650 },
    '04812': { width: 950, height: 1160, bgcolor: '#000' },
    '04939': { height: 650 },
    '04941': { height: 650 },
    '05024': { height: 650 },
    '05235': { height: 650 },
    '05249': { height: 650 },
    '05258': { width: 950, height: 650 },
    '05260': { width: 650, height: 450 },
    '05305': { width: 650, height: 450 },
    '05395': { width: 650, height: 450 },
    '05425': { height: 650 },
    '05509': { height: 650 },
    '05595': { height: 650 },
    '05777': { height: 650 },
    '06240': { height: 650 },
    'A6A6I1': { height: 650, bgcolor: '#000' },
    '06379': { height: 650 },
    '06394': { height: 650 },
    '06398': { height: 650 },
    '06609': { height: 650 },
    '06713': { height: 650 },
    '06720': { width: 680, height: 650 },
    '06725': { height: 650 },
    '06727': { height: 650 },
    '06898': { width: 950, height: 650, bgcolor: '#042300' },
    '07083': { height: 550 },
    '07095': { width: 950, height: 650, bgcolor: '#000' },
    '07122': { width: 950, height: 650 },
    '07138': { width: 950, height: 650 },
    '07152': { width: 950, height: 650 },
    '07161': { width: 950, height: 650 },
    '07402': { width: 950, height: 650, bgcolor: '#000' },
    '07445': { width: 950, height: 650, bgcolor: '#000' },
    '07826': { height: 650 },
    '07953': { width: 800, height: 600, bgcolor: '#000' },
    '08080': { width: 650, height: 650 },
    '08120': { width: 1280, height: 720 },

    // Ryanquest
    '01_6': { width: 900, height: 600 },
    '02_5': { width: 800, height: 600 },
    '04_4': { width: 800, height: 600 },
    '06_9': { width: 800, height: 600 },
    '08_9': { width: 800, height: 600 },
    '09_12': { width: 800, height: 600 },
    '10_6': { width: 800, height: 600 },
    '14_9': { width: 800, height: 600 },
    '15_4': { width: 800, height: 600 },
    '15_5': { width: 800, height: 600 },

    // Sweet Bro and Hella Jeff
    'moviesbhj': { width: 550, height: 400 },
    'sbhjthemovie1': { width: 800, height: 600 }
  };

  /**
   * Initial cropped CSS heights for Flashes that dynamically expand during playback.
   */
  const CROP_HEIGHT = {
    '04106': '550px',
    '06240': '560px',
    'A6A6I1': '522px',
    '07095': '521px',
    '07122': '631px',
    '07445': '522px',
    '07953': '522px'
  };

  /**
   * Viewport height transition parameters triggered by `heightStart?<mode>`.
   */
  const HEIGHT_TRANSITIONS = {
    'cascade1': {
      time: 60900,
      transition: 'height 8s linear',
      heightTo: '650px'
    },
    'cascade1.5': {
      time: 8067,
      transition: 'height 8s linear',
      heightTo: '650px'
    },
    'A6A6I1': {
      time: 18833,
      transition: 'height 17.53s cubic-bezier(0.12, 0, 0.39, 0)',
      heightTo: '650px'
    },
    'pipeorgan': {
      time: 100100,
      transition: 'height 11.3s linear',
      heightTo: '650px'
    },
    'vriskagram': {
      time: 10650,
      transition: 'height 1.3s linear',
      heightTo: '650px'
    },
    'remem8er': {
      time: 16133,
      transition: 'height 10s linear',
      heightTo: '600px'
    }
  };

  /**
   * 7-step background strike progression timestamps (in ms) for `[S] GAME OVER.` (06898 / page 008801).
   */
  const GAME_OVER_STEPS = [22433, 82300, 94800, 118566, 143930, 146973, 224876];

  const GAME_OVER = {
    steps: GAME_OVER_STEPS
  };

  /**
   * Video pause thresholds (in seconds).
   */
  const PAUSE_AT = {
    '08080': 18
  };

  /**
   * Calibrated audio sync offsets in milliseconds.
   * Positive value -> delay audio.play() via setTimeout(delay).
   * Negative value -> advance audio.currentTime = -delay / 1000 and play immediately.
   */
  const AUDIO_DELAYS = {
    '00090': -250,
    '00246': -1300,
    '00383': -200,
    '00637': -50,
    '00860': -50,
    '00980_1': 1500,
    '00980_bolin_1': 3100,
    '01070': -100,
    '01070_bolin': -100,
    '01073': -65,
    '01267': -250,
    '01407': -50,
    '01641': -50,
    '01668': -50,
    '01801': -100,
    '01931': -125,
    '01940': -150,
    '01990': -250,
    '02577': -350,
    '02625': -100,
    '02786': -50,
    '02847': 4250,
    '02926': -800,
    '02985': -150,
    '03085': -150,
    '03676': -60,
    '03692': -100,
    '03741': -100,
    '04082': -75,
    '04106_1': -50,
    '04106_2': 575,
    '04106_3': 1550,
    '04106_4': 600,
    '04106_5': 1200,
    '04106_1.5': -52900,
    '04106_2.5': 575,
    '04106_3.5': 1550,
    '04106_4.5': -125250,
    '04106_5.5': -77900,
    '04108': -100,
    '04110': -33,
    '04272': -100,
    '04370_1': -100,
    '04370_2': 450,
    '04387': -300,
    '04388': 600,
    '04483': -100,
    '04569': -100,
    '04572': -50,
    '04614': -100,
    '04662': -60,
    '04812': -50,
    '04939': -50,
    '04941': -100,
    '05024': -100,
    '05235': 2000,
    '05249': -125,
    '05258': -150,
    '05258_replay': 0,
    '05419': -50,
    '05425': -33,
    '05509': -50,
    '05595': -50,
    '05777': -75,
    '06240': 1600,
    '06273': -50,
    '06274': -50,
    '06275': -50,
    '06609': -50,
    '06898': 1133,
    '07083': -1450,
    '07095': -200,
    '07402': 933,
    '07445': 2600,
    '07826': -50,
    '07953': 650,
    '10_6': -400
  };

  /**
   * 94 MSPA Page IDs (and corresponding Flash IDs) that use `_hq.swf` + external MP3 audio tracks.
   */
  const HQ_PAGE_LIST = [
    '001990', '002146', '002283', '002537', '002760', '002880', '002970', '002973',
    '003167', '003307', '003541', '003568', '003701', '003831', '003840', '004478',
    '004526', '004687', '004748', '004827', '004888', '004988', '005579', '005595',
    '005596', '005644', '005985', '006009', '006011', '006013', '006175', '006273',
    '006290', '006291', '006386', '006472', '006475', '006517', '006565', '006715',
    '006842', '006844', '006927', '007138', '007152', '007161', '007322', '007328',
    '007412', '007498', '007680', '008143', '008176', '008177', '008178', '008300',
    '008512', '008801', '008986', '008998', '009305', '009348', '009729', '009859',
    '002544', '002722', '002733', '002879', '003052', '003218', '003620', '003792',
    '003996', '004190', '004662', '004692', '004979', '005261', '005338', '005348',
    '005625', '005660', '005875', '006676', '006720', '006725', '006727', '006758',
    '007162', '007338', '008282', '008301', '008616', '008628'
  ];

  /**
   * 23 MSPA Page IDs that use `*_hqbaked.swf` (high-quality audio baked into the SWF).
   */
  const BAKED_PAGE_LIST = [
    '002050', '002153', '002288', '002376', '002551', '002730', '002838', '002921',
    '003850', '004569', '004682', '004748', '004901', '004902', '005091', '005221',
    '005618', '005644', '005981', '007595', '007677', '008297', '008301'
  ];

  /**
   * 10 MSPA Page IDs that have Bill Bolin music variants (`_bolin.swf` / `_bolin_hq.swf`).
   */
  const BOLIN_PAGE_LIST = [
    '002288', '002544', '002551', '002722', '002730',
    '002733', '002880', '002926', '002970', '003620'
  ];

  /**
   * Convert a 6-digit Homestuck page number (e.g. '002880') to its 5-digit flash ID ('00980').
   * Note: Early Homestuck flashes up to ~004478 use an offset of -1900, while later pages
   * after the missing page numbers (e.g. 006009 -> 04106) use an offset of -1903.
   */
  const HQ_FLASH_IDS = new Set([
    '00090', '00246', '00383', '00637', '00644', '00822', '00833', '00860',
    '00979', '00980', '01070', '01073', '01152', '01267', '01318', '01407',
    '01641', '01668', '01720', '01801', '01892', '01931', '01940', '02096',
    '02290', '02577', '02625', '02759', '02786', '02789', '02847', '02926',
    '02985', '03076', '03085', '03358', '03435', '03445', '03676', '03692',
    '03693', '03722', '03741', '03757', '03972', '04082', '04106', '04108',
    '04110', '04272', '04370', '04387', '04388', '04483', '04569', '04572',
    '04614', '04662', '04773', '04812', '04817', '04822', '04824', '04855',
    '04939', '04941', '05024', '05235', '05249', '05258', '05259', '05419',
    '05425', '05435', '05509', '05595', '05777', '06240', '06273', '06274',
    '06275', '06379', '06397', '06398', '06609', '06713', '06725', '06898',
    '07083', '07095', '07402', '07445', '07826', '07953'
  ]);

  const BAKED_FLASH_IDS = new Set([
    '00150', '00253', '00388', '00476', '00651', '00830', '00938', '01021',
    '01950', '02666', '02779', '02845', '02998', '02999', '03188', '03318',
    '03715', '03741', '04078', '05692', '05774', '06394', '06398'
  ]);

  const BOLIN_FLASH_IDS = new Set([
    '00388', '00644', '00651', '00822', '00830',
    '00833', '00980', '01026', '01070', '01720'
  ]);

  const HQ_PAGES = new Set([...HQ_PAGE_LIST, ...HQ_FLASH_IDS]);
  const BAKED_PAGES = new Set([...BAKED_PAGE_LIST, ...BAKED_FLASH_IDS]);
  const BOLIN_PAGES = new Set([...BOLIN_PAGE_LIST, ...BOLIN_FLASH_IDS]);

  /**
   * Multi-track audio definitions for Cascade (04106) and two-track Flashes.
   */
  const CASCADE_SEGMENTS = [
    '/storyfiles/hs2/04106/cascade_segment1.mp3',
    '/storyfiles/hs2/04106/cascade_segment2.mp3',
    '/storyfiles/hs2/04106/cascade_segment3.mp3',
    '/storyfiles/hs2/04106/cascade_segment4.mp3',
    '/storyfiles/hs2/04106/cascade_segment5.mp3'
  ];

  const TWO_TRACK_FLASHES = new Set(['00980', '00980_bolin', '03435', '04370']);

  /**
   * Extract canonical Flash ID from a media URL and optional pageId.
   * Preserves `_bolin` in the returned ID so Bolin-specific audio delays resolve accurately,
   * while stripping `_hq` and `_hqbaked` suffixes.
   * Also handles the special `A6A6I1` override on page `008143` (`06240.swf`).
   *
   * @param {string} url - Media URL (e.g., `/storyfiles/hs2/00980/00980_hq.swf`)
   * @param {string} [pageId] - Optional 6-digit MSPA page ID (e.g., `'008143'`)
   * @returns {string} Flash ID (e.g., `'00980'`, `'00980_bolin'`, `'04106'`, `'A6A6I1'`)
   */
  function getFlashIdFromUrl(url, pageId) {
    if (!url || typeof url !== 'string') return '';
    const cleanUrl = url.split('?')[0].split('#')[0];
    const parts = cleanUrl.split('/');
    const lastPart = parts[parts.length - 1] || '';
    const dotIdx = lastPart.lastIndexOf('.');
    const stem = dotIdx !== -1 ? lastPart.slice(0, dotIdx) : lastPart;
    const baseId = stem.replace(/_hqbaked$/i, '').replace(/_hq$/i, '');

    if (pageId === '008143' && baseId === '06240') {
      return 'A6A6I1';
    }
    return baseId;
  }

  /**
   * Compute the calibrated audio delay in milliseconds for a given Flash ID and track specifier.
   *
   * @param {string} flashId - Flash ID (e.g. `'00980'`, `'01801'`, `'04106'`, `'05258'`)
   * @param {number|string} [trackIndex=1] - 1-based track index or Cascade scrub string (`'1.5'`, `'4.5'`, `'5.5'`)
   * @param {boolean} [isReplay=false] - True when a track is replaying after > 30s (`05258_replay`)
   * @param {boolean} [isCascadeScrub=false] - True if scrubbing to a `.5` half-segment in Cascade
   * @returns {number} Calibrated offset in milliseconds
   */
  function getAudioDelay(flashId, trackIndex = 1, isReplay = false, isCascadeScrub = false) {
    if (!flashId) return 0;
    const rawId = flashId === 'A6A6I1' ? '06240' : String(flashId);
    const trackStr = String(trackIndex ?? 1);

    // Handle Cascade .5 mid-segment scrub offsets
    if (rawId === '04106' && (trackStr.endsWith('.5') || isCascadeScrub)) {
      const segNum = trackStr.charAt(0);
      if (segNum === '1') return -52900;
      if (segNum === '4') return -125250;
      if (segNum === '5') return -77900;
      const halfKey = `${rawId}_${segNum}.5`;
      if (halfKey in AUDIO_DELAYS) return AUDIO_DELAYS[halfKey];
    }

    const parsedTrack = parseInt(trackStr, 10);
    const trackNum = Number.isNaN(parsedTrack) || parsedTrack < 1 ? 1 : parsedTrack;

    if (isReplay && `${rawId}_replay` in AUDIO_DELAYS) {
      return AUDIO_DELAYS[`${rawId}_replay`];
    }

    const indexedKey = `${rawId}_${trackNum}`;
    if (indexedKey in AUDIO_DELAYS) {
      return AUDIO_DELAYS[indexedKey];
    }

    if (rawId in AUDIO_DELAYS) {
      return AUDIO_DELAYS[rawId];
    }

    // Fallback without _bolin if a Bolin track shares standard delay
    const nonBolinId = rawId.replace(/_bolin$/i, '');
    if (`${nonBolinId}_${trackNum}` in AUDIO_DELAYS) {
      return AUDIO_DELAYS[`${nonBolinId}_${trackNum}`];
    }
    if (nonBolinId in AUDIO_DELAYS) {
      return AUDIO_DELAYS[nonBolinId];
    }

    return 0;
  }

  /**
   * Build a full HTTP asset URL from an archive-relative path.
   */
  function formatAssetUrl(relPath, assetBase, cacheBuster) {
    if (!relPath || typeof relPath !== 'string') return '';
    if (/^https?:\/\//i.test(relPath) && !relPath.includes('mspaintadventures.com')) {
      return relPath;
    }
    let clean = relPath
      .replace(/^https?:\/\/(?:www\.|cdn\.)?mspaintadventures\.com\//i, '/')
      .replace(/^assets:\/\//i, '/')
      .replace(/^\/+/, '');
    if (clean.startsWith('assets/')) {
      clean = clean.slice('assets/'.length);
    }
    const base = (assetBase || '').replace(/\/+$/, '');
    const prefix = base ? `${base}/assets/${clean}` : `/assets/${clean}`;
    if (cacheBuster && !prefix.includes('?')) {
      return `${prefix}?v=${encodeURIComponent(cacheBuster)}`;
    }
    return prefix;
  }

  /**
   * Resolve a raw media URL (and optional page context) according to user settings (`hqAudio`, `bolin`,
   * `retcon1..5`, `unpeachy`, `pxsTavros`) into its effective SWF/image/video/HTML URL, external HQ MP3 tracks,
   * and Flash viewport dimensions/crop properties.
   *
   * @param {string} url - Raw media URL from `mspa.json` (e.g. `"/storyfiles/hs2/00980/00980.swf"`)
   * @param {object} [settings={}] - Active user settings (`hqAudio`, `bolin`, `jsFlashes`, `retcon1`..`5`, etc.)
   * @param {string} [assetBase=''] - Base URL for the Axum asset server (e.g. `"http://127.0.0.1:45100"`)
   * @param {string|number} [cacheBuster=''] - Optional VFS overlay generation counter for cache busting
   * @param {string} [pageId=''] - Optional MSPA 6-digit page ID
   * @param {string[]} [pageFlags=[]] - Optional page flags (`['R1']`, `['S']`, etc.)
   * @returns {object} Resolved media descriptor
   */
  function resolveMediaAssets(url, settings = {}, assetBase = '', cacheBuster = '', pageId = '', pageFlags = []) {
    const cfg = settings || {};
    const hqAudio = cfg.hqAudio !== false; // Default true
    const bolin = Boolean(cfg.bolin);
    const jsFlashes = cfg.jsFlashes !== false; // Default true

    let effectivePath = String(url || '').trim();

    // 1. Retcon / Enhancement image replacements
    if (Array.isArray(pageFlags) && pageFlags.length > 0) {
      for (let i = 1; i <= 5; i++) {
        const flagKey = `R${i}`;
        const settingKey = `retcon${i}`;
        if (pageFlags.includes(flagKey) && cfg[settingKey] === false) {
          effectivePath = effectivePath
            .replace(new RegExp(`_retcon${i}?`, 'gi'), '')
            .replace(/\/retcon(1|2|3|4|5)?\//gi, '/');
        }
      }
    }
    if (cfg.unpeachy && effectivePath.includes('05779_2.gif')) {
      effectivePath = effectivePath.replace('05779_2.gif', '05779_unpeachy.gif');
    }
    if (cfg.pxsTavros && effectivePath.includes('06852.gif')) {
      effectivePath = effectivePath.replace('06852.gif', '06852_pxs.gif');
    }

    const cleanNoQuery = effectivePath.split('?')[0].split('#')[0];
    const extMatch = cleanNoQuery.match(/\.([a-z0-9]+)$/i);
    const ext = extMatch ? extMatch[1].toLowerCase() : '';

    const rawFlashId = getFlashIdFromUrl(cleanNoQuery, '');
    const propId = getFlashIdFromUrl(cleanNoQuery, pageId);
    const audioTracks = [];

    // 2. SWF HQ Audio, Baked-in HQ, and Bolin resolution
    if (ext === 'swf') {
      const baseNoExt = cleanNoQuery.slice(0, -4).replace(/_hqbaked$/i, '').replace(/_hq$/i, '').replace(/_bolin$/i, '');
      const isBolinEligible = bolin && (BOLIN_PAGES.has(pageId) || BOLIN_PAGES.has(rawFlashId));
      const isHqEligible = hqAudio && (HQ_PAGES.has(pageId) || HQ_PAGES.has(rawFlashId));
      const isBakedEligible = hqAudio && (BAKED_PAGES.has(pageId) || BAKED_PAGES.has(rawFlashId));

      if (isBolinEligible) {
        if (hqAudio && rawFlashId !== '01026' && pageId !== '002926') {
          effectivePath = `${baseNoExt}_bolin_hq.swf`;
          if (rawFlashId === '00980' || pageId === '002880') {
            audioTracks.push(`${baseNoExt}_bolin_1.mp3`, `${baseNoExt}_2.mp3`);
          } else {
            audioTracks.push(`${baseNoExt}_bolin.mp3`);
          }
        } else {
          effectivePath = `${baseNoExt}_bolin.swf`;
        }
      } else if (isHqEligible) {
        effectivePath = `${baseNoExt}_hq.swf`;
        const dirPath = baseNoExt.slice(0, baseNoExt.lastIndexOf('/'));
        if (rawFlashId === '04106' || pageId === '006009') {
          for (let s = 1; s <= 5; s++) {
            audioTracks.push(`${dirPath}/cascade_segment${s}.mp3`);
          }
        } else if (TWO_TRACK_FLASHES.has(rawFlashId)) {
          audioTracks.push(`${baseNoExt}_1.mp3`, `${baseNoExt}_2.mp3`);
        } else {
          audioTracks.push(`${baseNoExt}.mp3`);
        }
      } else if (isBakedEligible) {
        effectivePath = `${baseNoExt}_hqbaked.swf`;
      }
    }

    const effectiveFlashId = getFlashIdFromUrl(effectivePath, pageId);
    const rawLookupId = getFlashIdFromUrl(effectivePath, '');
    const baseDims = INDEXED_FLASH_PROPS[effectiveFlashId] || INDEXED_FLASH_PROPS[rawLookupId] || INDEXED_FLASH_PROPS[rawFlashId] || {};

    const width = baseDims.width || 650;
    const height = baseDims.height || 450;
    const bgcolor = baseDims.bgcolor || '#fff';
    const croppedCssHeight = (jsFlashes && (CROP_HEIGHT[effectiveFlashId] || CROP_HEIGHT[rawLookupId])) || `${height}px`;

    return {
      originalUrl: url,
      effectivePath,
      resolvedUrl: formatAssetUrl(effectivePath, assetBase, cacheBuster),
      audioTracks: audioTracks.map((t) => formatAssetUrl(t, assetBase, cacheBuster)),
      rawAudioTracks: audioTracks,
      ext,
      flashId: effectiveFlashId,
      rawFlashId: rawLookupId || rawFlashId,
      width,
      height,
      bgcolor,
      cropHeight: croppedCssHeight,
      isCropped: Boolean(jsFlashes && (CROP_HEIGHT[effectiveFlashId] || CROP_HEIGHT[rawLookupId])),
      pauseAt: PAUSE_AT[rawLookupId] || PAUSE_AT[rawFlashId] || null
    };
  }

  const TUHCFlashProps = {
    // Canonical uppercase constants (M3 specification)
    INDEXED_FLASH_PROPS,
    AUDIO_DELAYS,
    HQ_PAGES,
    BAKED_PAGES,
    BOLIN_PAGES,
    CASCADE_SEGMENTS,
    TWO_TRACK_FLASHES,
    CROP_HEIGHT,
    HEIGHT_TRANSITIONS,
    GAME_OVER_STEPS,
    GAME_OVER,
    PAUSE_AT,

    // Legacy / reference lowercase aliases for test and component compatibility
    indexedFlashProps: INDEXED_FLASH_PROPS,
    audioDelay: AUDIO_DELAYS,
    cropHeight: CROP_HEIGHT,
    heightTransitions: HEIGHT_TRANSITIONS,
    gameOver: GAME_OVER,
    pauseAt: PAUSE_AT,

    // Helper functions
    getFlashIdFromUrl,
    getAudioDelay,
    formatAssetUrl,
    resolveMediaAssets
  };

  if (typeof root !== 'undefined') {
    root.TUHCFlashProps = TUHCFlashProps;
  }
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = TUHCFlashProps;
  }
})(typeof window !== 'undefined' ? window : globalThis);
