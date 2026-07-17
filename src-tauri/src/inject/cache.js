// Pake Plus — Cache Proxy + Offline Mode
// Intercepts fetch() and XMLHttpRequest, caches responses via Rust engine,
// and falls back to cache when offline.

(function () {
  "use strict";

  if (window.__pakeCacheInjected) return;
  window.__pakeCacheInjected = true;

  var invoke =
    window.__TAURI__ && window.__TAURI__.core
      ? window.__TAURI__.core.invoke
      : null;

  if (!invoke) {
    console.warn("[Pake Cache] Tauri IPC not available — cache disabled.");
    return;
  }

  // ── Offline state ─────────────────────────────────────────────
  var isOnline = navigator.onLine !== false;
  var offlineBadge = null;
  var offlineBanner = null;

  function showOfflineBanner() {
    if (offlineBanner) return;
    offlineBanner = document.createElement("div");
    offlineBanner.id = "__pake_offline_bnr__";
    offlineBanner.style.cssText =
      "position:fixed;top:0;left:0;right:0;z-index:2147483641;" +
      "background:#fef3c7;color:#92400e;padding:6px 16px;" +
      "text-align:center;font-size:12px;font-weight:500;" +
      "border-bottom:1px solid #fcd34d;" +
      "display:flex;align-items:center;justify-content:center;gap:8px";
    offlineBanner.innerHTML =
      "&#x26A0; You are offline &mdash; cached pages are still available.";
    setTimeout(function () {
      if (document.body)
        document.body.insertBefore(offlineBanner, document.body.firstChild);
    }, 100);
  }

  function hideOfflineBanner() {
    if (offlineBanner) {
      if (offlineBanner.parentNode)
        offlineBanner.parentNode.removeChild(offlineBanner);
      offlineBanner = null;
    }
  }

  function showOfflineBadge() {
    if (offlineBadge) return;
    offlineBadge = document.createElement("div");
    offlineBadge.id = "__pake_offline__";
    offlineBadge.style.cssText =
      "position:fixed;bottom:20px;left:20px;z-index:2147483640;" +
      "background:#f59e0b;color:#fff;padding:4px 12px;" +
      "border-radius:12px;font-size:11px;font-weight:600;" +
      "box-shadow:0 2px 8px rgba(245,158,11,.3);pointer-events:none";
    offlineBadge.textContent = "OFFLINE";
    setTimeout(function () {
      if (document.body) document.body.appendChild(offlineBadge);
    }, 200);
  }

  function hideOfflineBadge() {
    if (offlineBadge) {
      if (offlineBadge.parentNode)
        offlineBadge.parentNode.removeChild(offlineBadge);
      offlineBadge = null;
    }
  }

  function updateOnlineState(online) {
    var changed = isOnline !== online;
    isOnline = online;
    if (changed) {
      if (online) {
        hideOfflineBanner();
        hideOfflineBadge();
        console.log("[Pake Cache] Network restored — online.");
      } else {
        showOfflineBanner();
        showOfflineBadge();
        console.log("[Pake Cache] Network lost — offline mode.");
      }
    }
  }

  window.addEventListener("online", function () {
    updateOnlineState(true);
  });
  window.addEventListener("offline", function () {
    updateOnlineState(false);
  });
  updateOnlineState(navigator.onLine !== false);

  // ── Cache hit toast ─────────────────────────────────────────
  var toastTimer = null;

  function showCacheToast(fromCache, url) {
    if (!fromCache) return;
    var toast = document.getElementById("__pake_cache_toast__");
    if (!toast) {
      toast = document.createElement("div");
      toast.id = "__pake_cache_toast__";
      toast.style.cssText =
        "position:fixed;bottom:100px;right:20px;z-index:2147483640;" +
        "background:#16a34a;color:#fff;padding:6px 14px;" +
        "border-radius:20px;font-size:12px;font-weight:600;" +
        "box-shadow:0 2px 8px rgba(22,163,74,.35);" +
        "opacity:0;transform:translateY(10px);" +
        "transition:opacity .3s,transform .3s;pointer-events:none";
      document.body.appendChild(toast);
    }
    // Show a shortened URL
    var shortUrl = url;
    try {
      var u = new URL(url);
      shortUrl = u.hostname + u.pathname;
      if (shortUrl.length > 40) shortUrl = shortUrl.substring(0, 37) + "...";
    } catch (e) {}
    toast.textContent = "⚡ from cache • " + shortUrl;
    toast.style.opacity = "1";
    toast.style.transform = "translateY(0)";
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(function () {
      toast.style.opacity = "0";
      toast.style.transform = "translateY(10px)";
    }, 2500);
  }

  // ── Helpers ────────────────────────────────────────────────────
  function isTextContent(contentType) {
    if (!contentType) return true;
    var ct = contentType.toLowerCase();
    return (
      ct.startsWith("text/") ||
      ct.indexOf("json") !== -1 ||
      ct.indexOf("javascript") !== -1 ||
      ct.indexOf("xml") !== -1 ||
      ct.indexOf("svg") !== -1
    );
  }

  function offlineHTML(url) {
    return (
      '<!DOCTYPE html><html><head><meta charset="UTF-8"><title>Offline</title>' +
      "<style>*{margin:0;padding:0;box-sizing:border-box}" +
      "body{font-family:-apple-system,BlinkMacSystemFont,sans-serif;display:flex;" +
      "align-items:center;justify-content:center;height:100vh;background:#f8fafc;" +
      "color:#475569}.card{text-align:center;max-width:360px;padding:40px 32px;" +
      "background:#fff;border-radius:16px;box-shadow:0 4px 24px rgba(0,0,0,.06)}" +
      ".icon{font-size:48px;margin-bottom:16px}h1{font-size:18px;font-weight:700;" +
      "margin-bottom:8px;color:#1e293b}p{font-size:13px;line-height:1.6;margin-bottom:4px}" +
      ".url{font-size:11px;color:#94a3b8;word-break:break-all;margin-top:12px;" +
      "padding:8px;background:#f1f5f9;border-radius:8px}" +
      '</style></head><body><div class="card"><div class="icon">&#x1F4E1;</div>' +
      "<h1>Currently Offline</h1><p>This page is not cached and cannot be loaded " +
      "without a network connection.</p><p>Please check your connection and try again.</p>" +
      '<div class="url">' +
      url +
      "</div></div></body></html>"
    );
  }

  // ── Intercept fetch() ──────────────────────────────────────────
  var _origFetch = window.fetch;

  window.fetch = function (input, init) {
    init = init || {};
    var method = (init.method || "GET").toUpperCase();
    if (method !== "GET") {
      return _origFetch.apply(this, arguments);
    }

    var url = typeof input === "string" ? input : input ? input.url : "";
    if (!url || (!url.startsWith("http://") && !url.startsWith("https://"))) {
      return _origFetch.apply(this, arguments);
    }

    // Try Rust cache proxy — if offline, forceCache ensures we only use cache.
    var forceCache = !isOnline ? true : true;
    return invoke("cache_fetch", { url: url, forceCache: forceCache })
      .then(function (res) {
        if (res.error) {
          throw new Error(res.error);
        }
        // Show cache hit toast for navigational requests
        if (res.from_cache) showCacheToast(true, url);
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
          status: res.status || 200,
          headers: headers,
        });
      })
      .catch(function (err) {
        // If offline and cache failed, return a friendly offline page for HTML requests.
        if (!isOnline) {
          var accept = "";
          try {
            accept = (init.headers && init.headers.Accept) || "";
          } catch (e) {}
          if (accept.indexOf("text/html") !== -1 || !accept) {
            var html = offlineHTML(url);
            return new Response(html, {
              status: 503,
              headers: { "Content-Type": "text/html; charset=utf-8" },
            });
          }
        }
        // Fall back to real network.
        console.warn("[Pake Cache] cache_fetch failed, falling back: " + err);
        return _origFetch.apply(this, arguments);
      });
  };

  // ── Intercept XMLHttpRequest ───────────────────────────────────
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
      if (
        cacheMethod === "GET" &&
        cacheUrl &&
        (cacheUrl.startsWith("http://") || cacheUrl.startsWith("https://"))
      ) {
        var self = this;
        invoke("cache_fetch", { url: cacheUrl, forceCache: true })
          .then(function (res) {
            if (res.error) throw new Error(res.error);
            var responseText = res.body;
            if (res.body_base64) {
              return _send.call(self, body);
            }
            Object.defineProperty(self, "readyState", {
              value: 4,
              writable: true,
            });
            Object.defineProperty(self, "status", {
              value: res.status || 200,
              writable: true,
            });
            Object.defineProperty(self, "statusText", {
              value: "OK",
              writable: true,
            });
            Object.defineProperty(self, "responseText", {
              value: responseText,
              writable: true,
            });
            Object.defineProperty(self, "response", {
              value: responseText,
              writable: true,
            });
            Object.defineProperty(self, "responseURL", {
              value: cacheUrl,
              writable: true,
            });
            Object.defineProperty(self, "responseType", {
              value: "",
              writable: true,
            });

            var headerStr = "";
            Object.keys(res.headers || {}).forEach(function (k) {
              headerStr += k + ": " + res.headers[k] + "\r\n";
            });
            Object.defineProperty(self, "getAllResponseHeaders", {
              value: function () {
                return headerStr;
              },
              writable: true,
            });
            Object.defineProperty(self, "getResponseHeader", {
              value: function (name) {
                var lc = name.toLowerCase();
                var found = null;
                Object.keys(res.headers || {}).forEach(function (k) {
                  if (k.toLowerCase() === lc) found = res.headers[k];
                });
                return found;
              },
              writable: true,
            });

            self.dispatchEvent(new Event("readystatechange"));
            self.dispatchEvent(new Event("load"));
            if (self.onreadystatechange) self.onreadystatechange();
            if (self.onload) self.onload();
          })
          .catch(function () {
            // Offline fallback: just try normal request (will fail with network error)
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

  console.log(
    "[Pake Cache] Fetch & XHR interception active. Offline mode ready.",
  );
})();
