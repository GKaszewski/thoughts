import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "CSS Customization Reference",
  description: "All the IDs, variables, and examples you need to style your Thoughts profile.",
};

const EXAMPLE_THEMES = [
  {
    name: "Vaporwave",
    description: "Pink/purple gradient header with a glassy card",
    css: `#profile-header {
  background: linear-gradient(135deg, #ff6b9d 0%, #c44dff 100%);
}

#profile-card {
  background: rgba(255, 107, 157, 0.08);
  border: 1px solid rgba(196, 77, 255, 0.3);
  backdrop-filter: blur(20px);
}`,
  },
  {
    name: "Dark Minimal",
    description: "Pitch black, thin border, no blur",
    css: `#main-container {
  --background: 0 0% 4%;
  --card: 0 0% 7%;
  --border: 0 0% 14%;
}

#profile-header {
  background: #0a0a0a;
  border-bottom: 1px solid #1a1a1a;
}`,
  },
  {
    name: "Neon Glow",
    description: "Electric green borders and glow on a dark background",
    css: `#profile-header {
  background: #050505;
  border-bottom: 2px solid #00ff88;
}

#profile-card {
  border: 1px solid #00ff88;
  box-shadow: 0 0 24px rgba(0, 255, 136, 0.25);
  background: rgba(0, 255, 136, 0.04);
}`,
  },
  {
    name: "Pastel",
    description: "Soft pink and lavender, gentle and rounded",
    css: `#profile-header {
  background: linear-gradient(135deg, #ffd6e7 0%, #c7ceea 100%);
}

#profile-card {
  background: rgba(255, 214, 231, 0.25);
  border: 1px solid #ffd6e7;
  border-radius: 1.5rem;
}`,
  },
  {
    name: "Retro Terminal",
    description: "Green-on-black monospace, old school CRT feel",
    css: `#main-container {
  --background: 0 0% 4%;
  --foreground: 120 100% 70%;
  --card: 0 0% 6%;
  --border: 120 60% 20%;
  font-family: 'Courier New', monospace;
}

#profile-header {
  background: #000;
  border-bottom: 2px solid #00cc44;
}

#profile-card {
  border: 1px solid #00cc44;
  box-shadow: 0 0 8px rgba(0, 204, 68, 0.3);
}`,
  },
];

const IDS = [
  { id: "#profile-page-{username}", description: "Root element — scope all rules here to avoid leaking to other pages" },
  { id: "#profile-header", description: "Banner image strip at the top of the page" },
  { id: "#main-container", description: "Full-width grid wrapper — the whole page body" },
  { id: "#left-sidebar", description: "The <aside> wrapping the entire left column" },
  { id: "#left-sidebar__inner", description: "Sticky inner wrapper (profile card + top friends)" },
  { id: "#profile-card", description: "Profile sidebar card" },
  { id: "#profile-card__inner", description: "Flex row containing avatar + follow/edit button" },
  { id: "#profile-card__avatar", description: "Avatar section wrapper" },
  { id: "#profile-card__avatar-image", description: "Avatar circle container" },
  { id: "#profile-card__action", description: "Follow / Edit profile button container" },
  { id: "#profile-card__info", description: "Display name and @username block" },
  { id: "#profile-card__name", description: "Display name <h1>" },
  { id: "#profile-card__username", description: "@username paragraph" },
  { id: "#profile-card__bio", description: "Bio paragraph" },
  { id: "#profile-card__stats", description: "Following / Followers count row" },
  { id: "#profile-card__joined", description: "Joined date row" },
  { id: "#profile-card__thoughts", description: "Right column containing the thoughts feed" },
  { id: "#top-friends", description: "Top Friends card" },
  { id: "#top-friends__header", description: "Top Friends card header" },
  { id: "#top-friends__title", description: "\"Top Friends\" title" },
  { id: "#top-friends__content", description: "Top Friends avatar list" },
];

