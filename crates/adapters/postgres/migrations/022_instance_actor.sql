INSERT INTO users (id, username, email, password_hash, display_name, bio)
VALUES (
    '00000000-0000-4000-8000-000000000000',
    'instance',
    'noreply@instance.invalid',
    '!service-actor-no-login',
    NULL,
    NULL
)
ON CONFLICT (id) DO NOTHING;
