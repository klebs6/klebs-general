(function () {
  try {
    if (window.__rustBridgeInstalled) return;
    window.__rustBridgeInstalled = true;

    function post(type, payload) {
      try {
        window.webkit.messageHandlers.rust.postMessage(
          String(type) + "\n" + String(payload ?? "")
        );
      } catch (e) {}
    }

    function info(msg) { post("bridge_info", msg); }
    function err(msg) { post("error", msg); }

    function setTextareaValue(el, value) {
      const proto = Object.getPrototypeOf(el);
      const desc = Object.getOwnPropertyDescriptor(proto, "value");
      if (desc && desc.set) desc.set.call(el, value);
      else el.value = value;
      el.dispatchEvent(new Event("input", { bubbles: true }));
      el.dispatchEvent(new Event("change", { bubbles: true }));
    }

    function findComposer() {
      return document.querySelector("#prompt-textarea")
          || document.querySelector("textarea[placeholder]")
          || document.querySelector("textarea");
    }

    function findSendButton() {
      return document.querySelector('button[data-testid="send-button"]')
          || document.querySelector('button[aria-label*="Send"]')
          || document.querySelector('button[type="submit"]');
    }

    async function send(text) {
      try {
        const ta = findComposer();
        if (!ta) { err("composer textarea not found"); return false; }

        ta.focus();
        setTextareaValue(ta, text);

        await new Promise(r => setTimeout(r, 20));

        const btn = findSendButton();
        if (btn) {
          btn.click();
          post("user_send", text);
          return true;
        }

        ta.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", code: "Enter", bubbles: true }));
        ta.dispatchEvent(new KeyboardEvent("keyup",   { key: "Enter", code: "Enter", bubbles: true }));
        post("user_send", text);
        return true;
      } catch (e) {
        err("send exception: " + e);
        return false;
      }
    }

    function findLastAssistantNode() {
      const nodes = Array.from(document.querySelectorAll('[data-message-author-role="assistant"]'));
      if (!nodes.length) return null;
      const last = nodes[nodes.length - 1];
      return last.querySelector(".markdown") || last;
    }

    // Streaming: observe DOM and emit deltas for the latest assistant message.
    let lastNode = null;
    let lastText = "";
    let idleTimer = null;
    let scheduled = false;

    function updateAssistant() {
      const node = findLastAssistantNode();
      if (!node) return;

      const text = (node.innerText || "").replace(/\s+$/g, ""); // trim end only

      if (node !== lastNode) {
        lastNode = node;
        lastText = "";
        post("assistant_start", "");
      }

      if (text === lastText) return;

      const delta = text.startsWith(lastText) ? text.slice(lastText.length) : text;
      lastText = text;

      if (delta) post("assistant_delta", delta);

      if (idleTimer) clearTimeout(idleTimer);
      idleTimer = setTimeout(() => {
        post("assistant_done", lastText);
      }, 900);
    }

    function scheduleUpdate() {
      if (scheduled) return;
      scheduled = true;
      requestAnimationFrame(() => {
        scheduled = false;
        updateAssistant();
      });
    }

    const obs = new MutationObserver(() => scheduleUpdate());
    obs.observe(document.documentElement || document.body, {
      subtree: true,
      childList: true,
      characterData: true
    });

    window.__rustBridge = { send };

    post("bridge_ready", location.href);
    info("shim installed @ " + location.href);

    window.addEventListener("load", () => {
      info("window load title=" + document.title);
      scheduleUpdate();
    }, { once: true });
  } catch (e) {}
})();

