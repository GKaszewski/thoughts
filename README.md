# Thoughts

Web service that allows users to post short messages (up to 128 characters)! Users can also invite each other to be
friends, and see each other's posts.

# Features

- Users can post short messages (up to 128 characters)
- Users can invite each other to be friends
- Customizable user profile
- User can decide who can see their profile
- Thoughts can expire after a certain amount of time (if user provides an expiration date)
- Social login (GitHub, Discord)

# Tech Stack

- Django
- HTMX (for real-time updates)
- PostgreSQL (for database)
- Docker (not yet implemented)
- Cron (for expiring thoughts) (not yet implemented)

# Todo
- Integrate Docker
- Implement Cron for expiring thoughts
- Add styling and better UX
- Write tests
- Write documentation

# Useful tips
- Rebuild tailwindcss: `npx tailwindcss -i ./static/src/input.css -o ./static/src/output.css --watch`