const CLASSES = [
  { name: ".glass-effect", description: "Semi-transparent card with backdrop blur and border — applied to #profile-card by default" },
  { name: ".gloss-highlight", description: "Adds Frutiger Aero gloss shimmer via ::before pseudo-element — add to any card" },
  { name: ".avatar-gradient", description: "Gradient fallback background + ring applied to the avatar circle" },
  { name: ".profile-header", description: "Class on the banner div (same element as #profile-header)" },
];

const VARIABLES = [
  { name: "--background", description: "Page background colour" },
  { name: "--foreground", description: "Default text colour" },
  { name: "--card", description: "Card background colour" },
  { name: "--card-foreground", description: "Text colour inside cards" },
  { name: "--primary", description: "Primary accent colour (buttons, active tabs)" },
  { name: "--muted", description: "Muted background (used for secondary sections)" },
  { name: "--muted-foreground", description: "Secondary text colour" },
  { name: "--border", description: "Border colour" },
  { name: "--radius", description: "Base border radius for all rounded elements" },
];

const AERO_VARIABLES = [
  { name: "--shadow-fa-sm", description: "Small Frutiger Aero layered box-shadow" },
  { name: "--shadow-fa-md", description: "Medium Frutiger Aero layered box-shadow" },
  { name: "--shadow-fa-lg", description: "Large Frutiger Aero layered box-shadow — used on cards" },
  { name: "--gradient-fa-blue", description: "Signature blue gradient direction (135deg, sky → cyan)" },
  { name: "--gradient-fa-green", description: "Green gradient variant" },
  { name: "--text-shadow-default", description: "Subtle dark text shadow" },
  { name: "--text-shadow-sm", description: "Light text shadow (white highlight)" },
  { name: "--text-shadow-md", description: "Medium dark text shadow" },
  { name: "--text-shadow-lg", description: "Strong dark text shadow" },
];

