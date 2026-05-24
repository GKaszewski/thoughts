"use client";

import { useEffect } from "react";

export function CssPreviewListener() {
  useEffect(() => {
    if (window.self === window.top) return;

    function handler(e: MessageEvent) {
      if (e.origin !== window.location.origin) return;
      if (!e.data || e.data.type !== "css-preview") return;

      let el = document.getElementById("css-preview-style") as HTMLStyleElement | null;
      if (!el) {
        el = document.createElement("style");
        el.id = "css-preview-style";
        document.head.appendChild(el);
      }
      el.textContent = typeof e.data.css === "string" ? e.data.css : "";
    }

    window.addEventListener("message", handler);
    return () => window.removeEventListener("message", handler);
  }, []);

  return null;
}
