"use client";

import { useEffect, useRef, useState } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { css } from "@codemirror/lang-css";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ExternalLink } from "lucide-react";

interface CssEditorProps {
  value?: string;
  onChange?: (value: string) => void;
  username: string;
}

export function CssEditor({ value = "", onChange, username }: CssEditorProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [iframeReady, setIframeReady] = useState(false);

  useEffect(() => {
    if (!iframeReady) return;
    const iframe = iframeRef.current;
    if (!iframe) return;
    const timer = setTimeout(() => {
      iframe.contentWindow?.postMessage(
        { type: "css-preview", css: value },
        window.location.origin
      );
    }, 150);
    return () => clearTimeout(timer);
  }, [value, iframeReady]);

  return (
    <div className="rounded-lg border bg-card overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <span className="text-sm font-medium">Custom CSS</span>
        <a
          href="/docs/css"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
        >
          <ExternalLink className="h-3 w-3" />
          CSS Reference
        </a>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="edit">
        <TabsList className="w-full justify-start rounded-none border-b h-auto p-0 bg-transparent">
          <TabsTrigger
            value="edit"
            className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent px-4 py-2 text-sm"
          >
            Edit
          </TabsTrigger>
          <TabsTrigger
            value="preview"
            className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent px-4 py-2 text-sm"
          >
            Preview
          </TabsTrigger>
        </TabsList>

        <TabsContent value="edit" className="mt-0">
          <CodeMirror
            value={value}
            height="280px"
            extensions={[css()]}
            theme="dark"
            onChange={(val) => onChange?.(val)}
            basicSetup={{
              lineNumbers: true,
              foldGutter: false,
              dropCursor: false,
              allowMultipleSelections: false,
              indentOnInput: true,
            }}
          />
        </TabsContent>

        <TabsContent value="preview" className="mt-0 data-[state=inactive]:hidden" forceMount>
          <div className="relative">
            <iframe
              ref={iframeRef}
              src={`/users/${username}`}
              onLoad={() => setIframeReady(true)}
              className="w-full border-0"
              style={{ height: "560px" }}
              title="Profile preview"
            />
            <div className="absolute top-2 right-2 text-xs text-muted-foreground bg-background/80 px-2 py-1 rounded">
              Live preview
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
