// Keep Trunk's streaming compilation, with feedback on slow tablet connections.
export default function initializer() {
  let lastUpdate = 0;
  let watchdog;
  const status = document.getElementById('status');
  const progress = document.getElementById('boot-progress');
  const armTimeout = () => {
    clearTimeout(watchdog);
    watchdog = setTimeout(() => window.findcatFailure(new Error('게임 다운로드가 90초 동안 진행되지 않았어요.')), 90000);
  };
  return {
    onStart() {
      status.textContent = '율을 만나기 위해 게임을 내려받고 있어요…';
      armTimeout();
    },
    onProgress({current, total}) {
      armTimeout();
      const now = performance.now();
      if (now - lastUpdate < 100 && current !== total) return;
      lastUpdate = now;
      // Trunk 0.21 reports the size before wasm-opt, not the smaller served file.
      // Show measured downloaded bytes, without an incorrect percentage/denominator.
      const mb = (current / 1048576).toFixed(1);
      status.textContent = current >= total && total > 0
        ? '게임을 시작하고 있어요. 첫 실행은 조금 걸릴 수 있어요…'
        : `게임 다운로드 중 · ${mb} MB`;
    },
    onComplete() { clearTimeout(watchdog); },
    onSuccess() {
      progress.hidden = true;
      setTimeout(() => {
        if (document.getElementById('boot').style.display !== 'none'
            && !document.getElementById('boot').dataset.failed) {
          window.findcatFailure(new Error('동네 데이터를 불러오는 시간이 초과됐어요. 인터넷 연결을 확인해 주세요.'));
        }
      }, 120000);
    },
    onFailure(error) {
      window.findcatFailure(error);
    },
  };
}
