(function () {
  if (!window.pakeAdblock || !window.pakeAdblock.enabled) {
    return;
  }

  const config = window.pakeAdblock;
  const domains = new Set((config.domains || []).map((d) => d.toLowerCase()));
  const regexes = (config.regexes || [])
    .map((pattern) => {
      try {
        return new RegExp(pattern);
      } catch {
        return null;
      }
    })
    .filter(Boolean);

  let blocked = 0;

  function hostMatches(host, domain) {
    const h = host.toLowerCase();
    const d = domain.toLowerCase();
    return h === d || h.endsWith('.' + d);
  }

  function shouldBlockUrl(url) {
    if (!url || typeof url !== 'string') return false;
    try {
      const parsed = new URL(url, window.location.href);
      const host = parsed.hostname;
      for (const domain of domains) {
        if (hostMatches(host, domain)) return true;
      }
      const full = parsed.href;
      for (const regex of regexes) {
        if (regex.test(full)) return true;
      }
    } catch {
      for (const regex of regexes) {
        if (regex.test(url)) return true;
      }
    }
    return false;
  }

  function reportBlock() {
    blocked += 1;
    if (window.__TAURI_INTERNALS__?.invoke) {
      window.__TAURI_INTERNALS__.invoke('adblock_report_blocked', { count: blocked }).catch(() => {});
    }
  }

  function applyCosmeticRules() {
    const selectors = config.cosmetic_selectors || [];
    if (!selectors.length) return;
    const css = selectors
      .map((sel) => {
        if (sel.startsWith('##')) return sel.slice(2) + '{display:none!important;visibility:hidden!important;}';
        if (sel.startsWith('###')) return sel.slice(3) + '{display:none!important;visibility:hidden!important;}';
        return sel + '{display:none!important;visibility:hidden!important;}';
      })
      .join('\n');
    let style = document.getElementById('pake-adblock-style');
    if (!style) {
      style = document.createElement('style');
      style.id = 'pake-adblock-style';
      (document.head || document.documentElement).appendChild(style);
    }
    style.textContent = css;
  }

  function watchDom() {
    const observer = new MutationObserver(() => applyCosmeticRules());
    observer.observe(document.documentElement, { childList: true, subtree: true });
  }

  const originalFetch = window.fetch;
  window.fetch = function (input, init) {
    const url = typeof input === 'string' ? input : input?.url;
    if (shouldBlockUrl(url)) {
      reportBlock();
      return Promise.resolve(new Response('', { status: 204, statusText: 'Blocked' }));
    }
    return originalFetch.apply(this, arguments);
  };

  const originalOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method, url) {
    if (shouldBlockUrl(url)) {
      reportBlock();
      this.__pakeBlocked = true;
      return originalOpen.call(this, method, 'about:blank');
    }
    return originalOpen.apply(this, arguments);
  };

  const originalSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.send = function () {
    if (this.__pakeBlocked) return;
    return originalSend.apply(this, arguments);
  };

  applyCosmeticRules();
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      applyCosmeticRules();
      watchDom();
    });
  } else {
    watchDom();
  }
})();
