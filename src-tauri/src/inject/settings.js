(() => {
  document.title += " [C侧栏已注入]";
  if (window.__pakeClipboardSidebarInstalled) return;
  window.__pakeClipboardSidebarInstalled = true;

  const rules = {
    C1: {
      title: "系统剪贴板自动监听",
      items: [
        "应用启动后在后台监听系统文本剪贴板，不弹窗、不打断复制操作。",
        "从任意应用复制两段不同文本，再按 Ctrl+Shift+V；两条内容都应出现在历史列表中。",
        "重复复制同一文本不会生成重复记录。",
      ],
    },
    C2: {
      title: "剪贴板历史面板",
      items: [
        "按 Ctrl+Shift+V 打开历史面板；初始为 320 × 480，可用鼠标拖动窗口边缘调整大小。",
        "搜索、复制、展开、删除、设置和清空操作都不会退出历史面板。",
        "按 ESC 或点击历史面板右上角的 × 关闭历史面板。",
      ],
    },
    C3: {
      title: "全文搜索",
      items: [
        "在历史面板顶部输入中文、英文或多个空格分隔的关键词。",
        "无需按回车，列表应随输入实时更新；多词采用 AND 逻辑。",
        "没有结果时显示“没有匹配记录”。",
      ],
    },
    C4: {
      title: "一键复制复用",
      items: [
        "点击任意历史记录或记录上的“复制”，该条内容写回系统剪贴板。",
        "历史面板保持打开，鼠标点击位置显示“已复制到剪贴板”提示，3 秒后消失。",
        "按 ESC 或点击右上角关闭按钮退出历史面板。",
        "回到任意应用按 Ctrl+V，应粘贴刚选择的历史内容。",
      ],
    },
    C5: {
      title: "隐私与过滤",
      items: [
        "点击历史面板右上角齿轮配置短文本、密码样式、银行卡号和来源应用过滤。",
        "密码样式要求 6-30 位且同时包含字母和数字；银行卡支持 13-19 位并通过 Luhn 校验。",
        "保存后分别复制符合过滤条件的文本，它们不应进入历史列表。",
        "普通文本仍应正常记录；超过 10,000 字符的文本不记录。",
      ],
    },
    C6: {
      title: "自动清理",
      items: [
        "历史超过最大条数时按固定 500 条批次删除最早记录；超过 30 天的记录每小时自动清理。",
        "点击历史面板底部“清空”后显示完整的面板内确认框。",
        "点击“取消”不删除；点击“确认清空”后列表为空，面板保持打开。",
      ],
    },
    C7: {
      title: "快捷键入口",
      items: [
        "全局按 Ctrl+Shift+V 打开历史面板；重复按快捷键不会关闭面板。",
        "点击记录会复制该条，并在鼠标位置显示 3 秒提示。",
        "按 ESC 或点击右上角 × 关闭历史面板。",
      ],
    },
    C8: {
      title: "CLI 选项 --clipboard",
      items: [
        "用下面的命令构建 Weekly，启动后复制几段文本，再按 Ctrl+Shift+V。",
        "能打开历史面板并看到刚复制的内容，即表示 --clipboard 已生效。",
        "不带 --clipboard 构建时，不应注册剪贴板历史监听和快捷键。",
      ],
      command:
        'node dist/cli.js "https://weekly.tw93.fun/en" --name Weekly --clipboard',
    },
    C9: {
      title: "CLI 选项 --clipboard-max",
      items: [
        "用下面的命令把最大记录数设为 500；该选项需要与 --clipboard 一起使用。",
        "快速验证：历史面板底部容量上限应显示 500。",
        "完整验证：上限为 500 时写入第 501 条会固定删除最早 500 条，最终剩余 1 条。",
      ],
      command:
        'node dist/cli.js "https://weekly.tw93.fun/en" --name Weekly --clipboard --clipboard-max 500',
    },
  };

  function installSidebar() {
    if (!document.documentElement) return;
    if (document.getElementById("pake-clipboard-settings-sidebar")) return;

    const host = document.createElement("div");
    host.id = "pake-clipboard-settings-sidebar";
    host.style.cssText = [
      "position:fixed",
      "top:0",
      "right:0",
      "bottom:0",
      "z-index:2147483647",
      "display:none",
      "width:min(520px,100vw)",
      "height:100vh",
      "margin:0",
      "padding:0",
      "transition:width 180ms ease",
    ].join(";");

    const shadow = host.attachShadow({ mode: "closed" });
    shadow.innerHTML = `
      <style>
        * { box-sizing: border-box; }
        :host {
          color-scheme: light dark;
          font-family: Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
          color: #182235;
        }
        button { font: inherit; cursor: pointer; }
        .shell {
          display: grid;
          grid-template-rows: 64px minmax(0, 1fr);
          width: 100%;
          height: 100%;
          background: #fff;
          border-left: 1px solid #e3e8f1;
          box-shadow: -16px 0 42px rgba(20, 34, 58, 0.16);
        }
        .topbar {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 0 16px;
          border-bottom: 1px solid #e9edf4;
        }
        .brand-stack {
          display: grid;
          flex: 0 0 38px;
          gap: 2px;
          justify-items: center;
        }
        .brand {
          display: grid;
          width: 38px;
          height: 38px;
          place-items: center;
          color: #fff;
          font-weight: 800;
          background: linear-gradient(145deg, #6f7bff, #3c6ff3);
          border-radius: 12px;
        }
        .collapse-toggle {
          display: grid;
          width: 38px;
          height: 16px;
          place-items: center;
          padding: 0;
          color: #75839a;
          background: transparent;
          border: 0;
          border-radius: 5px;
          font-size: 10px;
          line-height: 1;
        }
        .collapse-toggle:hover { color: #2867ed; background: #eaf1ff; }
        .brand-copy strong, .brand-copy span { display: block; }
        .brand-copy strong { font-size: 15px; }
        .brand-copy span { margin-top: 2px; color: #8a96a9; font-size: 11px; }
        .workspace {
          display: grid;
          min-height: 0;
          grid-template-columns: 142px minmax(0, 1fr);
        }
        .navigation {
          padding: 16px 10px;
          background: #f7f9fc;
          border-right: 1px solid #e9edf4;
        }
        .nav-label {
          padding: 0 10px 8px;
          color: #9aa4b4;
          font-size: 10px;
          font-weight: 700;
          letter-spacing: .08em;
        }
        .nav-item {
          display: flex;
          align-items: center;
          gap: 8px;
          width: 100%;
          padding: 11px 10px;
          color: #2867ed;
          text-align: left;
          background: #eaf1ff;
          border: 0;
          border-radius: 10px;
          font-size: 12px;
          font-weight: 700;
        }
        .nav-note {
          margin: 14px 8px 0;
          color: #8490a2;
          font-size: 10px;
          line-height: 1.55;
        }
        .content { min-width: 0; overflow: auto; padding: 20px; background: #fff; }
        h1 { margin: 0; font-size: 22px; }
        .subtitle { margin: 6px 0 16px; color: #8a96a9; font-size: 11px; line-height: 1.5; }
        .feature-list { display: grid; gap: 8px; }
        .feature-item {
          display: grid;
          grid-template-columns: 34px minmax(0, 1fr) 18px;
          gap: 10px;
          align-items: center;
          width: 100%;
          padding: 11px;
          color: inherit;
          text-align: left;
          background: #fff;
          border: 1px solid #e4e9f1;
          border-radius: 11px;
        }
        .feature-item:hover, .feature-item.active {
          border-color: #9dbbfb;
          box-shadow: 0 4px 14px rgba(40, 91, 190, .1);
        }
        .feature-item.active { background: #f5f8ff; }
        .feature-code {
          padding: 4px 0;
          color: #2e68df;
          text-align: center;
          background: #edf3ff;
          border-radius: 7px;
          font-size: 10px;
          font-weight: 800;
        }
        .feature-name { font-size: 12px; font-weight: 700; }
        .arrow { color: #9aa5b5; font-size: 16px; }
        .rule-panel {
          margin-top: 14px;
          padding: 16px;
          background: #f7f9fc;
          border: 1px solid #e2e8f1;
          border-radius: 14px;
        }
        .rule-panel[hidden] { display: none; }
        .rule-heading { display: flex; gap: 8px; align-items: center; }
        .rule-heading h2 { margin: 0; font-size: 15px; }
        .rule-label { margin: 14px 0 7px; color: #526077; font-size: 11px; font-weight: 800; }
        .rule-list {
          display: grid;
          gap: 7px;
          margin: 0;
          padding-left: 18px;
          color: #58667a;
          font-size: 11px;
          line-height: 1.55;
        }
        .command {
          display: block;
          margin-top: 10px;
          padding: 9px;
          overflow: auto;
          color: #364760;
          background: #edf1f7;
          border-radius: 8px;
          font: 10px/1.5 Consolas, monospace;
          white-space: pre-wrap;
          overflow-wrap: anywhere;
        }
        .collapsed .shell { grid-template-rows: 100%; }
        .collapsed .topbar {
          align-items: flex-start;
          justify-content: center;
          padding: 12px 0 0;
          border-bottom: 0;
        }
        .collapsed .brand-copy, .collapsed .workspace { display: none; }
        @media (prefers-color-scheme: dark) {
          :host { color: #edf1f7; }
          .shell, .content, .feature-item { background: #1b1f27; }
          .topbar, .navigation, .feature-item, .rule-panel { border-color: #333a47; }
          .navigation { background: #161a21; }
          .feature-item.active, .rule-panel { background: #222833; }
          .nav-item, .feature-code { background: #223455; }
          .command { color: #e5eaf2; background: #181c23; }
        }
      </style>
      <section class="shell">
        <header class="topbar">
          <div class="brand-stack">
            <div class="brand">P</div>
            <button class="collapse-toggle" type="button" aria-label="收起边栏" title="收起边栏">▶</button>
          </div>
          <div class="brand-copy">
            <strong>Pake Plus</strong>
            <span>功能 C 手动测试面板</span>
          </div>
        </header>
        <div class="workspace">
          <aside class="navigation">
            <div class="nav-label">设置</div>
            <button class="nav-item" type="button"><span>▣</span><span>剪贴板管理</span></button>
            <p class="nav-note">点击“剪贴板管理”显示九项功能；点击具体功能只展示功能规则和测试方法。</p>
          </aside>
          <main class="content">
            <h1>剪贴板管理</h1>
            <p class="subtitle">选择 C1–C9 查看规则。此处不直接执行功能，避免误操作。</p>
            <div class="feature-list">
              <button class="feature-item" data-feature="C1" type="button"><span class="feature-code">C1</span><span class="feature-name">系统剪贴板自动监听</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C2" type="button"><span class="feature-code">C2</span><span class="feature-name">剪贴板历史面板</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C3" type="button"><span class="feature-code">C3</span><span class="feature-name">全文搜索</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C4" type="button"><span class="feature-code">C4</span><span class="feature-name">一键复制复用</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C5" type="button"><span class="feature-code">C5</span><span class="feature-name">隐私与过滤</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C6" type="button"><span class="feature-code">C6</span><span class="feature-name">自动清理</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C7" type="button"><span class="feature-code">C7</span><span class="feature-name">快捷键入口</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C8" type="button"><span class="feature-code">C8</span><span class="feature-name">CLI 选项 --clipboard</span><span class="arrow">›</span></button>
              <button class="feature-item" data-feature="C9" type="button"><span class="feature-code">C9</span><span class="feature-name">CLI 选项 --clipboard-max</span><span class="arrow">›</span></button>
            </div>
            <section class="rule-panel" hidden>
              <div class="rule-heading"><span class="rule-code feature-code"></span><h2 class="rule-title"></h2></div>
              <p class="rule-label">功能规则与测试方法</p>
              <ul class="rule-list"></ul>
              <code class="command" hidden></code>
            </section>
          </main>
        </div>
      </section>
    `;

    const shell = shadow.querySelector(".shell");
    const toggle = shadow.querySelector(".collapse-toggle");
    const content = shadow.querySelector(".content");
    const rulePanel = shadow.querySelector(".rule-panel");
    const ruleCode = shadow.querySelector(".rule-code");
    const ruleTitle = shadow.querySelector(".rule-title");
    const ruleList = shadow.querySelector(".rule-list");
    const command = shadow.querySelector(".command");
    let collapsed = false;
    let hidden = true;

    toggle.addEventListener("click", () => {
      collapsed = !collapsed;
      shell.classList.toggle("collapsed", collapsed);
      host.style.width = collapsed ? "72px" : "min(520px, 100vw)";
      toggle.textContent = collapsed ? "◀" : "▶";
      toggle.title = collapsed ? "展开边栏" : "收起边栏";
      toggle.setAttribute("aria-label", toggle.title);
    });

    shadow.querySelector(".nav-item").addEventListener("click", () => {
      content.scrollTo({ top: 0, behavior: "smooth" });
    });

    shadow.querySelectorAll("[data-feature]").forEach((button) => {
      button.addEventListener("click", () => {
        const feature = button.dataset.feature;
        const rule = rules[feature];
        shadow.querySelectorAll("[data-feature]").forEach((item) => {
          item.classList.toggle("active", item === button);
        });
        ruleCode.textContent = feature;
        ruleTitle.textContent = rule.title;
        ruleList.replaceChildren(
          ...rule.items.map((item) => {
            const entry = document.createElement("li");
            entry.textContent = item;
            return entry;
          }),
        );
        command.hidden = !rule.command;
        command.textContent = rule.command || "";
        rulePanel.hidden = false;
        rulePanel.scrollIntoView({ behavior: "smooth", block: "nearest" });
      });
    });

    document.documentElement.append(host);

    window.__pakeToggleClipboardSidebar = function () {
      hidden = !hidden;
      if (hidden) {
        host.style.display = "none";
      } else {
        host.style.display = "block";
        collapsed = false;
        shell.classList.remove("collapsed");
        host.style.width = "min(520px, 100vw)";
        toggle.textContent = "▶";
        toggle.title = "收起边栏";
        toggle.setAttribute("aria-label", toggle.title);
      }
      return !hidden;
    };
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", installSidebar, {
      once: true,
    });
  } else {
    installSidebar();
  }
})();
