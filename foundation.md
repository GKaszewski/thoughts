Project Thoughts: Foundational Plan & System Architecture

1. Vision & Core Philosophy
   Project Vision: To create a decentralized social media platform that prioritizes genuine user connection, creative self-expression, and a user-controlled experience. Thoughts is a deliberate departure from algorithm-driven feeds and corporate-controlled online spaces.

Core Philosophy: We are building a digital "third place" reminiscent of the early internet, where the focus is on community and individuality. The platform's aesthetic and function are guided by the "Frutiger Aero" design language—optimistic, humanistic, and clear.

Tagline: Your Space, Your Friends, Your Feed.

2. Guiding Principles & Rules to Follow
   These are the non-negotiable rules that will guide all development and feature decisions.

Chronological Above All: The main feed will always be in reverse chronological order. There will be no algorithmic sorting or "while you were away" features.

User in Control: Users, not the platform, decide what they see by choosing who to follow. There will be no algorithmic content or user suggestions.

Radical Self-Expression: Profiles are a canvas. Users will be given significant freedom to customize the look and feel of their personal space.

Open and Federated: The platform must be a citizen of the Fediverse. ActivityPub integration is a primary, not secondary, feature.

Performance by Default: The user experience should be fast and lightweight, minimizing client-side JavaScript and optimizing for quick page loads.

API-First Design: The backend is the core service. The frontend is simply the first and primary client of a well-documented, public-facing API. This ensures that third-party developers have the same power as the official web client.

3. System Architecture
   This section details the technical structure of the platform. Your concern about running a Rust backend with a Next.js frontend is a common one. The standard and most effective solution is to run them as two separate services that communicate with each other, orchestrated by a reverse proxy.

3.1. Architecture Diagram (High Level)
+----------------+ +---------------------+ +------------------------+ +-------------------+
| User's | <--> | Reverse Proxy | <--> | Next.js Frontend | | |
| Browser | | (Nginx / Caddy) | | (Web Server) | | |
+----------------+ +---------------------+ +------------------------+ | PostgreSQL |
| | | Database |
| (Routes /api/\*) | | |
| | +-------------------+
v |
+------------------------+ <--------------------------------------+
| Rust Backend |
| (API & ActivityPub) |
+------------------------+

3.2. Component Breakdown
Frontend: Next.js

Role: Acts as the primary client for the Rust API. It is responsible for rendering the user interface.

Justification: Using Next.js for Server-Side Rendering (SSR) gives us the best of both worlds: fast initial page loads with minimal client-side JavaScript (fulfilling the "lightweight" requirement) and a modern, component-based development experience. It will handle user sessions and render pages by fetching data from the Rust API during the request-response cycle on the server.

Backend: Rust

Role: The core of the application. It's a stateless API server that handles all business logic, user authentication, database operations, and ActivityPub federation logic.

Recommended Framework: axum. It is modern, modular, and integrates seamlessly with the tokio ecosystem, making it a robust choice for building high-performance APIs.

API Specification: We will use the OpenAPI 3.0 standard to define and document the REST API.

Database: PostgreSQL

Role: The single source of truth for all user data, posts, follows, etc.

Justification: PostgreSQL is a powerful, reliable, and open-source relational database with excellent support in the Rust ecosystem (via libraries like sqlx and diesel). It provides the structure needed for a social platform.

Deployment: Docker & Docker Compose

Role: To containerize each component (Frontend, Backend, Database, Reverse Proxy) for consistent, reproducible, and easy deployments.

docker-compose.yml Structure: The final docker-compose.yml file will define four services:

thoughts-backend: Builds and runs the Rust application.

thoughts-frontend: Builds and runs the Next.js application.

database: Runs the official PostgreSQL image, with a persistent volume for data.

proxy: Runs an Nginx or Caddy container configured to route traffic. Requests to yourdomain.com/api/\* will be forwarded to the Rust service, and all other requests will go to the Next.js service.

4. Features & Acceptance Criteria
   This outlines the minimum viable product (MVP) features.

Feature

Description

Acceptance Criteria

User Accounts

Users can sign up, log in, log out. Usernames are unique.

I can successfully create an account. I can log in with my credentials and am issued a session. I can log out, which invalidates my session.

Customizable Profiles

Each user has a public profile page (/username). Profiles have a display name, bio, avatar, header, and a "Top Friends" list.

I can edit my profile details. I can upload an avatar and header image. I can select up to 8 other users to display in my "Top Friends" list.

Profile CSS

A field in the profile settings allows users to input custom CSS to style their profile page. This CSS is sanitized to prevent security risks.

I can add custom CSS to my profile. When another user visits my profile, they see my custom styling applied. Malicious CSS (e.g., url() with external scripts) is stripped out.

Posting Thoughts

Users can publish short text posts (max 128 characters) to their profile. Posts can include hashtags (e.g., #frutigeraero).

I can publish a post. It appears on my profile and in the feeds of my followers. Hashtags are clickable links. I receive an error if my post is over 128 characters.

Following System

Users can follow and unfollow other users on the platform.

When I visit a user's profile, I see a "Follow" button. Clicking it adds them to my follow list. Clicking "Unfollow" removes them.

The Main Feed

The homepage for a logged-in user. It displays posts from all followed users in reverse chronological order.

My feed contains posts from everyone I follow. The newest post is always at the top. The feed contains no posts from users I don't follow.

Tag Discovery

A "Popular Tags" section on the main page lists tags that are currently trending. Clicking a tag shows a public feed of all posts with that tag.

The popular tags list updates based on recent post activity. Clicking a tag takes me to a page showing all posts with that tag, sorted chronologically.

ActivityPub (MVP)

A user's profile is exposed as an ActivityPub actor (e.g., @username@thoughts.social).

A user on another Fediverse platform (e.g., Mastodon) can search for my profile and follow me. My new posts are federated and appear on their timeline.

Public API (MVP)

A simple, token-based API for publishing thoughts.

I can generate an API key from my settings. Using this key and curl or another tool, I can successfully publish a "thought" to my profile. API documentation is available.
