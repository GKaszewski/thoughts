import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function isApRequest(accept: string): boolean {
  return (
    accept.includes("application/activity+json") ||
    accept.includes("application/ld+json")
  );
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const parts = pathname.split("/");

  if (parts[1] === "media") {
    const apiBase =
      process.env.NEXT_PUBLIC_SERVER_SIDE_API_URL ?? "http://api:8000";
    const forwardHeaders: Record<string, string> = {};
    for (const [key, value] of request.headers.entries()) {
      if (key.toLowerCase() !== "host") forwardHeaders[key] = value;
    }
    const res = await fetch(`${apiBase}${pathname}`, { headers: forwardHeaders });
    const body = await res.arrayBuffer();
    return new NextResponse(body, {
      status: res.status,
      headers: {
        "content-type": res.headers.get("content-type") ?? "application/octet-stream",
        "cache-control": res.headers.get("cache-control") ?? "public, max-age=31536000, immutable",
      },
    });
  }

  if (parts.length >= 3 && parts[1] === "users") {
    const segment = decodeURIComponent(parts[2]);
    const accept = request.headers.get("accept") ?? "";

    if (UUID_RE.test(segment)) {
      const apiBase =
        process.env.NEXT_PUBLIC_SERVER_SIDE_API_URL ?? "http://api:8000";

      if (isApRequest(accept)) {
        // AP GET request → proxy to backend (actor JSON, outbox, followers, following)
        // Inbox POSTs are routed directly via Traefik to preserve the host header for signature verification
        const forwardHeaders: Record<string, string> = {};
        for (const [key, value] of request.headers.entries()) {
          if (key.toLowerCase() !== "host") {
            forwardHeaders[key] = value;
          }
        }

        const res = await fetch(`${apiBase}${pathname}`, {
          headers: forwardHeaders,
        });

        // Buffer the body — streaming ReadableStream via NextResponse is unreliable in Edge runtime
        const body = await res.text();
        return new NextResponse(body, {
          status: res.status,
          headers: {
            "content-type":
              res.headers.get("content-type") ?? "application/activity+json",
          },
        });
      }

      // Browser request → redirect to the human-readable username URL
      const res = await fetch(`${apiBase}/users/${segment}`);
      if (res.ok) {
        const user = await res.json();
        const url = request.nextUrl.clone();
        url.pathname = `/users/${user.username}`;
        return NextResponse.redirect(url, 301);
      }
    }

    // Remote handle redirect: /users/@user@instance
    if (segment.startsWith("@") && segment.indexOf("@", 1) !== -1) {
      const url = request.nextUrl.clone();
      url.pathname = "/remote-actor";
      url.searchParams.set("handle", segment);
      return NextResponse.rewrite(url);
    }
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/users/:path*", "/media/:path*"],
};
