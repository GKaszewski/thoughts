import Link from "next/link";
import { User } from "@/lib/api";
import { UserAvatar } from "@/components/user-avatar";
import { formatDistanceToNow, format } from "date-fns";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { profileHref } from "@/lib/utils";

function isSafeImageUrl(url: string): boolean {
  try {
    const u = new URL(url);
    return u.protocol === "https:" || u.protocol === "http:";
  } catch {
    return false;
  }
}

interface MovieMeta {
  movieTitle: string;
  releaseYear?: number;
  posterUrl?: string | null;
  rating?: number;
  comment?: string | null;
  watchedAt?: string | null;
  watchlistEntry?: boolean;
}

interface MovieCardProps {
  meta: MovieMeta;
  author: User;
  createdAt: Date;
}

function StarRating({ rating, max = 5 }: { rating: number; max?: number }) {
  return (
    <div className="flex gap-0.5">
      {Array.from({ length: max }).map((_, i) => (
        <span key={i} className={i < rating ? "text-yellow-400" : "text-muted-foreground/30"}>
          ★
        </span>
      ))}
    </div>
  );
}

export function MovieCard({ meta, author, createdAt }: MovieCardProps) {
  const isWatchlist = meta.watchlistEntry === true;
  const year = meta.releaseYear ? ` (${meta.releaseYear})` : "";
  const timeAgo = formatDistanceToNow(createdAt, { addSuffix: true });
  const watchedDate = meta.watchedAt
    ? new Date(meta.watchedAt).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      })
    : null;

  return (
    <Card className="transition-transform duration-200 hover:-translate-y-0.5 hover:shadow-fa-lg">
      <CardHeader className="flex flex-row items-center space-y-0 pb-3">
        <Link
          href={profileHref(author.username, author.local)}
          className="flex items-center gap-3 hover:opacity-80 min-w-0 w-full"
        >
          <UserAvatar src={author.avatarUrl} alt={author.displayName ?? author.username} />
          <div className="flex flex-col min-w-0">
            <span className="font-bold truncate">{author.displayName ?? author.username}</span>
            {!author.local && (
              <div className="flex items-center gap-1.5 min-w-0">
                <span className="text-xs text-muted-foreground/70 truncate flex-1">
                  {author.username.startsWith("@") ? author.username : `@${author.username}`}
                </span>
                <span className="text-[10px] font-medium px-2 py-0.5 rounded-full bg-blue-500/10 border border-blue-500/20 text-blue-500 shrink-0 max-w-[8rem] truncate">
                  {author.username.split("@").filter(Boolean).at(-1)}
                </span>
              </div>
            )}
            <time
              dateTime={createdAt.toISOString()}
              title={format(createdAt, "PPP p")}
              className="text-sm text-muted-foreground"
            >
              {timeAgo}
            </time>
          </div>
        </Link>
      </CardHeader>

      <CardContent className="pt-0">
        <div className="flex gap-3">
          <div className="shrink-0 w-16 h-24 rounded overflow-hidden bg-muted">
            {meta.posterUrl && isSafeImageUrl(meta.posterUrl) ? (
              <img
                src={meta.posterUrl}
                alt={meta.movieTitle}
                className="w-full h-full object-cover"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center text-muted-foreground text-xs text-center px-1">
                No poster
              </div>
            )}
          </div>

          <div className="flex-1 min-w-0">
            <p className="font-semibold text-sm leading-tight">
              {meta.movieTitle}
              {year && <span className="font-normal text-muted-foreground">{year}</span>}
            </p>

            {isWatchlist ? (
              <p className="text-xs text-muted-foreground mt-1">📋 Want to watch</p>
            ) : (
              <>
                {meta.rating !== undefined && (
                  <div className="mt-1">
                    <StarRating rating={meta.rating} />
                  </div>
                )}
                {watchedDate && (
                  <p className="text-xs text-muted-foreground mt-1">Watched {watchedDate}</p>
                )}
              </>
            )}

            {meta.comment && (
              <p className="text-sm mt-2 text-foreground/80 line-clamp-3">{meta.comment}</p>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
