// Observe failed WebGL links without modifying shader sources or rendering state.
// Keep the last two failed programs on this origin for the local diagnostic page.
(() => {
  'use strict';
  const key = 'findcat.failed-shaders.v1';
  const proto = globalThis.WebGL2RenderingContext?.prototype;
  if (!proto) return;
  const parameter = proto.getProgramParameter;
  const failures = [];
  proto.getProgramParameter = function(program, pname) {
    const value = parameter.call(this, program, pname);
    if (pname !== this.LINK_STATUS || value !== false || failures.length >= 2 || failures.some(f => f.program === program)) return value;
    try {
      const shaders = this.getAttachedShaders(program).map(shader => ({
        stage: this.getShaderParameter(shader, this.SHADER_TYPE),
        compiled: this.getShaderParameter(shader, this.COMPILE_STATUS),
        log: this.getShaderInfoLog(shader),
        source: this.getShaderSource(shader),
      }));
      const report = {
        id: `device-failure-${failures.length + 1}`, shaders,
        date: new Date().toISOString(),
        log: this.getProgramInfoLog(program), context: this.getContextAttributes(),
      };
      failures.push({program, report});
      window.findcatGpuFailures = failures.map(f => f.report);
      // Exclude successful shaders and unrelated game/save/browser data.
      if (failures.length <= 2) {
        try { localStorage.setItem(key, JSON.stringify(window.findcatGpuFailures)); } catch {}
      }
    } catch {
      // Diagnostics must never change the WebGL API's return value or exception.
    }
    return value;
  };
})();
