"use client";

import React, { useEffect, useRef } from "react";
import * as Tone from "tone";

interface ConfettiProps {
  fire: boolean;
  onComplete: () => void;
}

const colors = ["#26ccff", "#a25afd", "#ff5e7e", "#88ff5a", "#fcff42"];

export function Confetti({ fire, onComplete }: ConfettiProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (fire) {
      const canvas = canvasRef.current;
      if (!canvas) return;

      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      const synth = new Tone.PolySynth(Tone.Synth, {
        oscillator: { type: "sine" },
        envelope: { attack: 0.005, decay: 0.1, sustain: 0.3, release: 1 },
      }).toDestination();

      const notes = ["C4", "E4", "G4", "A4"];

      let animationFrameId: number;
      const confetti: {
        x: number;
        y: number;
        r: number;
        d: number;
        color: string;
        tilt: number;
      }[] = [];
      const numConfetti = 100;

      const resizeCanvas = () => {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
      };
      window.addEventListener("resize", resizeCanvas);
      resizeCanvas();

      for (let i = 0; i < numConfetti; i++) {
        confetti.push({
          x: Math.random() * canvas.width,
          y: -20,
          r: Math.random() * 6 + 1,
          d: Math.random() * numConfetti,
          color: colors[Math.floor(Math.random() * colors.length)],
          tilt: Math.floor(Math.random() * 10) - 10,
        });
      }

      let animationFinished = false;

      const draw = () => {
        if (animationFinished) return;
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        let allOffScreen = true;

        for (let i = 0; i < numConfetti; i++) {
          const c = confetti[i];
          ctx.beginPath();
          ctx.lineWidth = c.r / 2;
          ctx.strokeStyle = c.color;
          ctx.moveTo(c.x + c.tilt, c.y);
          ctx.lineTo(c.x, c.y + c.tilt + c.r);
          ctx.stroke();

          c.y += Math.cos(c.d + i + 1.2) + 1.5 + c.r / 2;
          c.x += Math.sin(i) * 1.5;

          if (c.y <= canvas.height) {
            allOffScreen = false;
          }
        }

        if (allOffScreen) {
          animationFinished = true;
          onComplete();
        } else {
          animationFrameId = requestAnimationFrame(draw);
        }
      };

      try {
        Tone.start();
        const now = Tone.now();
        notes.forEach((note, i) => {
          synth.triggerAttackRelease(note, "8n", now + i * 0.1);
        });
        draw();
      } catch (error) {
        console.error("Audio could not be started", error);
        draw();
      }

      return () => {
        window.removeEventListener("resize", resizeCanvas);
        cancelAnimationFrame(animationFrameId);
      };
    }
  }, [fire, onComplete]);

  if (!fire) return null;

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        pointerEvents: "none",
        zIndex: 9999,
      }}
    />
  );
}
