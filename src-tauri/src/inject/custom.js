// Pake Plus Settings Panel
window.__pakeDebug = {
  invoke: function (n) {
    var t = window.__TAURI__;
    return t
      ? t.core
          .invoke(n)
          .then(function (r) {
            alert("OK: " + JSON.stringify(r));
          })
          .catch(function (e) {
            alert("ERR: " + e);
          })
      : alert("No __TAURI__");
  },
};
document.title += " [D面板已注入]";
(function () {
  "use strict";

  var P = null,
    S = null,
    T = "general",
    theme = "light",
    lang = "zh",
    dirty = false;
  var I = function (c, a) {
    try {
      var t = window.__TAURI__ || window.__TAURI_INTERNALS__;
      var inv = t && (t.invoke || (t.core && t.core.invoke));
      if (!inv) {
        console.error("No Tauri invoke API found");
        return Promise.reject("No Tauri API");
      }
      return inv(c, a);
    } catch (e) {
      console.error("I() error:", e);
      return Promise.reject(e);
    }
  };
  var L = {
    zh: {
      title: "Pake Plus",
      sub: "设置面板",
      ng: "一般",
      na: "广告拦截",
      nc: "离线缓存",
      nl: "剪贴板",
      nd: "数据管理",
      ni: "关于",
      gt: "一般设置",
      gd: "应用外观与行为",
      tl: "暗色主题",
      th: "切换面板配色",
      ll: "界面语言",
      rl: "恢复默认设置",
      rh: "将所有设置重置为出厂值",
      rb: "恢复默认",
      at: "广告拦截",
      ad: "基于 EasyList 规则引擎，双重过滤",
      ae: "启用拦截",
      ah: "自动过滤广告与跟踪请求",
      ar: "自定义规则",
      arh: "每行一条，||匹配域名 ##隐藏元素",
      arb: "高级",
      ct: "离线缓存",
      cd: "HTTP 层代理缓存，断网也能浏览",
      ce: "启用缓存",
      ceh: "自动缓存浏览过的资源",
      cl: "缓存上限",
      cs: "统计",
      ch: "命中率（近1小时）",
      pt: "剪贴板管理",
      pd: "系统级监听，历史记录一键复用",
      pe: "启用监听",
      peh: "后台自动记录剪贴板变化",
      pm: "最大记录数",
      pr: "保留天数",
      ps: "忽略短文本",
      psh: "少于 2 字符不记录",
      dt: "数据管理",
      dd: "一键导出或导入，轻松迁移设备",
      de: "导出数据",
      ded: "打包为 .pake-data.zip",
      deb: "导出",
      di: "导入数据",
      did: "从下载目录读取恢复",
      dib: "导入",
      ot: "关于",
      od: "系统诊断信息",
      oa: "应用信息",
      os: "系统信息",
      oc: "复制诊断报告",
      sv: "保存设置",
      cc: "取消",
      ts: "已保存",
      tl2: "加载失败",
      tr: "已恢复默认",
      tf: "保存失败",
      tex: "导出中...",
      teok: "导出完成",
      tef: "导出失败",
      tim: "导入中...",
      tiok: "导入完成",
      tif: "导入失败",
      tc: "已复制",
      tcf: "复制失败",
      v: "版本",
      gi: "Git",
      bt: "编译时间",
      ru: "rustc",
      tg: "Target",
      fe: "启用功能",
      o: "OS",
      cp: "CPU",
      rm: "内存",
      dk: "磁盘可用",
      pi: "PID",
    },
    en: {
      title: "Pake Plus",
      sub: "Settings",
      ng: "General",
      na: "Adblock",
      nc: "Cache",
      nl: "Clipboard",
      nd: "Data",
      ni: "About",
      gt: "General",
      gd: "Appearance & behavior",
      tl: "Dark theme",
      th: "Toggle panel colors",
      ll: "Language",
      rl: "Reset defaults",
      rh: "Restore factory settings",
      rb: "Reset",
      at: "Adblock",
      ad: "EasyList engine, dual filtering",
      ae: "Enable",
      ah: "Filter ads & tracking",
      ar: "Custom rules",
      arh: "One per line, ||domain or ##.selector",
      arb: "Advanced",
      ct: "Cache",
      cd: "HTTP proxy, offline browsing",
      ce: "Enable",
      ceh: "Auto-cache resources",
      cl: "Cache limit",
      cs: "Statistics",
      ch: "Hit rate (1h)",
      pt: "Clipboard",
      pd: "System monitor, history reuse",
      pe: "Enable",
      peh: "Auto-record clipboard",
      pm: "Max records",
      pr: "Retention",
      ps: "Ignore short",
      psh: "Skip <2 chars",
      dt: "Data",
      dd: "Export or import data",
      de: "Export",
      ded: "Package as .pake-data.zip",
      deb: "Export",
      di: "Import",
      did: "Read from Downloads",
      dib: "Import",
      ot: "About",
      od: "System diagnostics",
      oa: "App Info",
      os: "System Info",
      oc: "Copy Report",
      sv: "Save",
      cc: "Cancel",
      ts: "Saved",
      tl2: "Load failed",
      tr: "Defaults restored",
      tf: "Save failed",
      tex: "Exporting...",
      teok: "Export complete",
      tef: "Export failed",
      tim: "Importing...",
      tiok: "Import complete",
      tif: "Import failed",
      tc: "Copied",
      tcf: "Copy failed",
      v: "Version",
      gi: "Git",
      bt: "Built",
      ru: "rustc",
      tg: "Target",
      fe: "Features",
      o: "OS",
      cp: "CPU",
      rm: "Memory",
      dk: "Disk free",
      pi: "PID",
    },
  };
  function t(k) {
    return (L[lang] || L.zh)[k] || k;
  }

  // ====== DOM helpers ======
  function el(tag, css) {
    var e = document.createElement(tag);
    if (css) e.style.cssText = css;
    return e;
  }
  function add(parent, child) {
    parent.appendChild(child);
    return child;
  }
  function txt(text, css) {
    var e = el("div", css);
    e.textContent = text;
    return e;
  }
  function btn(text, css, click) {
    var b = el(
      "button",
      css ||
        "padding:8px 16px;border:none;border-radius:8px;font-size:12px;font-weight:600;cursor:pointer",
    );
    b.textContent = text;
    if (click) b.onclick = click;
    return b;
  }
  function card() {
    return el(
      "div",
      "background:#fff;border:1px solid #f1f5f9;border-radius:10px;padding:16px;margin-bottom:12px;box-shadow:0 1px 3px rgba(0,0,0,.03)",
    );
  }
  function tg(id) {
    var t = el(
      "div",
      "width:42px;height:24px;background:#cbd5e1;border-radius:12px;cursor:pointer;flex-shrink:0;position:relative;transition:background .2s",
    );
    t.className = "__ps_tgl__";
    if (id) t.id = id;
    t.onclick = function () {
      t.classList.toggle("on");
      t.style.background = t.classList.contains("on") ? "#3b82f6" : "#cbd5e1";
      dirty = true;
    };
    return t;
  }
  function row(label, hint, child) {
    var r = el(
      "div",
      "display:flex;align-items:center;justify-content:space-between;padding:10px 0;border-bottom:1px solid #f8fafc",
    );
    var l = el("div");
    l.appendChild(txt(label, "font-size:12px;font-weight:500;color:#334155"));
    if (hint)
      l.appendChild(txt(hint, "font-size:10px;color:#94a3b8;margin-top:1px"));
    r.appendChild(l);
    if (child) r.appendChild(child);
    return r;
  }
  function kv(l, v) {
    return (
      '<div style="display:flex;justify-content:space-between;padding:5px 0;border-bottom:1px solid #f8fafc"><span style="color:#94a3b8">' +
      l +
      '</span><span style="font-weight:500;color:#334155">' +
      v +
      "</span></div>"
    );
  }

  // ====== Toast ======
  function toast(msg) {
    var t = document.getElementById("__ps_toast__");
    if (!t) {
      t = el(
        "div",
        "position:fixed;bottom:80px;right:24px;background:#1e293b;color:#fff;padding:10px 20px;border-radius:10px;font-size:12px;font-weight:500;z-index:2147483641;opacity:0;transform:translateY(10px);transition:all .25s;pointer-events:none;box-shadow:0 4px 16px rgba(0,0,0,.2)",
      );
      t.id = "__ps_toast__";
      document.body.appendChild(t);
    }
    t.textContent = msg;
    t.style.opacity = "1";
    t.style.transform = "translateY(0)";
    clearTimeout(t._t);
    t._t = setTimeout(function () {
      t.style.opacity = "0";
      t.style.transform = "translateY(10px)";
    }, 2000);
  }

  function tglSet(id, on) {
    var e = document.getElementById(id);
    if (e) {
      if (on) {
        e.classList.add("on");
        e.style.background = "#3b82f6";
      } else {
        e.classList.remove("on");
        e.style.background = "#cbd5e1";
      }
    }
  }
  function tglOn(id) {
    var e = document.getElementById(id);
    return !!(e && e.classList.contains("on"));
  }

  // ====== Open / Close ======
  window.__pakeOpenSettings = function () {
    if (P) {
      closePanel();
      return;
    }
    buildPanel();
    loadSettings();
    loadDiagnostics();
    setTimeout(function () {
      document.getElementById("__ps__").style.transform = "translateX(0)";
    }, 10);
  };
  function closePanel(force) {
    if (!P) return;
    if (!force && dirty) {
      if (!confirm("You have unsaved changes. Close anyway?")) return;
    }
    var e = document.getElementById("__ps__");
    if (e) {
      e.style.transform = "translateX(100%)";
      setTimeout(function () {
        if (P) P.remove();
        P = null;
        dirty = false;
      }, 200);
    }
  }

  // ====== Build ======
  function buildPanel() {
    P = el("div");
    P.innerHTML =
      '<div id="__ps__" style="position:fixed;top:0;right:0;width:400px;height:100vh;z-index:2147483640;background:#fff;box-shadow:-8px 0 40px rgba(0,0,0,.12);display:flex;flex-direction:column;font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Microsoft YaHei,sans-serif;font-size:13px;color:#1e293b;transform:translateX(100%);transition:transform .25s cubic-bezier(.4,0,.2,1)">' +
      '<div style="display:flex;align-items:center;justify-content:space-between;padding:16px 20px;border-bottom:1px solid #f1f5f9;flex-shrink:0"><div style="display:flex;align-items:center;gap:10px"><div style="width:32px;height:32px;background:linear-gradient(135deg,#3b82f6,#8b5cf6);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#fff;font-size:16px;font-weight:700">P</div><div><div id="__ps_title__" style="font-weight:700;font-size:14px;color:#0f172a">Pake Plus</div><div id="__ps_sub__" style="font-size:10px;color:#94a3b8">Settings</div></div></div><button id="__ps_close__" style="width:32px;height:32px;border:none;background:#f1f5f9;color:#64748b;border-radius:8px;font-size:18px;cursor:pointer;display:flex;align-items:center;justify-content:center">&times;</button></div>' +
      '<div style="display:flex;flex:1;overflow:hidden"><div id="__ps_nav__" style="width:120px;background:#f8fafc;flex-shrink:0;padding:12px 0;overflow-y:auto;border-right:1px solid #f1f5f9"></div><div style="flex:1;display:flex;flex-direction:column;overflow:hidden"><div id="__ps_body__" style="flex:1;overflow-y:auto;padding:20px"></div><div style="padding:10px 20px;border-top:1px solid #f1f5f9;display:flex;justify-content:space-between;align-items:center;flex-shrink:0"><span style="font-size:10px;color:#94a3b8">ESC to close</span><div style="display:flex;gap:8px"><button id="__ps_cancel__" style="padding:8px 16px;border:1px solid #e2e8f0;border-radius:8px;background:#fff;color:#64748b;cursor:pointer;font-size:12px;font-weight:600">Cancel</button><button id="__ps_save__" style="padding:8px 20px;border:none;border-radius:8px;background:#3b82f6;color:#fff;cursor:pointer;font-size:12px;font-weight:600;box-shadow:0 2px 8px rgba(59,130,246,.2)">Save</button></div></div></div></div></div>';
    document.body.appendChild(P);

    // Event bindings via JS
    document.getElementById("__ps_close__").onclick = closePanel;
    document.getElementById("__ps_cancel__").onclick = closePanel;
    document.getElementById("__ps_save__").onclick = doSave;
    P.addEventListener("input", function () {
      dirty = true;
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && P) closePanel();
    });

    buildNav();
    switchTab("general");
  }

  function buildNav() {
    var nav = document.getElementById("__ps_nav__");
    if (!nav) return;
    nav.innerHTML = "";
    var items = [
      ["general", t("ng"), "⚙"],
      ["adblock", t("na"), "🛡"],
      ["cache", t("nc"), "💾"],
      ["clipboard", t("nl"), "📋"],
      ["data", t("nd"), "📦"],
      ["about", t("ni"), "ℹ"],
    ];
    for (var i = 0; i < items.length; i++) {
      var n = el(
        "div",
        "display:flex;align-items:center;gap:8px;padding:10px 16px;cursor:pointer;font-size:12px;color:" +
          (items[i][0] === T ? "#3b82f6" : "#64748b") +
          ";font-weight:" +
          (items[i][0] === T ? "600" : "400") +
          ";border-right:" +
          (items[i][0] === T ? "2px solid #3b82f6" : "2px solid transparent") +
          ";transition:all .15s;margin:1px 0;border-bottom:1px solid #f1f5f9;background:" +
          (items[i][0] === T ? "#fff" : "transparent"),
      );
      n.textContent = items[i][2] + " " + items[i][1];
      n.style.userSelect = "none";
      (function (tab) {
        n.onclick = function () {
          switchTab(tab);
        };
      })(items[i][0]);
      nav.appendChild(n);
    }
  }

  // ====== Tab switching ======
  function switchTab(name) {
    T = name;
    var body = document.getElementById("__ps_body__");
    if (!body) return;
    body.innerHTML = "";
    var map = {
      general: [t("gt"), t("gd")],
      adblock: [t("at"), t("ad")],
      cache: [t("ct"), t("cd")],
      clipboard: [t("pt"), t("pd")],
      data: [t("dt"), t("dd")],
      about: [t("ot"), t("od")],
    };
    var m = map[name] || ["", ""];
    body.appendChild(
      txt(
        m[0],
        "font-size:16px;font-weight:700;color:#0f172a;margin-bottom:4px",
      ),
    );
    body.appendChild(
      txt(m[1], "font-size:11px;color:#94a3b8;margin-bottom:18px"),
    );

    if (name === "general") buildGeneral(body);
    else if (name === "adblock") buildAdblock(body);
    else if (name === "cache") buildCache(body);
    else if (name === "clipboard") buildClipboard(body);
    else if (name === "data") buildData(body);
    else if (name === "about") buildAbout(body);

    buildNav();
    if (S) {
      if (name === "general") fillGeneral();
      else if (name === "adblock") fillAdblock();
      else if (name === "cache") fillCache();
      else if (name === "clipboard") fillClipboard();
    }
  }

  function buildGeneral(body) {
    var c = card();
    body.appendChild(c);
    c.appendChild(row(t("tl"), t("th"), tg("__ps_theme__")));
    var sel = el(
      "select",
      "padding:6px 10px;border:1px solid #e2e8f0;border-radius:8px;font-size:11px;background:#f8fafc;color:#334155;outline:none;cursor:pointer",
    );
    sel.id = "__ps_lang__";
    sel.innerHTML =
      '<option value="zh">中文</option><option value="en">English</option>';
    c.appendChild(row(t("ll"), null, sel));

    var c2 = card();
    body.appendChild(c2);
    var r = el(
      "div",
      "display:flex;align-items:center;justify-content:space-between",
    );
    r.appendChild(
      txt(
        t("rl") +
          '<div style="font-size:10px;color:#94a3b8;margin-top:2px">' +
          t("rh") +
          "</div>",
        "font-size:12px;font-weight:600;color:#334155",
      ),
    );
    var b = btn(
      t("rb"),
      "padding:6px 14px;border:1px solid #fecaca;border-radius:6px;background:#fef2f2;color:#dc2626;font-size:11px;font-weight:600;cursor:pointer",
      doReset,
    );
    r.appendChild(b);
    c2.appendChild(r);

    var c3 = card();
    body.appendChild(c3);
    c3.appendChild(
      txt(
        "Version History",
        "font-size:12px;font-weight:600;color:#334155;margin-bottom:10px",
      ),
    );
    var vh = el("div", "font-size:11px");
    vh.id = "__ps_versions__";
    c3.appendChild(vh);
    loadVersionHistory();
  }

  function buildAdblock(body) {
    var c = card();
    body.appendChild(c);
    c.appendChild(row(t("ae"), t("ah"), tg("__ps_adblk__")));
    var c2 = card();
    body.appendChild(c2);
    var h = txt(
      "",
      "font-size:12px;font-weight:600;color:#334155;margin-bottom:4px",
    );
    h.innerHTML =
      t("ar") +
      ' <span style="font-size:10px;background:#eff6ff;color:#3b82f6;padding:2px 8px;border-radius:10px;margin-left:4px">' +
      t("arb") +
      "</span>";
    c2.appendChild(h);
    c2.appendChild(
      txt(t("arh"), "font-size:10px;color:#94a3b8;margin-bottom:10px"),
    );
    var ta = el(
      "textarea",
      "width:100%;height:90px;border:1px solid #e2e8f0;border-radius:8px;padding:10px;font-family:SF Mono,Consolas,monospace;font-size:11px;resize:none;outline:none;background:#f8fafc;line-height:1.6",
    );
    ta.id = "__ps_rules__";
    ta.placeholder = "||doubleclick.net^\n##.banner-ad";
    c2.appendChild(ta);
  }

  function buildCache(body) {
    var c = card();
    body.appendChild(c);
    c.appendChild(row(t("ce"), t("ceh"), tg("__ps_cache__")));
    var r2 = el(
      "div",
      "display:flex;align-items:center;justify-content:space-between;padding:10px 0",
    );
    r2.appendChild(txt(t("cl"), "font-size:12px;font-weight:500"));
    var w = el("div", "display:flex;align-items:center;gap:8px");
    var inp = el("input", "width:90px");
    inp.type = "range";
    inp.id = "__ps_csize__";
    inp.min = "50";
    inp.max = "1000";
    inp.step = "50";
    inp.value = "200";
    var v = el(
      "span",
      "font-size:12px;font-weight:600;color:#3b82f6;min-width:48px",
    );
    v.id = "__ps_cval__";
    v.textContent = "200 MB";
    inp.oninput = function () {
      v.textContent = inp.value + " MB";
    };
    w.appendChild(inp);
    w.appendChild(v);
    r2.appendChild(w);
    c.appendChild(r2);
    var c2 = card();
    body.appendChild(c2);
    c2.appendChild(
      txt(
        t("cs"),
        "font-size:12px;font-weight:600;color:#334155;margin-bottom:10px",
      ),
    );
    var s = el(
      "div",
      "display:flex;justify-content:space-between;padding:6px 0",
    );
    s.innerHTML =
      '<span style="font-size:11px;color:#94a3b8">' +
      t("ch") +
      '</span><span style="font-size:13px;font-weight:700;color:#3b82f6" id="__ps_chit__">0%</span>';
    c2.appendChild(s);
  }

  function buildClipboard(body) {
    var c = card();
    body.appendChild(c);
    c.appendChild(row(t("pe"), t("peh"), tg("__ps_clip__")));
    var s1 = el(
      "select",
      "padding:6px 10px;border:1px solid #e2e8f0;border-radius:8px;font-size:11px;background:#f8fafc;color:#334155;outline:none;cursor:pointer",
    );
    s1.id = "__ps_cmax__";
    s1.innerHTML =
      '<option value="500">500</option><option value="1000">1,000</option><option value="2000" selected>2,000</option><option value="5000">5,000</option>';
    c.appendChild(row(t("pm"), null, s1));
    var s2 = el(
      "select",
      "padding:6px 10px;border:1px solid #e2e8f0;border-radius:8px;font-size:11px;background:#f8fafc;color:#334155;outline:none;cursor:pointer",
    );
    s2.id = "__ps_cret__";
    s2.innerHTML =
      '<option value="7">7</option><option value="14">14</option><option value="30" selected>30</option><option value="0">∞</option>';
    c.appendChild(row(t("pr"), null, s2));
    c.appendChild(row(t("ps"), t("psh"), tg("__ps_cshort__")));

    // Clipboard stats - use the Rust global shortcut path
    c.appendChild(
      (function () {
        var r = el(
          "div",
          "padding:4px 0;display:flex;align-items:center;gap:8px",
        );
        r.innerHTML =
          '<span style="font-size:11px;color:#64748b">📊 剪贴板状态: </span><span id="__ps_cstats__" style="font-size:11px;font-weight:600;color:#334155">已启用</span>';
        return r;
      })(),
    );

    // Button to open clipboard history panel
    c.appendChild(
      (function () {
        var r = el("div", "padding:12px 0;text-align:center");
        var b = btn(
          "📋 打开剪贴板历史",
          "width:100%;padding:10px 0;border:none;border-radius:8px;background:#3b82f6;color:#fff;font-size:13px;cursor:pointer;box-shadow:0 2px 8px rgba(59,130,246,.2)",
        );
        b.onclick = function () {
          b.textContent = "打开中...";
          b.disabled = true;
          try {
            var T = window.__TAURI__ || window.__TAURI_INTERNALS__;
            var inv = T && (T.invoke || (T.core && T.core.invoke));
            if (inv) {
              var emit = T && T.event && T.event.emit;
              if (emit) {
                emit("open-clipboard-panel")
                  .then(function () {
                    b.textContent = "📋 打开剪贴板历史";
                    b.disabled = false;
                  })
                  .catch(function (e) {
                    b.textContent = "打开失败，请用 Ctrl+Shift+V";
                    b.disabled = false;
                  });
              } else {
                b.textContent = "请用 Ctrl+Shift+V";
                b.disabled = false;
              }
            } else {
              b.textContent = "请用 Ctrl+Shift+V";
              b.disabled = false;
            }
          } catch (e) {
            b.textContent = "请用 Ctrl+Shift+V";
            b.disabled = false;
          }
        };
        r.appendChild(b);
        return r;
      })(),
    );
    // Hint to open clipboard history via keyboard
    c.appendChild(
      (function () {
        var h = el(
          "div",
          "margin-top:8px;padding:10px 12px;background:#f0f9ff;border:1px solid #bae6fd;border-radius:8px;font-size:11px;line-height:1.6;color:#0369a1",
        );
        h.innerHTML =
          "<strong>⌨ 快捷键</strong><br>按 <b>Ctrl+Shift+V</b> 打开剪贴板历史面板<br>复制文本后可在面板中查看、搜索、复用";
        return h;
      })(),
    );

    // Try to load stats via IPC as well (may fail silently)
    try {
      var T = window.__TAURI__ || window.__TAURI_INTERNALS__;
      if (T) {
        var inv = T.invoke || (T.core && T.core.invoke);
        if (inv) {
          inv("clipboard_stats")
            .then(function (st) {
              var el = document.getElementById("__ps_cstats__");
              if (el && st)
                el.textContent = "共 " + (st.total || 0) + " 条，已启用";
            })
            .catch(function () {});
        }
      }
    } catch (e) {}
  }
  function buildData(body) {
    var c = card();
    body.appendChild(c);
    var r = el(
      "div",
      "display:flex;align-items:center;justify-content:space-between",
    );
    r.appendChild(
      txt(
        t("de") +
          '<div style="font-size:10px;color:#94a3b8;margin-top:2px">' +
          t("ded") +
          "</div>",
        "font-size:12px;font-weight:600;color:#334155",
      ),
    );
    r.appendChild(
      btn(
        t("deb"),
        "padding:8px 16px;border:none;border-radius:8px;background:#3b82f6;color:#fff;font-size:11px;font-weight:600;cursor:pointer",
        doExport,
      ),
    );
    c.appendChild(r);
    var res = el("div", "font-size:10px;color:#94a3b8;margin-top:8px");
    res.id = "__ps_exp_res__";
    c.appendChild(res);
    var c2 = card();
    body.appendChild(c2);
    var r2 = el(
      "div",
      "display:flex;align-items:center;justify-content:space-between",
    );
    r2.appendChild(
      txt(
        t("di") +
          '<div style="font-size:10px;color:#94a3b8;margin-top:2px">' +
          t("did") +
          "</div>",
        "font-size:12px;font-weight:600;color:#334155",
      ),
    );
    r2.appendChild(
      btn(
        t("dib"),
        "padding:8px 16px;border:1px solid #e2e8f0;border-radius:8px;background:#fff;color:#64748b;font-size:11px;font-weight:600;cursor:pointer",
        doImport,
      ),
    );
    c2.appendChild(r2);
    var res2 = el("div", "font-size:10px;color:#94a3b8;margin-top:8px");
    res2.id = "__ps_imp_res__";
    c2.appendChild(res2);
  }

  function buildAbout(body) {
    var c = card();
    body.appendChild(c);
    c.appendChild(
      txt(
        t("oa"),
        "font-size:12px;font-weight:600;color:#334155;margin-bottom:10px",
      ),
    );
    var da = el("div", "font-size:11px");
    da.id = "__ps_dapp__";
    c.appendChild(da);
    var c2 = card();
    body.appendChild(c2);
    c2.appendChild(
      txt(
        t("os"),
        "font-size:12px;font-weight:600;color:#334155;margin-bottom:10px",
      ),
    );
    var ds = el("div", "font-size:11px");
    ds.id = "__ps_dsys__";
    c2.appendChild(ds);
    body.appendChild(
      btn(
        "📋 " + t("oc"),
        "width:100%;padding:10px;border:1px solid #e2e8f0;border-radius:8px;background:#fff;color:#64748b;font-size:12px;font-weight:600;cursor:pointer",
        doCopyReport,
      ),
    );
    var res = el(
      "div",
      "font-size:10px;color:#3b82f6;margin-top:8px;text-align:center",
    );
    res.id = "__ps_dres__";
    body.appendChild(res);
  }

  // ====== Load / Fill ======
  function loadSettings() {
    I("get_settings")
      .then(function (s) {
        S = s;
        theme = (S.general || {}).theme || "light";
        lang = (S.general || {}).language || "zh";
        fillAll();
        updateUI();
      })
      .catch(function () {
        toast(t("tl2"));
      });
  }
  function fillAll() {
    fillGeneral();
    fillAdblock();
    fillCache();
    fillClipboard();
  }
  function updateUI() {
    document.getElementById("__ps_title__").textContent = t("title");
    document.getElementById("__ps_sub__").textContent = t("sub");
    document.getElementById("__ps_save__").textContent = t("sv");
    document.getElementById("__ps_cancel__").textContent = t("cc");
  }
  function fillGeneral() {
    var tg = document.getElementById("__ps_theme__");
    if (tg) {
      if (theme === "dark") {
        tg.classList.add("on");
        tg.style.background = "#3b82f6";
      } else {
        tg.classList.remove("on");
        tg.style.background = "#cbd5e1";
      }
    }
    var ls = document.getElementById("__ps_lang__");
    if (ls) ls.value = lang;
  }
  function fillAdblock() {
    if (!S || !S.adblock) return;
    tglSet("__ps_adblk__", S.adblock.enabled);
    var ta = document.getElementById("__ps_rules__");
    if (ta) ta.value = (S.adblock.custom_rules || []).join("\n");
  }
  function fillCache() {
    if (!S || !S.cache) return;
    tglSet("__ps_cache__", S.cache.enabled);
    var r = document.getElementById("__ps_csize__");
    if (r) r.value = S.cache.max_size_mb || 200;
    var v = document.getElementById("__ps_cval__");
    if (v) v.textContent = (S.cache.max_size_mb || 200) + " MB";
    var h = document.getElementById("__ps_chit__");
    if (h) h.textContent = Math.round(S.cache.hit_rate_1h || 0) + "%";
  }
  function fillClipboard() {
    if (!S || !S.clipboard) return;
    tglSet("__ps_clip__", S.clipboard.enabled);
    var mr = document.getElementById("__ps_cmax__");
    if (mr) mr.value = S.clipboard.max_records || 2000;
    var rd = document.getElementById("__ps_cret__");
    if (rd) rd.value = S.clipboard.retention_days || 30;
    tglSet("__ps_cshort__", S.clipboard.ignore_short);
  }
  function loadDiagnostics() {
    I("get_diagnostics")
      .then(function (d) {
        var a = document.getElementById("__ps_dapp__");
        if (a)
          a.innerHTML =
            kv(t("v"), "v" + d.app_version) +
            kv(t("gi"), d.git_commit) +
            kv(t("bt"), d.build_time) +
            kv(t("ru"), d.rustc_version) +
            kv(t("tg"), d.target_triple) +
            kv(t("fe"), (d.enabled_features || []).join(", ") || "-");
        var s = document.getElementById("__ps_dsys__");
        if (s)
          s.innerHTML =
            kv(t("o"), d.os_name + " " + d.os_version) +
            kv(t("cp"), d.cpu_cores + "") +
            kv(t("rm"), d.used_ram_mb + " / " + d.total_ram_mb + " MB") +
            kv(t("dk"), d.disk_free_mb + " / " + d.disk_total_mb + " MB") +
            kv(t("pi"), d.pid);
      })
      .catch(function () {});
  }

  // ====== Actions ======
  function safeVal(id, fallback) {
    var el = document.getElementById(id);
    return el ? el.value : fallback;
  }
  function doSave() {
    var btn = document.getElementById("__ps_save__");
    var prevText = btn.textContent;
    btn.textContent = "...";
    btn.disabled = true;
    var prev = S || { adblock: {}, cache: {}, clipboard: {}, general: {} };
    var s = {
      adblock: {
        enabled: document.getElementById("__ps_adblk__")
          ? tglOn("__ps_adblk__")
          : (prev.adblock || {}).enabled || false,
        custom_rules: document.getElementById("__ps_rules__")
          ? document
              .getElementById("__ps_rules__")
              .value.split("\n")
              .map(function (l) {
                return l.trim();
              })
              .filter(function (l) {
                return l;
              })
          : (prev.adblock || {}).custom_rules || [],
      },
      cache: {
        enabled: document.getElementById("__ps_cache__")
          ? tglOn("__ps_cache__")
          : (prev.cache || {}).enabled,
        max_size_mb: document.getElementById("__ps_csize__")
          ? parseInt(document.getElementById("__ps_csize__").value) || 200
          : (prev.cache || {}).max_size_mb || 200,
        hit_rate_1h: (prev.cache || {}).hit_rate_1h || 0,
      },
      clipboard: {
        enabled: document.getElementById("__ps_clip__")
          ? tglOn("__ps_clip__")
          : (prev.clipboard || {}).enabled,
        max_records: document.getElementById("__ps_cmax__")
          ? parseInt(document.getElementById("__ps_cmax__").value) || 2000
          : (prev.clipboard || {}).max_records || 2000,
        retention_days: document.getElementById("__ps_cret__")
          ? parseInt(document.getElementById("__ps_cret__").value) || 30
          : (prev.clipboard || {}).retention_days || 30,
        ignore_short: document.getElementById("__ps_cshort__")
          ? tglOn("__ps_cshort__")
          : (prev.clipboard || {}).ignore_short,
      },
      general: {
        theme: document.getElementById("__ps_theme__")
          ? tglOn("__ps_theme__")
            ? "dark"
            : "light"
          : (prev.general || {}).theme || "light",
        language:
          safeVal("__ps_lang__", (prev.general || {}).language || "zh") || "zh",
      },
    };
    I("validate_settings", { settings: s })
      .then(function () {
        I("save_settings", { settings: s })
          .then(function () {
            var ot = theme,
              ol = lang;
            S = s;
            theme = s.general.theme;
            lang = s.general.language;
            if (theme !== ot) applyTheme();
            if (lang !== ol) {
              switchTab(T);
            }
            dirty = false;
            toast(t("ts"));
          })
          .catch(function (e) {
            console.error("[Pake] save failed:", e);
            toast(t("tf") + ": " + e);
          })
          .finally(function () {
            btn.textContent = prevText;
            btn.disabled = false;
          });
      })
      .catch(function (errors) {
        btn.textContent = prevText;
        btn.disabled = false;
        toast(
          "Validation error: " +
            (Array.isArray(errors) ? errors.join("; ") : errors),
        );
      });
  }
  function doReset() {
    I("reset_settings")
      .then(function () {
        S = null;
        theme = "light";
        lang = "zh";
        loadSettings();
        switchTab("general");
        toast(t("tr"));
      })
      .catch(function (e) {
        console.error("[Pake] reset failed:", e);
        toast(t("tf") + ": " + e);
      });
  }
  function doExport() {
    var el2 = document.getElementById("__ps_exp_res__");
    if (el2) {
      el2.textContent = t("tex");
      el2.style.color = "#94a3b8";
    }
    I("export_data")
      .then(function (p) {
        if (el2) {
          el2.textContent = t("teok") + ": " + p;
          el2.style.color = "#16a34a";
        }
        toast(t("teok"));
      })
      .catch(function (e) {
        console.error("[Pake] export failed:", e);
        if (el2) {
          el2.textContent = t("tef");
          el2.style.color = "#dc2626";
        }
      });
  }
  function doImport() {
    var el2 = document.getElementById("__ps_imp_res__");
    if (el2) {
      el2.textContent = "previewing...";
      el2.style.color = "#94a3b8";
    }
    I("preview_import")
      .then(function (preview) {
        var ok = confirm(preview + "\n\nProceed with import?");
        if (!ok) {
          if (el2) {
            el2.textContent = "cancelled";
            el2.style.color = "#94a3b8";
          }
          return;
        }
        if (el2) {
          el2.textContent = t("tim");
          el2.style.color = "#94a3b8";
        }
        I("import_data")
          .then(function (m) {
            if (el2) {
              el2.textContent = m;
              el2.style.color = "#16a34a";
            }
            toast(t("tiok"));
          })
          .catch(function (e) {
            console.error("[Pake] import failed:", e);
            if (el2) {
              el2.textContent = t("tef");
              el2.style.color = "#dc2626";
            }
          });
      })
      .catch(function (e) {
        console.error("[Pake] preview failed:", e);
        if (el2) {
          el2.textContent = "no data file found";
          el2.style.color = "#dc2626";
        }
      });
  }
  function doCopyReport() {
    I("copy_diagnostics_report")
      .then(function () {
        var el2 = document.getElementById("__ps_dres__");
        if (el2) el2.textContent = t("tc");
        toast(t("tc"));
      })
      .catch(function (e) {
        console.error("[Pake] copy diagnostics failed:", e);
        toast(t("tcf"));
      });
  }
  function loadVersionHistory() {
    I("list_backups")
      .then(function (list) {
        var vh = document.getElementById("__ps_versions__");
        if (!vh) return;
        if (!list || !list.length) {
          vh.innerHTML = '<span style="color:#94a3b8">No backups yet</span>';
          return;
        }
        vh.innerHTML = "";
        for (var i = 0; i < list.length; i++) {
          var b2 = list[i];
          var row2 = el(
            "div",
            "display:flex;align-items:center;justify-content:space-between;padding:6px 0;border-bottom:1px solid #f1f5f9",
          );
          row2.innerHTML =
            '<span style="color:#64748b">v' +
            b2.version +
            " — " +
            b2.time +
            "</span>";
          var rst = btn(
            "Restore",
            "padding:3px 10px;border:1px solid #e2e8f0;border-radius:5px;background:#fff;color:#3b82f6;font-size:10px;font-weight:600;cursor:pointer",
            function () {
              doRollback(b2.version);
            },
          );
          row2.appendChild(rst);
          vh.appendChild(row2);
        }
      })
      .catch(function (e) {
        console.error("[Pake] list backups failed:", e);
      });
  }
  function doRollback(v) {
    if (!confirm("Restore settings from backup v" + v + "?")) return;
    I("rollback_settings", { version: v })
      .then(function (msg) {
        toast(msg);
        loadSettings();
        switchTab(T);
      })
      .catch(function (e) {
        toast("Rollback failed: " + e);
      });
  }

  // ====== Theme ======
  function applyTheme() {
    var s = document.getElementById("__ps_ts__");
    if (!s) {
      s = document.createElement("style");
      s.id = "__ps_ts__";
      document.head.appendChild(s);
    }
    s.textContent =
      theme === "dark"
        ? "#__ps__{background:#1e293b!important;color:#e2e8f0}#__ps__ .__ps_card{background:#334155!important;border-color:#475569!important}#__ps_nav__{background:#0f172a!important;border-color:#334155!important}#__ps__ textarea,#__ps__ select{background:#475569!important;color:#e2e8f0!important;border-color:#64748b!important}#__ps__ select option{background:#334155;color:#e2e8f0}"
        : "";
    var fab = document.getElementById("__ps_fab_inner__");
    if (fab)
      fab.style.background =
        theme === "dark"
          ? "linear-gradient(135deg,#6366f1,#a855f7)"
          : "linear-gradient(135deg,#3b82f6,#8b5cf6)";
  }

  // ====== Welcome page ======
  function showWelcome() {
    var w = document.getElementById("__ps_welcome__");
    if (w) return;
    w = el(
      "div",
      "position:fixed;top:0;left:0;width:100vw;height:100vh;z-index:2147483645;background:linear-gradient(135deg,#0f172a 0%,#1e293b 100%);display:flex;align-items:center;justify-content:center;font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Microsoft YaHei,sans-serif;transition:opacity .4s",
    );
    w.id = "__ps_welcome__";
    w.innerHTML =
      '<div style="text-align:center;max-width:520px">' +
      '<div style="width:64px;height:64px;margin:0 auto 20px;background:linear-gradient(135deg,#3b82f6,#8b5cf6);border-radius:16px;display:flex;align-items:center;justify-content:center;font-size:32px;font-weight:700;color:#fff">P</div>' +
      '<h1 style="font-size:28px;font-weight:800;color:#f1f5f9;margin:0 0 6px">Pake Plus</h1>' +
      '<p style="font-size:13px;color:#94a3b8;margin:0 0 32px">Lightweight desktop app with superpowers</p>' +
      '<div style="display:flex;gap:12px;margin-bottom:32px" id="__ps_welcome_cards__"></div>' +
      "<button id=\"__ps_welcome_start__\" style=\"padding:12px 48px;border:none;border-radius:10px;background:linear-gradient(135deg,#3b82f6,#8b5cf6);color:#fff;font-size:15px;font-weight:700;cursor:pointer;box-shadow:0 4px 20px rgba(59,130,246,.3);transition:transform .15s,box-shadow .15s\" onmouseover=\"this.style.transform='scale(1.03)';this.style.boxShadow='0 6px 28px rgba(59,130,246,.4)'\" onmouseout=\"this.style.transform='scale(1)';this.style.boxShadow='0 4px 20px rgba(59,130,246,.3)'\">Start</button>" +
      "</div>";
    document.body.appendChild(w);

    document.getElementById("__ps_welcome_start__").onclick = function () {
      w.style.opacity = "0";
      setTimeout(function () {
        if (w.parentNode) w.parentNode.removeChild(w);
      }, 400);
    };

    // Load status
    var cards = document.getElementById("__ps_welcome_cards__");
    if (cards)
      cards.innerHTML =
        '<div style="color:#94a3b8;font-size:12px">Loading...</div>';
    I("get_module_stats")
      .then(function (m) {
        var cards2 = document.getElementById("__ps_welcome_cards__");
        if (!cards2) return;
        cards2.innerHTML =
          card2(
            "🛡",
            "Adblock",
            m.adblock?.summary || "—",
            m.adblock?.stats?.enabled ? "#16a34a" : "#94a3b8",
          ) +
          card2(
            "💾",
            "Cache",
            m.cache?.summary || "—",
            m.cache?.stats?.enabled ? "#16a34a" : "#94a3b8",
          ) +
          card2(
            "📋",
            "Clipboard",
            m.clipboard?.summary || "—",
            m.clipboard?.stats?.enabled ? "#16a34a" : "#94a3b8",
          );
      })
      .catch(function (e) {
        console.error("[Pake] welcome stats failed:", e);
        var cards3 = document.getElementById("__ps_welcome_cards__");
        if (cards3)
          cards3.innerHTML =
            '<div style="color:#ef4444;font-size:12px">Failed to load: ' +
            (e || "unknown") +
            "</div>";
      });
  }
  function card2(icon, name, status, color) {
    return (
      '<div style="flex:1;background:rgba(255,255,255,.05);border:1px solid rgba(255,255,255,.08);border-radius:12px;padding:16px 12px;text-align:center">' +
      '<div style="font-size:24px;margin-bottom:6px">' +
      icon +
      "</div>" +
      '<div style="font-size:11px;color:#e2e8f0;font-weight:600;margin-bottom:4px">' +
      name +
      "</div>" +
      '<div style="font-size:10px;color:' +
      color +
      '">' +
      status +
      "</div></div>"
    );
  }

  // ====== DOM init (deferred until DOMContentLoaded) ======
  document.addEventListener("DOMContentLoaded", function () {
    // Welcome page
    showWelcome();

    // Floating button
    var fab = el(
      "div",
      "position:fixed;bottom:28px;right:28px;z-index:2147483640",
    );
    fab.innerHTML =
      '<div id="__ps_fab_inner__" style="width:44px;height:44px;background:linear-gradient(135deg,#3b82f6,#8b5cf6);border-radius:50%;display:flex;align-items:center;justify-content:center;color:#fff;font-size:20px;box-shadow:0 4px 16px rgba(59,130,246,.3);cursor:pointer;transition:all .2s;user-select:none">⚙</div>';
    fab.onclick = function () {
      window.__pakeOpenSettings();
    };
    document.body.appendChild(fab);

    // Toggle CSS
    var css = document.createElement("style");
    css.textContent =
      '.__ps_tgl__{width:42px;height:24px;background:#cbd5e1;border-radius:12px;cursor:pointer;flex-shrink:0;position:relative;transition:background .2s}.__ps_tgl__.on{background:#3b82f6}.__ps_tgl__::after{content:"";position:absolute;top:2px;left:2px;width:20px;height:20px;background:#fff;border-radius:50%;transition:transform .2s;box-shadow:0 1px 4px rgba(0,0,0,.12)}.__ps_tgl__.on::after{transform:translateX(18px)}';
    document.head.appendChild(css);
  });
})();
try {
  var __pgb = document.createElement("div");
  __pgb.style.cssText =
    "position:fixed;bottom:20px;right:20px;z-index:2147483640;width:44px;height:44px;background:linear-gradient(135deg,#3b82f6,#8b5cf6);border-radius:14px;cursor:pointer;display:flex;align-items:center;justify-content:center;color:#fff;font-size:22px;box-shadow:0 4px 16px rgba(59,130,246,.35);transition:transform .15s;user-select:none";
  __pgb.textContent = "⚙";
  __pgb.title = "Pake Plus Settings";
  __pgb.onclick = function () {
    if (window.__pakeOpenSettings) window.__pakeOpenSettings();
  };
  document.documentElement.appendChild(__pgb);
} catch (e) {}
