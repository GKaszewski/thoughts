"use client";

import { useEffect, useRef } from "react";

const FEATURES = [
  {
    title: "Say it in 128",
    body: "Short, focused thoughts. No bloat, no essays.",
  },
  {
    title: "Make it yours",
    body: "Customize your profile with CSS. Full creative control.",
  },
  {
    title: "Your audience, your rules",
    body: "Public, followers-only, unlisted, or direct. You pick for each post.",
  },
  {
    title: "Movies Diary",
    body: "Your Movies Diary posts show up as rich cards with ratings and posters. Feels native.",
  },
];

export function LandingFeatures() {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const cards = Array.from(
      ref.current?.querySelectorAll("[data-animate]") ?? []
    );
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("visible");
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.15 }
    );
    cards.forEach((card) => observer.observe(card));
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={ref} className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-8">
      {FEATURES.map((f, i) => (
        <div
          key={f.title}
          data-animate
          className="landing-card-animate glass-effect rounded-xl p-6 shadow-fa-md"
          style={{ animationDelay: `${i * 100}ms` }}
        >
          <h3 className="font-bold text-base mb-1">{f.title}</h3>
          <p className="text-sm text-muted-foreground">{f.body}</p>
        </div>
      ))}
    </div>
  );
}
