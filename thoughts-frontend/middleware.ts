import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export function middleware(request: NextRequest) {
  const parts = request.nextUrl.pathname.split("/");

  // /users/@user@instance or /users/%40user%40instance
  if (parts.length === 3 && parts[1] === "users") {
    const decoded = decodeURIComponent(parts[2]);
    if (decoded.startsWith("@") && decoded.indexOf("@", 1) !== -1) {
      const url = request.nextUrl.clone();
      url.pathname = "/remote-actor";
      url.searchParams.set("handle", decoded);
      return NextResponse.rewrite(url);
    }
  }

  return NextResponse.next();
}

export const config = {
  matcher: "/users/:path*",
};
