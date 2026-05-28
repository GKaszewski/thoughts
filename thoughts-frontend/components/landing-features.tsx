"use client";

import { useEffect, useRef } from "react";

const FEATURES = [
  {
    icon: "✍️",
    title: "Say it in 128",
    body: "Short, focused thoughts. No bloat, no essays.",
  },
  {
    icon: "🎨",
    title: "Make it yours",
    body: "Customize your profile with CSS. Full creative control.",
  },
  {
    icon: "🔒",
    title: "Your audience, your rules",
    body: "Public, followers-only, unlisted, or direct — you decide every time.",
  },
  {
    icon: "🎬",
    title: "Movies Diary",
    body: "Posts from your Movies Diary appear as rich cards — ratings, posters, reviews. First-class, not an afterthought.",
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
          <div className="text-3xl mb-3">{f.icon}</div>
          <h3 className="font-bold text-base mb-1">{f.title}</h3>
          <p className="text-sm text-muted-foreground">{f.body}</p>
        </div>
      ))}
    </div>
  );
}