export default function CssReferencePage() {
  return (
    <div className="container mx-auto max-w-3xl px-4 py-12 space-y-12">
      <div>
        <h1 className="text-3xl font-bold mb-2">CSS Customization Reference</h1>
        <p className="text-muted-foreground">
          Style every part of your profile page. Write CSS in{" "}
          <a href="/settings/profile" className="underline hover:text-foreground">
            Settings → Profile
          </a>{" "}
          and preview it live before saving.
        </p>
      </div>

      {/* Targetable IDs */}
      <section>
        <h2 className="text-xl font-semibold mb-1">Targetable IDs</h2>
        <p className="text-sm text-muted-foreground mb-4">
          Use these IDs in your CSS selectors to target specific elements on your profile page.
          Prefix with <code className="bg-muted px-1 rounded">#profile-page-yourusername</code> to
          avoid affecting other pages.
        </p>
        <div className="overflow-hidden rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted">
              <tr>
                <th className="text-left px-4 py-2 font-medium">Selector</th>
                <th className="text-left px-4 py-2 font-medium">Element</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {IDS.map(({ id, description }) => (
                <tr key={id} className="odd:bg-muted/60 even:bg-muted/80 transition-colors">
                  <td className="px-4 py-2 font-mono text-xs text-primary">{id}</td>
                  <td className="px-4 py-2 text-muted-foreground">{description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Custom Classes */}
      <section>
        <h2 className="text-xl font-semibold mb-1">Custom Classes</h2>
        <p className="text-sm text-muted-foreground mb-4">
          These classes are defined globally and can be added to or overridden for any element.
        </p>
        <div className="overflow-hidden rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted">
              <tr>
                <th className="text-left px-4 py-2 font-medium">Class</th>
                <th className="text-left px-4 py-2 font-medium">Effect</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {CLASSES.map(({ name, description }) => (
                <tr key={name} className="odd:bg-muted/60 even:bg-muted/80 transition-colors">
                  <td className="px-4 py-2 font-mono text-xs text-primary">{name}</td>
                  <td className="px-4 py-2 text-muted-foreground">{description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* CSS Variables */}
      <section>
        <h2 className="text-xl font-semibold mb-1">Theme Variables</h2>
        <p className="text-sm text-muted-foreground mb-4">
          Override these tokens to change colours globally. Set them on{" "}
          <code className="bg-muted px-1 rounded">#main-container</code> or{" "}
          <code className="bg-muted px-1 rounded">:root</code>. Values are bare HSL triplets —
          for example <code className="bg-muted px-1 rounded">--background: 220 14% 96%</code>{" "}
          (hue saturation lightness, consumed as{" "}
          <code className="bg-muted px-1 rounded">hsl(var(--background))</code>).
        </p>
        <div className="overflow-hidden rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted">
              <tr>
                <th className="text-left px-4 py-2 font-medium">Variable</th>
                <th className="text-left px-4 py-2 font-medium">Controls</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {VARIABLES.map(({ name, description }) => (
                <tr key={name} className="odd:bg-muted/60 even:bg-muted/80 transition-colors">
                  <td className="px-4 py-2 font-mono text-xs text-primary">{name}</td>
                  <td className="px-4 py-2 text-muted-foreground">{description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Frutiger Aero Variables */}
      <section>
        <h2 className="text-xl font-semibold mb-1">Frutiger Aero Variables</h2>
        <p className="text-sm text-muted-foreground mb-4">
          The design system defines these extra variables for shadows, gradients, and text effects.
          Use them with <code className="bg-muted px-1 rounded">var()</code> in your CSS.
        </p>
        <div className="overflow-hidden rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted">
              <tr>
                <th className="text-left px-4 py-2 font-medium">Variable</th>
                <th className="text-left px-4 py-2 font-medium">Effect</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {AERO_VARIABLES.map(({ name, description }) => (
                <tr key={name} className="odd:bg-muted/60 even:bg-muted/80 transition-colors">
                  <td className="px-4 py-2 font-mono text-xs text-primary">{name}</td>
                  <td className="px-4 py-2 text-muted-foreground">{description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Example Themes */}
      <section>
        <h2 className="text-xl font-semibold mb-1">Example Themes</h2>
        <p className="text-sm text-muted-foreground mb-4">
          Copy and paste any of these into the editor as a starting point.
        </p>
        <div className="space-y-6">
          {EXAMPLE_THEMES.map(({ name, description, css }) => (
            <div key={name} className="rounded-lg border overflow-hidden">
              <div className="px-4 py-3 border-b bg-muted/50">
                <p className="font-medium text-sm">{name}</p>
                <p className="text-xs text-muted-foreground">{description}</p>
              </div>
              <pre className="p-4 text-xs overflow-x-auto bg-[#0d1117] text-[#e6edf3]">
                <code>{css}</code>
              </pre>
            </div>
          ))}
        </div>
      </section>

      {/* Tips */}
      <section>
        <h2 className="text-xl font-semibold mb-3">Tips</h2>
        <ul className="space-y-2 text-sm text-muted-foreground list-disc list-inside">
          <li>
            Scope all your rules to{" "}
            <code className="bg-muted px-1 rounded text-foreground">#profile-page-yourusername</code>{" "}
            to prevent them leaking to other pages.
          </li>
          <li>
            Use <code className="bg-muted px-1 rounded text-foreground">var(--card)</code> instead of
            hardcoded colours so your theme adapts to light and dark mode.
          </li>
          <li>
            The layout stacks to a single column below{" "}
            <code className="bg-muted px-1 rounded text-foreground">1024px</code>. Use{" "}
            <code className="bg-muted px-1 rounded text-foreground">@media (max-width: 1024px)</code>{" "}
            to adjust for mobile.
          </li>
          <li>
            Hex, rgb, and hsl all work. For theme token overrides, use bare HSL triplets:{" "}
            <code className="bg-muted px-1 rounded text-foreground">--card: 220 14% 12%</code>.
          </li>
        </ul>
      </section>
    </div>
  );
}
