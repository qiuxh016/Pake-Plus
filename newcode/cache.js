// Pake Plus — Cache Proxy Injection
// Intercepts fetch() and XMLHttpRequest to route through the Rust cache engine.

(function () {
  'use strict';

  // ── Guard: only inject once ──────────────────────────────────
  if (window.__pakeCacheInjected) return;
  window.__pakeCacheInjected = true;

  // ── Detect Tauri IPC ──────────────────────────────────────────
  var invoke =
    window.__TAURI__ && window.__TAURI__.core
      ? window.__TAURI__.core.invoke
      : null;

  if (!invoke) {
    console.warn('[Pake Cache] Tauri IPC not available — cache disabled.');
    return;
  }

  // ── isTextContent helper ──────────────────────────────────────
  function isTextContent(contentType) {
    if (!contentType) return true;
    var ct = contentType.toLowerCase();
    return (
      ct.startsWith('text/') ||
      ct.indexOf('json') !== -1 ||
      ct.indexOf('javascript') !== -1 ||
      ct.indexOf('xml') !== -1 ||
      ct.indexOf('svg') !== -1
    );
  }

  // ── Intercept fetch() ─────────────────────────────────────────
  var _origFetch = window.fetch;

  window.fetch = function (input, init) {
    init = init || {};
    var method = (init.method || 'GET').toUpperCase();
    if (method !== 'GET') {
      return _origFetch.apply(this, arguments);
    }

    var url = typeof input === 'string' ? input : input.url;
    if (!url) return _origFetch.apply(this, arguments);

    // Skip non-http URLs.
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      return _origFetch.apply(this, arguments);
    }

    // Call Rust cache proxy.
    return invoke('cache_fetch', { url: url, forceCache: true })
      .then(function (res) {
        var headers = new Headers(res.headers || {});
        var body = res.body;
        if (res.body_base64) {
          var binary = atob(res.body_base64);
          var bytes = new Uint8Array(binary.length);
          for (var i = 0; i < binary.length; i++) {
            bytes[i] = binary.charCodeAt(i);
          }
          body = bytes.buffer;
        }
        return new Response(body, {
          status: res.status,
          headers: headers,
        });
      })
      .catch(function (err) {
        console.warn('[Pake Cache] cache_fetch failed, falling back: ' + err);
        return _origFetch.apply(this, arguments);
      });
  };

  // ── Intercept XMLHttpRequest ─────────────────────────────────
  // We wrap the open/send methods to check the cache for GET requests.
  var OrigXHR = window.XMLHttpRequest;

  function CacheXHR() {
    var xhr = new OrigXHR();
    var _open = xhr.open;
    var _send = xhr.send;
    var cacheUrl = null;
    var cacheMethod = null;

    xhr.open = function (method, url, async, user, password) {
      cacheMethod = method.toUpperCase();
      cacheUrl = url;
      return _open.call(this, method, url, async, user, password);
    };

    xhr.send = function (body) {
      if (cacheMethod === 'GET' && cacheUrl && (cacheUrl.startsWith('http://') || cacheUrl.startsWith('https://'))) {
        var self = this;
        invoke('cache_fetch', { url: cacheUrl, forceCache: true })
          .then(function (res) {
            var responseText = res.body;
            if (res.body_base64) {
              // For binary, we can't easily feed to XHR — fall back.
              return _send.call(self, body);
            }
            // Simulate a successful response.
            Object.defineProperty(self, 'readyState', { value: 4, writable: true });
            Object.defineProperty(self, 'status', { value: res.status, writable: true });
            Object.defineProperty(self, 'statusText', { value: 'OK', writable: true });
            Object.defineProperty(self, 'responseText', { value: responseText, writable: true });
            Object.defineProperty(self, 'response', { value: responseText, writable: true });
            Object.defineProperty(self, 'responseURL', { value: cacheUrl, writable: true });
            var contentType = res.content_type || 'text/html';
            Object.defineProperty(self, 'responseType', { value: '', writable: true });
            // Build response headers.
            var headerStr = '';
            Object.keys(res.headers || {}).forEach(function (k) {
              headerStr += k + ': ' + res.headers[k] + '\r\n';
            });
            var getAllResponseHeaders = function () { return headerStr; };
            var getResponseHeader = function (name) {
              var lc = name.toLowerCase();
              var found = null;
              Object.keys(res.headers || {}).forEach(function (k) {
                if (k.toLowerCase() === lc) found = res.headers[k];
              });
              return found;
            };
            Object.defineProperty(self, 'getAllResponseHeaders', {
              value: getAllResponseHeaders,
              writable: true,
            });
            Object.defineProperty(self, 'getResponseHeader', {
              value: getResponseHeader,
              writable: true,
            });
            // Dispatch events.
            var evt = new Event('readystatechange');
            self.dispatchEvent(evt);
            var loadEvt = new Event('load');
            self.dispatchEvent(loadEvt);
            if (self.onreadystatechange) self.onreadystatechange();
            if (self.onload) self.onload();
          })
          .catch(function () {
            return _send.call(self, body);
          });
      } else {
        return _send.call(this, body);
      }
    };

    return xhr;
  }

  CacheXHR.prototype = OrigXHR.prototype;
  CacheXHR.UNSENT = 0;
  CacheXHR.OPENED = 1;
  CacheXHR.HEADERS_RECEIVED = 2;
  CacheXHR.LOADING = 3;
  CacheXHR.DONE = 4;

  window.XMLHttpRequest = CacheXHR;

  console.log('[Pake Cache] Fetch & XHR interception active.');
})();
