import Link from "next/link";
import { ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { LandingFeatures } from "./landing-features";

export function LandingPage() {
  return (
    <div className="min-h-screen relative overflow-hidden font-sans">
      {/* Background blur overlay */}
      <div className="fixed inset-0 bg-white/30 backdrop-blur-sm -z-[5]" />
      {/* Ambient orbs */}
      <div
        className="landing-orb"
        style={{
          width: 280, height: 280,
          background: "radial-gradient(circle, rgba(147,210,255,0.55) 0%, transparent 70%)",
          top: -80, left: -60,
          "--orb-duration": "16s",
          "--orb-delay": "0s",
        } as React.CSSProperties}
      />
      <div
        className="landing-orb"
        style={{
          width: 220, height: 220,
          background: "radial-gradient(circle, rgba(134,239,172,0.5) 0%, transparent 70%)",
          bottom: "10%", right: "5%",
          "--orb-duration": "20s",
          "--orb-delay": "-5s",
          animationDirection: "reverse",
        } as React.CSSProperties}
      />
      <div
        className="landing-orb"
        style={{
          width: 160, height: 160,
          background: "radial-gradient(circle, rgba(196,181,253,0.5) 0%, transparent 70%)",
          top: "35%", left: "65%",
          "--orb-duration": "14s",
          "--orb-delay": "-8s",
        } as React.CSSProperties}
      />
      <div
        className="landing-orb hidden sm:block"
        style={{
          width: 200, height: 200,
          background: "radial-gradient(circle, rgba(253,186,116,0.3) 0%, transparent 70%)",
          top: "60%", left: "10%",
          "--orb-duration": "18s",
          "--orb-delay": "-3s",
          animationDirection: "reverse",
        } as React.CSSProperties}
      />
      <div
        className="landing-orb hidden lg:block"
        style={{
          width: 140, height: 140,
          background: "radial-gradient(circle, rgba(167,243,208,0.45) 0%, transparent 70%)",
          top: "20%", right: "20%",
          "--orb-duration": "12s",
          "--orb-delay": "-10s",
        } as React.CSSProperties}
      />

      {/* ── Section 1: Hero ── */}
      <section className="relative z-10 flex items-center justify-center min-h-screen px-4 py-16">
        <div className="flex flex-col items-center gap-6 w-full max-w-md mx-auto">
        <div className="landing-hero-card glass-effect rounded-2xl shadow-fa-lg p-8 sm:p-12 text-center w-full">
          <h1 className="text-4xl sm:text-5xl font-black tracking-tight mb-3 text-shadow-sm">
            Thoughts
          </h1>
          <p className="text-muted-foreground text-base sm:text-lg mb-8">
            128 characters. No algorithms. Your web.
          </p>
          <div className="flex flex-col sm:flex-row gap-3 justify-center mb-6 relative z-10">
            <Button
              asChild
              size="lg"
              className="landing-cta rounded-full border-0 text-white"
            >
              <Link href="/register">Register</Link>
            </Button>
            <Button asChild size="lg" variant="outline" className="rounded-full">
              <Link href="/login">Sign in</Link>
            </Button>
          </div>
          <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground relative z-10">
            <span className="landing-badge-dot" />
            Works with Mastodon, Pixelfed &amp; more
          </div>
        </div>
        <ChevronDown
          className="animate-float-bob text-muted-foreground/60"
          size={28}
          strokeWidth={1.5}
        />
        </div>
      </section>

      {/* ── Section 2: Features ── */}
      <section className="relative z-10 container mx-auto max-w-3xl px-4 py-16">
        <p className="text-xs font-semibold uppercase tracking-widest text-muted-foreground text-center mb-2">
          What you can do
        </p>
        <LandingFeatures />
      </section>

      {/* ── Section 3: Fediverse ── */}
      <section className="relative z-10 container mx-auto max-w-2xl px-4 py-16">
        <div className="glass-effect rounded-2xl p-8 shadow-fa-md border border-sky-200/30">
          <p className="text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-2">
            Part of something bigger
          </p>
          <h2 className="text-2xl font-bold mb-4">
            Thoughts speaks ActivityPub
          </h2>
          <p className="text-muted-foreground text-sm leading-relaxed mb-6">
            Follow and be followed by anyone on Mastodon, Pixelfed, or any
            ActivityPub-compatible platform. Your thoughts travel across the
            open web.
          </p>
          <div className="flex flex-wrap gap-2 mb-6">
            {["Mastodon", "Pixelfed"].map((label) => (
              <span
                key={label}
                className="px-3 py-1 rounded-full text-xs font-medium bg-white/40 border border-white/60 hover:shadow-fa-sm transition-shadow cursor-default"
              >
                {label}
              </span>
            ))}
          </div>
          <Link
            href="/about/fediverse"
            className="text-sm text-primary hover:underline inline-flex items-center gap-1"
          >
            What is the Fediverse? →
          </Link>
        </div>
      </section>

      {/* ── Section 4: Vision ── */}
      <section className="relative z-10 container mx-auto max-w-xl px-4 py-16 text-center">
        <p className="text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-3">
          Why we built this
        </p>
        <h2 className="text-2xl sm:text-3xl font-bold mb-6">
          The web used to feel human
        </h2>
        <p className="text-muted-foreground leading-relaxed mb-6">
          No algorithm feeds. No ads. Just a timeline of people you actually
          follow, on a profile you can make look however you want.
        </p>
        <p className="text-sm italic text-muted-foreground">
          That version of the web still exists.
        </p>
      </section>

      {/* ── Section 5: Footer CTA ── */}
      <section className="relative z-10 container mx-auto max-w-md px-4 py-16 text-center">
        <div className="landing-hero-card glass-effect rounded-2xl p-8 shadow-fa-lg">
          <h2 className="text-xl font-bold mb-6">Ready to join?</h2>
          <Button
            asChild
            size="lg"
            className="landing-cta rounded-full border-0 text-white w-full mb-3"
          >
            <Link href="/register">Register</Link>
          </Button>
          <p className="text-sm text-muted-foreground relative z-10">
            Already have an account?{" "}
            <Link href="/login" className="text-primary hover:underline">
              Sign in
            </Link>
          </p>
        </div>
      </section>
    </div>
  );
}
