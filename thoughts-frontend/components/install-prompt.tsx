"use client";

import React, { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Download } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardAction,
} from "@/components/ui/card";
import Cookies from "js-cookie";

interface CustomWindow extends Window {
  MSStream?: unknown;
}

interface BeforeInstallPromptEvent extends Event {
  prompt: () => void;
  userChoice: Promise<{ outcome: "accepted" | "dismissed"; platform: string }>;
}

export default function InstallPrompt() {
  const [isIOS, setIsIOS] = useState(false);
  const [isStandalone, setIsStandalone] = useState(false);
  const [deferredPrompt, setDeferredPrompt] =
    useState<BeforeInstallPromptEvent | null>(null);
  const [isDismissed, setIsDismissed] = useState(false);

  useEffect(() => {
    const customWindow = window as CustomWindow;
    setIsIOS(
      /iPad|iPhone|iPod/.test(navigator.userAgent) && !customWindow.MSStream
    );
    setIsStandalone(window.matchMedia("(display-mode: standalone)").matches);

    const dismissed = Cookies.get("install_prompt_dismissed");
    if (dismissed) {
      setIsDismissed(true);
    }

    const handleBeforeInstallPrompt = (e: Event) => {
      e.preventDefault();
      setDeferredPrompt(e as BeforeInstallPromptEvent);
    };

    window.addEventListener("beforeinstallprompt", handleBeforeInstallPrompt);

    return () => {
      window.removeEventListener(
        "beforeinstallprompt",
        handleBeforeInstallPrompt
      );
    };
  }, []);

  const handleInstallClick = async () => {
    if (!deferredPrompt) return;
    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;
    if (outcome === "accepted") {
      console.log("User accepted the install prompt");
    } else {
      console.log("User dismissed the install prompt");
      Cookies.set("install_prompt_dismissed", "true", { expires: 7 });
    }
    setDeferredPrompt(null);
  };

  const handleCloseClick = () => {
    setIsStandalone(true);
    Cookies.set("install_prompt_dismissed", "true", { expires: 7 });
  };

  if (isStandalone || (!isIOS && !deferredPrompt) || isDismissed) {
    return null;
  }

  return (
    <div className="fixed bottom-0 z-50">
      <Card className="w-full max-w-sm glass-effect glossy-effect bottom shadow-fa-lg">
        <CardHeader>
          <CardTitle>Install Thoughts</CardTitle>
          <CardDescription>
            Get the full app experience on your device.
          </CardDescription>
          <CardAction>
            <Button
              size="sm"
              variant="ghost"
              className="absolute top-2 right-2"
              onClick={handleCloseClick}
            >
              &times;
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          {!isIOS && deferredPrompt && (
            <Button className="w-full" onClick={handleInstallClick}>
              <Download className="mr-2 h-4 w-4" />
              Add to Home Screen
            </Button>
          )}
          {isIOS && (
            <p className="text-sm text-muted-foreground">
              To install, tap the Share icon
              <span className="mx-1 text-lg">⎋</span>
              and then &quot;Add to Home Screen&quot;
              <span className="mx-1 text-lg">➕</span>.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
