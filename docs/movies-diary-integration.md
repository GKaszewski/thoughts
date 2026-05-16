# Movies-Diary First-Class Integration

Since thoughts and movies-diary are both owned projects, movies-diary can be treated as a first-class citizen with deep, structured integration rather than a generic ActivityPub instance.

## Core idea

Add a custom ActivityPub `@context` extension to movies-diary's AP notes that carries structured movie review data. Thoughts understands this extension and renders movie review posts as rich cards instead of plain text. Movies-diary actor profiles in thoughts get a dedicated "Movie Diary" layout.

---

## Feature 1 — Custom AP Extension for Movie Reviews

### movies-diary side

Extend the AP Note with a `movies-diary` namespace in `@context`:

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "md": "https://movies.gabrielkaszewski.dev/ns#",
      "movieReview": "md:movieReview",
      "movieTitle": "md:movieTitle",
      "movieYear": "md:movieYear",
      "rating": "md:rating",
      "maxRating": "md:maxRating",
      "watchedAt": "md:watchedAt",
      "posterUrl": "md:posterUrl",
      "tmdbId": "md:tmdbId"
    }
  ],
  "type": "Note",
  "movieReview": true,
  "movieTitle": "Eternals",
  "movieYear": 2021,
  "rating": 3,
  "maxRating": 5,
  "watchedAt": "2025-09-30",
  "posterUrl": "https://image.tmdb.org/t/p/w300/...",
  "tmdbId": 524434,
  "content": "<p>⭐⭐⭐ Eternals (2021) Watched: Sep 30, 2025</p>"
}
```

The `content` field keeps the plain-text fallback so the post still renders correctly in any standard AP client.

### thoughts side

When fetching remote notes in `fetch_outbox_page`, detect the extension fields and store the structured data alongside the note. This requires:

- A new `remote_note_meta` table (or a JSON column on `thoughts`) for: `movie_title`, `movie_year`, `rating`, `max_rating`, `watched_at`, `poster_url`, `tmdb_id`
- A new domain model field or separate `MovieReviewMeta` struct
- The thought card in the frontend checks for this metadata and renders a `MovieReviewCard` component instead of plain text

### `MovieReviewCard` component

Shows:
- Movie poster (from `posterUrl`)
- Title + year
- Star rating (visual, not emoji)
- Watched date
- Optional review text (the `content` stripped of the auto-generated prefix)
- Link to the movie on the user's movies-diary instance

---

## Feature 2 — Dedicated Movies-Diary Actor Profile

When viewing an actor profile from a movies-diary instance (detected by actor URL domain or a custom AP actor field), the profile page shows a "Movie Diary" layout instead of the generic remote actor profile.

### Detection

Add a custom field to movies-diary's AP `Person` object:

```json
{
  "type": "Person",
  "md:softwareName": "movies-diary",
  "md:instanceUrl": "https://movies.gabrielkaszewski.dev"
}
```

Thoughts checks for `md:softwareName = "movies-diary"` and switches to the dedicated layout.

### Movie Diary profile layout

- **Header**: same avatar/banner/bio/follow button as the generic profile
- **Stats bar**: Total reviews · Watchlist size · Avg rating
- **Recent reviews grid**: Movie poster cards (not a feed of text posts) — each shows poster, title, year, rating, watched date
- **Tabs**: Recent Reviews | Watchlist | Following (other movie diary users)
- **Watchlist tab**: Shows movies marked as "want to watch" (requires a custom AP Collection type: `md:Watchlist`)

### API

The movies-diary instance exposes custom AP endpoints that thoughts can call (since it owns both):

- `GET /ap/users/{username}/watchlist` — returns AP OrderedCollection of watchlist items (with `md:` fields)
- `GET /ap/users/{username}/reviews?page=1` — returns AP OrderedCollectionPage of reviews (rich notes)

Thoughts fetches these when rendering the movie diary profile, similar to how it fetches the outbox.

---

## Implementation order (when ready)

1. Define and document the `md:` namespace schema in movies-diary
2. Emit `md:` fields on movies-diary AP notes and Person objects
3. Extend thoughts `fetch_outbox_page` to parse and store `md:` fields
4. Build `MovieReviewCard` frontend component
5. Add detection logic for movies-diary actors
6. Build the dedicated Movie Diary profile layout + watchlist/reviews tabs
7. Implement the custom AP endpoints on movies-diary side

---

## Notes

- The `content` fallback in AP notes ensures movies-diary posts remain readable in Mastodon, Pleroma, and any other standard client — the extension is additive
- The `md:` namespace URL should resolve to a JSON-LD context document for proper AP compliance
- Authentication between thoughts and movies-diary can use the existing AP HTTP signatures, so no separate auth system is needed
- TMDB poster URLs may require a TMDB API key on movies-diary's side; thoughts just stores and displays the URL
