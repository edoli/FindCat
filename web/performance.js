// Opt-in, on-device traces. Normal play installs no wrappers, observers or UI.
(() => {
  'use strict';
  if (new URLSearchParams(location.search).get('performance') !== '1') return;
  const nativeRaf = window.requestAnimationFrame.bind(window);
  const getContext = HTMLCanvasElement.prototype.getContext;
  let gl, active = false, current = null, started = 0, timer, observer, button, download, label;
  let frames = [], events = [], longTasks = [], slowCalls = [], result = null;
  let visibilityEpoch = 0;
  const limit = 14400; // Bound memory even if timers are throttled in the background.
  const now = () => performance.now() - started;
  const mark = (type, detail) => {
    if (active && events.length < 500) events.push({at_ms: now(), type, detail});
  };
  HTMLCanvasElement.prototype.getContext = function(kind, ...args) {
    const context = getContext.call(this, kind, ...args);
    if (kind === 'webgl2' && this.id === 'game' && context) gl = context;
    return context;
  };
  const proto = globalThis.WebGL2RenderingContext?.prototype;
  if (proto) {
    for (const name of ['drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced', 'blitFramebuffer']) {
      const original = proto[name];
      proto[name] = function(...args) {
        if (current && this === gl) {
          current[name === 'blitFramebuffer' ? 'blits' : 'draws']++;
          if (name !== 'blitFramebuffer') {
            const arrays = name.startsWith('drawArrays');
            const instances = name.endsWith('Instanced') ? args[arrays ? 3 : 4] : 1;
            current.submitted_elements += args[arrays ? 2 : 1] * instances;
          }
        }
        return original.apply(this, args);
      };
    }
    for (const name of ['bufferData', 'bufferSubData', 'texImage2D', 'texSubImage2D', 'texImage3D', 'texSubImage3D',
        'compileShader', 'linkProgram', 'getProgramParameter', 'getShaderParameter']) {
      const original = proto[name];
      proto[name] = function(...args) {
        if (!current || this !== gl) return original.apply(this, args);
        const start = performance.now();
        try { return original.apply(this, args); }
        finally {
          const elapsed = performance.now() - start;
          const group = name.startsWith('tex') ? 'texture' : name.startsWith('buffer') ? 'buffer' : 'shader';
          current[`${group}_calls`]++;
          current[`${group}_ms`] += elapsed;
          if (elapsed >= 4 && slowCalls.length < 250) slowCalls.push({at_ms: now(), call: name, duration_ms: elapsed});
        }
      };
    }
  }
  window.requestAnimationFrame = callback => nativeRaf(timestamp => {
    if (!active || !gl || current) return callback(timestamp);
    const start = performance.now();
    const frame = {at_ms: start - started, raf_ms: timestamp - started, callback_ms: 0,
      visible: document.visibilityState === 'visible', visibility_epoch: visibilityEpoch,
      draws: 0, blits: 0, submitted_elements: 0,
      texture_calls: 0, texture_ms: 0, buffer_calls: 0, buffer_ms: 0, shader_calls: 0, shader_ms: 0,
      width: gl.drawingBufferWidth, height: gl.drawingBufferHeight};
    current = frame;
    try { callback(timestamp); }
    finally {
      frame.callback_ms = performance.now() - start;
      current = null;
      if (frame.draws && frames.length < limit) frames.push(frame);
      if (frames.length >= limit) stop();
    }
  });
  const summarize = values => {
    const sorted = values.filter(Number.isFinite).sort((a, b) => a - b);
    if (!sorted.length) return null;
    return {samples: sorted.length, median: sorted[Math.floor(sorted.length * .5)],
      p95: sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * .95))],
      p99: sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * .99))],
      max: sorted[sorted.length - 1], mean: sorted.reduce((a, b) => a + b, 0) / sorted.length};
  };
  const device = () => {
    let renderer = null, vendor = null;
    if (gl && !gl.isContextLost()) {
      const debug = gl.getExtension('WEBGL_debug_renderer_info');
      renderer = gl.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER);
      vendor = gl.getParameter(debug ? debug.UNMASKED_VENDOR_WEBGL : gl.VENDOR);
    }
    return {user_agent: navigator.userAgent, renderer, vendor,
      viewport: [innerWidth, innerHeight], dpr: devicePixelRatio,
      physical: gl ? [gl.drawingBufferWidth, gl.drawingBufferHeight] : null};
  };
  let initialDevice;
  function stop() {
    if (!active) return;
    active = false;
    clearTimeout(timer);
    if (observer) {
      collectLongTasks(observer.takeRecords());
      observer.disconnect();
    }
    document.documentElement.removeAttribute('data-performance');
    const intervals = [];
    for (let i = 1; i < frames.length; i++) {
      // Keep background/rotation gaps in the raw trace, but exclude hidden pairs from statistics.
      if (frames[i].visible && frames[i - 1].visible && frames[i].visibility_epoch === frames[i - 1].visibility_epoch)
        intervals.push(frames[i].raf_ms - frames[i - 1].raf_ms);
    }
    const visible = frames.filter(f => f.visible);
    result = {schema: 'findcat.performance.v1', date: new Date().toISOString(),
      version_query: new URLSearchParams(location.search).get('v'), duration_ms: now(),
      device: initialDevice, final_device: device(),
      measurement: 'Instrumented browser RAF callback includes GL driver work/waits. No GPU busy time measurement; no gl.finish/readPixels. Background frames excluded from summaries. Diagnostic overhead is included.',
      summary: {frames: frames.length, visible_frames: visible.length,
        interval_ms: summarize(intervals), callback_ms: summarize(visible.map(f => f.callback_ms)),
        draws: summarize(visible.map(f => f.draws)), blits: summarize(visible.map(f => f.blits)),
        submitted_elements: summarize(visible.map(f => f.submitted_elements)),
        texture_upload_ms: summarize(visible.map(f => f.texture_ms)),
        buffer_upload_ms: summarize(visible.map(f => f.buffer_ms)),
        shader_api_ms: summarize(visible.map(f => f.shader_ms)),
        intervals_over_25ms: intervals.filter(v => v > 25).length,
        intervals_over_50ms: intervals.filter(v => v > 50).length},
      frames, events, long_tasks: longTasks, slow_gl_calls: slowCalls};
    button.textContent = '다시 기록';
    download.hidden = false;
    const timing = result.summary.interval_ms;
    label.textContent = timing ? `기록 완료 · ${timing.median.toFixed(1)}ms / p95 ${timing.p95.toFixed(1)}ms` : '프레임 기록 없음 · 게임 시작 후 다시 기록해 주세요';
  }
  function collectLongTasks(entries) {
    for (const entry of entries) {
      if (entry.startTime < started || longTasks.length >= 250) continue;
      longTasks.push({at_ms: entry.startTime - started, duration_ms: entry.duration});
    }
  }
  function start() {
    if (active) return stop();
    if (!gl || document.getElementById('boot')?.style.display !== 'none') {
      label.textContent = '게임이 준비된 뒤 기록을 시작해 주세요';
      return;
    }
    frames = []; events = []; longTasks = []; slowCalls = []; result = null;
    initialDevice = device();
    started = performance.now();
    active = true;
    document.documentElement.setAttribute('data-performance', '');
    if (globalThis.PerformanceObserver?.supportedEntryTypes?.includes('longtask')) {
      observer = new PerformanceObserver(list => collectLongTasks(list.getEntries()));
      observer.observe({type: 'longtask'});
    }
    button.textContent = '기록 끝내기';
    download.hidden = true;
    label.textContent = '30초 동안 평소처럼 플레이해 주세요';
    mark('start', {visibility: document.visibilityState});
    timer = setTimeout(stop, 30000);
  }
  document.addEventListener('DOMContentLoaded', () => {
    const panel = document.createElement('aside');
    panel.id = 'performance-panel';
    panel.setAttribute('aria-label', '성능 기록');
    panel.style.cssText = 'position:fixed;top:4px;right:4px;z-index:4;display:flex;gap:8px;align-items:center;flex-wrap:wrap;max-width:calc(100% - 8px);padding:8px;background:#263f35f2;color:#fbf7ed;border:1px solid #c2d1bd;border-radius:8px;font:13px system-ui';
    label = document.createElement('span');
    label.textContent = '끊기는 구간을 30초 기록해 주세요';
    button = document.createElement('button');
    button.textContent = '성능 기록 시작';
    button.id = 'performance-start';
    button.onclick = start;
    download = document.createElement('button');
    download.textContent = '기록 파일 저장';
    download.id = 'performance-download';
    download.hidden = true;
    download.onclick = () => {
      if (!result) return;
      const url = URL.createObjectURL(new Blob([JSON.stringify(result, null, 2)], {type: 'application/json'}));
      const link = document.createElement('a');
      link.href = url;
      link.download = 'findcat-performance.json';
      link.click();
      setTimeout(() => URL.revokeObjectURL(url), 10000);
    };
    for (const element of [button, download]) element.style.cssText = 'font:inherit;min-height:36px;padding:6px 10px;border:0;border-radius:5px;background:#eed9ab;color:#263f35;cursor:pointer';
    panel.append(label, button, download);
    for (const event of ['pointerdown', 'pointerup', 'keydown', 'keyup', 'wheel']) {
      document.addEventListener(event, e => {
        if (e.target.closest?.('#performance-panel') || e.repeat) return;
        // Record game controls, never typed text, pointer coordinates, or save contents.
        mark(event, e instanceof KeyboardEvent ? (['KeyW', 'KeyA', 'KeyS', 'KeyD', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Tab', 'Enter', 'Escape', 'Space'].includes(e.code) ? e.code : 'other') : e.pointerType || null);
      }, {capture: true, passive: true});
    }
    document.addEventListener('visibilitychange', () => {
      visibilityEpoch++;
      mark('visibility', document.visibilityState);
    });
    window.addEventListener('resize', () => mark('resize', {viewport: [innerWidth, innerHeight], dpr: devicePixelRatio}));
    new MutationObserver(() => {
      const value = document.documentElement.getAttribute('data-performance-state');
      if (value) mark('game', JSON.parse(value));
    }).observe(document.documentElement, {attributes: true, attributeFilter: ['data-performance-state']});
    document.body.append(panel);
  });
})();
