import { User } from "@/lib/api";

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
  const watchedDate = meta.watchedAt
    ? new Date(meta.watchedAt).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      })
    : null;

  return (
    <div className="rounded-lg border bg-card overflow-hidden">
      <div className="flex gap-3 p-3">
        {/* Poster */}
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

        {/* Info */}
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

      {/* Footer */}
      <div className="px-3 py-2 border-t bg-muted/30 flex items-center gap-2 text-xs text-muted-foreground">
        <span>@{author.username}</span>
        <span>·</span>
        <span>{createdAt.toLocaleDateString()}</span>
      </div>
    </div>
  );
}
