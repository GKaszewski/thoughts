import type { Metadata } from "next";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";

export const metadata: Metadata = {
  title: "About the Fediverse",
  description:
    "Learn what the fediverse is, how ActivityPub works, and how to connect with Thoughts users from any compatible app.",
};

const SECTIONS = [
  {
    value: "what-is-fediverse",
    title: "What is the Fediverse?",
    content:
      "The fediverse is a network of independent social apps that talk to each other using shared protocols. Think of it like email — you can send from Gmail to Yahoo because both support the same standards. In the fediverse, people on Mastodon can follow people on Thoughts, Pixelfed, or any other compatible app. No single company owns it; each server runs independently.",
  },
  {
    value: "what-is-activitypub",
    title: "What is ActivityPub?",
    content:
      "ActivityPub is the open protocol (a W3C standard) that makes the fediverse possible. It defines how servers send posts, follows, likes, and other interactions to each other. Thoughts is built on ActivityPub, which means anyone on a compatible app can follow and interact with users here.",
  },
  {
    value: "your-handle",
    title: "Your handle explained",
    content:
      "Your full fediverse handle looks like @username@thoughts.gabrielkaszewski.dev. The first part is your username on this server; the second part is the server itself. When someone on another fediverse app wants to find you, they type your full handle into their search bar — exactly like an email address.",
  },
  {
    value: "how-to-follow",
    title: "How to follow someone here from another app",
    content:
      "In your fediverse app (Mastodon, Pixelfed, Akkoma, etc.), open the search and type the full handle: @username@thoughts.gabrielkaszewski.dev. Hit follow. Their posts will appear in your home feed. You don't need a Thoughts account to follow someone here.",
  },
  {
    value: "what-you-can-do",
    title: "What you can do from other apps",
    content:
      "From any compatible fediverse app you can follow Thoughts users and reply to their posts — both work across the network. Thoughts is focused on short-form text, so the experience is intentionally simple.",
  },
  {
    value: "how-thoughts-is-different",
    title: "How Thoughts is different",
    content:
      "Thoughts is deliberately text-only. Posts from local users are capped at 128 characters. There are no polls, DMs, or media uploads — by design, not limitation. When posts arrive from other instances they are displayed as text; media attachments are noted but not shown. The goal is a fast, focused reading experience.",
  },
  {
    value: "movies-diary",
    title: "Special integration: movies.gabrielkaszewski.dev",
    content:
      "Thoughts has first-class support for movies-diary, a companion fediverse app for logging films. When someone you follow on movies-diary posts a review, rating, or watchlist entry, it appears in your Thoughts feed as a rich card — poster, title, year, and rating — rather than raw text. Just follow their movies-diary handle like any other fediverse account and it works automatically.",
  },
] as const;

export default function FediversePage() {
  return (
    <div className="container mx-auto max-w-2xl px-4 py-12">
      <h1 className="text-3xl font-bold mb-2">The Fediverse</h1>
      <p className="text-muted-foreground mb-8">
        Thoughts is part of an open, decentralised social network. Here&apos;s how it works.
      </p>

      <Accordion type="multiple" className="space-y-3">
        {SECTIONS.map(({ value, title, content }) => (
          <div
            key={value}
            className="bg-card/80 backdrop-blur-lg rounded-xl border border-white/10 shadow overflow-hidden"
          >
            <AccordionItem value={value} className="border-0 px-4">
              <AccordionTrigger className="text-base font-semibold hover:no-underline">
                {title}
              </AccordionTrigger>
              <AccordionContent>
                <p className="text-sm text-foreground/80 leading-relaxed">
                  {content}
                </p>
              </AccordionContent>
            </AccordionItem>
          </div>
        ))}
      </Accordion>
    </div>
  );
}
