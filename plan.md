# KoKo — Twitter-like Backend Plan
> Stack: **Rust · Axum · SQLx · PostgreSQL · Redis · JWT**

---

## 1. Tech Stack

| Layer        | Choice                              |
|--------------|-------------------------------------|
| HTTP         | `axum` (tokio runtime)              |
| DB ORM       | `sqlx` (async, compile-time checks) |
| Database     | PostgreSQL 16                       |
| Cache/Queue  | Redis 7                             |
| Auth         | JWT (access 15 min + refresh 7 d)   |
| Password     | `argon2`                            |
| Validation   | `validator`                         |
| Serialise    | `serde` + `serde_json`              |
| Env config   | `dotenvy`                           |
| Tracing      | `tracing` + `tracing-subscriber`    |
| Task queue   | `tokio` background tasks (simple)   |

---

## 2. Project Structure

```
src/
├── main.rs               # entry: build router, run server
├── config.rs             # AppConfig from env
├── db.rs                 # PgPool + RedisPool setup
├── errors.rs             # AppError enum → JSON responses
├── middleware/
│   ├── auth.rs           # JWT extractor
│   └── rate_limit.rs     # Redis-backed rate limiter
├── models/               # DB row structs
│   ├── user.rs
│   ├── tweet.rs
│   ├── follow.rs
│   ├── like.rs
│   ├── retweet.rs
│   ├── comment.rs
│   ├── notification.rs
│   └── media.rs
├── dto/                  # Request / Response shapes
│   └── *.rs
├── routes/
│   ├── mod.rs            # merge all routers
│   ├── auth.rs
│   ├── users.rs
│   ├── tweets.rs
│   ├── feed.rs
│   ├── notifications.rs
│   └── search.rs
├── services/             # business logic
│   ├── auth_service.rs
│   ├── tweet_service.rs
│   ├── feed_service.rs
│   └── notification_service.rs
└── utils/
    ├── jwt.rs
    ├── hash.rs
    └── pagination.rs
```

---

## 3. Database Schema

### 3.1 `users`
```sql
CREATE TABLE users (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  username      VARCHAR(50)  UNIQUE NOT NULL,
  handle        VARCHAR(50)  UNIQUE NOT NULL,   -- @handle
  email         VARCHAR(255) UNIQUE NOT NULL,
  password_hash TEXT         NOT NULL,
  bio           TEXT,
  avatar_url    TEXT,
  banner_url    TEXT,
  is_verified   BOOLEAN      DEFAULT FALSE,
  is_private    BOOLEAN      DEFAULT FALSE,
  follower_count INT         DEFAULT 0,
  following_count INT        DEFAULT 0,
  tweet_count    INT         DEFAULT 0,
  created_at    TIMESTAMPTZ  DEFAULT now(),
  updated_at    TIMESTAMPTZ  DEFAULT now()
);
```

### 3.2 `refresh_tokens`
```sql
CREATE TABLE refresh_tokens (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ DEFAULT now()
);
```

### 3.3 `tweets`
```sql
CREATE TABLE tweets (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  content       VARCHAR(280),
  media_urls    TEXT[],                          -- array of S3/CDN URLs
  reply_to_id   UUID REFERENCES tweets(id),      -- NULL = top-level
  retweet_of_id UUID REFERENCES tweets(id),      -- NULL = original
  like_count    INT  DEFAULT 0,
  retweet_count INT  DEFAULT 0,
  reply_count   INT  DEFAULT 0,
  view_count    BIGINT DEFAULT 0,
  is_deleted    BOOLEAN DEFAULT FALSE,
  created_at    TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_tweets_user_id   ON tweets(user_id);
CREATE INDEX idx_tweets_created   ON tweets(created_at DESC);
CREATE INDEX idx_tweets_reply     ON tweets(reply_to_id);
```

### 3.4 `follows`
```sql
CREATE TABLE follows (
  follower_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  following_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at   TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (follower_id, following_id)
);

CREATE INDEX idx_follows_following ON follows(following_id);
```

### 3.5 `likes`
```sql
CREATE TABLE likes (
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  tweet_id   UUID NOT NULL REFERENCES tweets(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (user_id, tweet_id)
);
```

### 3.6 `bookmarks`
```sql
CREATE TABLE bookmarks (
  user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  tweet_id   UUID NOT NULL REFERENCES tweets(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (user_id, tweet_id)
);
```

### 3.7 `notifications`
```sql
CREATE TABLE notifications (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- recipient
  actor_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- who triggered
  kind        VARCHAR(20) NOT NULL,  -- like | retweet | follow | reply | mention
  tweet_id    UUID REFERENCES tweets(id) ON DELETE CASCADE,
  is_read     BOOLEAN DEFAULT FALSE,
  created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_notif_user ON notifications(user_id, created_at DESC);
```

### 3.8 `hashtags` + `tweet_hashtags`
```sql
CREATE TABLE hashtags (
  id   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tag  VARCHAR(100) UNIQUE NOT NULL
);

CREATE TABLE tweet_hashtags (
  tweet_id   UUID NOT NULL REFERENCES tweets(id) ON DELETE CASCADE,
  hashtag_id UUID NOT NULL REFERENCES hashtags(id) ON DELETE CASCADE,
  PRIMARY KEY (tweet_id, hashtag_id)
);
```

---

## 4. Redis Usage

| Key Pattern                        | Purpose                        | TTL      |
|------------------------------------|--------------------------------|----------|
| `session:{user_id}`                | Blacklist / session meta       | 7 days   |
| `feed:{user_id}`                   | Cached home timeline (tweet IDs) | 5 min  |
| `rate:{ip}:{route}`                | Rate-limit counter             | 1 min    |
| `tweet_views:{tweet_id}`           | Batch view count               | 10 min   |
| `trending_tags`                    | Sorted set of hashtag scores   | 15 min   |

---

## 5. API Routes

### Auth  `POST /api/auth/*`
| Method | Path              | Description          |
|--------|-------------------|----------------------|
| POST   | `/register`       | Create account       |
| POST   | `/login`          | Issue JWT pair       |
| POST   | `/refresh`        | Rotate refresh token |
| POST   | `/logout`         | Revoke token         |

### Users  `GET|PATCH /api/users/*`
| Method | Path                          | Auth | Description           |
|--------|-------------------------------|------|-----------------------|
| GET    | `/:handle`                    | —    | Public profile        |
| PATCH  | `/me`                         | ✓    | Update profile        |
| GET    | `/:handle/followers`          | —    | Follower list         |
| GET    | `/:handle/following`          | —    | Following list        |
| POST   | `/:handle/follow`             | ✓    | Follow user           |
| DELETE | `/:handle/follow`             | ✓    | Unfollow user         |

### Tweets  `/api/tweets/*`
| Method | Path                      | Auth | Description               |
|--------|---------------------------|------|---------------------------|
| POST   | `/`                       | ✓    | Create tweet              |
| GET    | `/:id`                    | —    | Get tweet                 |
| DELETE | `/:id`                    | ✓    | Soft-delete own tweet     |
| POST   | `/:id/like`               | ✓    | Like / unlike toggle      |
| POST   | `/:id/retweet`            | ✓    | Retweet                   |
| POST   | `/:id/reply`              | ✓    | Reply                     |
| GET    | `/:id/replies`            | —    | Thread replies            |
| POST   | `/:id/bookmark`           | ✓    | Bookmark toggle           |

### Feed  `/api/feed`
| Method | Path              | Auth | Description              |
|--------|-------------------|------|--------------------------|
| GET    | `/home`           | ✓    | Home timeline            |
| GET    | `/explore`        | —    | Trending / global feed   |
| GET    | `/bookmarks`      | ✓    | Saved tweets             |

### Notifications  `/api/notifications`
| Method | Path       | Auth | Description          |
|--------|------------|------|----------------------|
| GET    | `/`        | ✓    | List (paginated)     |
| PATCH  | `/read`    | ✓    | Mark all as read     |

### Search  `/api/search`
| Method | Path      | Query Params             |
|--------|-----------|--------------------------|
| GET    | `/`       | `q`, `type=users\|tweets`, `cursor`, `limit` |

---

## 6. Request / Response Flow

```
Client
  │
  ▼
[ Rate Limit Middleware ]  →  Redis key check
  │
  ▼
[ Auth Middleware ]         →  Validate JWT → inject Claims into request extensions
  │
  ▼
[ Route Handler ]
  │
  ├─ validate DTO (validator crate)
  ├─ call Service layer
  │     ├─ hit Redis cache (feed, trending)
  │     └─ query PostgreSQL via sqlx
  ├─ queue Notification (tokio::spawn background task)
  │
  ▼
[ AppError → JSON response ]
  │
  ▼
Client
```

---

## 7. Auth Flow (JWT)

```
Register → hash password (argon2) → insert user → return tokens
Login    → verify hash → generate access (15 min) + refresh (7 d) → store refresh hash in DB
Request  → Bearer header → decode/verify → inject user_id
Refresh  → validate refresh token → rotate (delete old, insert new) → return new pair
Logout   → delete refresh token row → optionally blacklist in Redis
```

---

## 8. Feed Algorithm (simple fan-out on read)

```
GET /feed/home:
  1. Check Redis key feed:{user_id} → return if hit
  2. Fetch following_ids for user
  3. SELECT tweets WHERE user_id IN (following_ids)
     ORDER BY created_at DESC LIMIT 20
  4. Merge own tweets
  5. Cache result in Redis (5 min TTL)
  6. Return paginated list
```

---

## 9. Cargo Dependencies

```toml
[dependencies]
axum          = { version = "0.7", features = ["macros"] }
tokio         = { version = "1",   features = ["full"] }
sqlx          = { version = "0.8", features = ["postgres","uuid","chrono","runtime-tokio"] }
redis         = { version = "0.25", features = ["tokio-comp"] }
serde         = { version = "1",   features = ["derive"] }
serde_json    = "1"
uuid          = { version = "1",   features = ["v4","serde"] }
chrono        = { version = "0.4", features = ["serde"] }
jsonwebtoken  = "9"
argon2        = "0.5"
validator     = { version = "0.18", features = ["derive"] }
dotenvy       = "0.15"
tracing       = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tower-http    = { version = "0.5", features = ["cors","trace"] }
```

---

## 10. Environment Variables

```env
DATABASE_URL=postgres://user:pass@localhost:5432/koko
REDIS_URL=redis://localhost:6379
JWT_SECRET=super_secret_key
JWT_EXPIRE_MINS=15
REFRESH_EXPIRE_DAYS=7
SERVER_PORT=8080
```

---

## 11. Build Phases

| Phase | Tasks |
|-------|-------|
| **1 — Foundation** | Cargo setup, config, DB pool, Redis pool, error handling, tracing |
| **2 — Auth** | Register, login, refresh, logout; argon2, JWT utils |
| **3 — Users** | Profile CRUD, follow/unfollow, counter updates |
| **4 — Tweets** | Create, delete, like, retweet, reply; hashtag extraction |
| **5 — Feed** | Home timeline, explore, bookmarks; Redis caching |
| **6 — Notifications** | Background task; mark-read endpoint |
| **7 — Search** | Full-text search via `pg_trgm` or `tsvector` |
| **8 — Polish** | Rate limiting, pagination cursor, media upload stubs, CI |

---

## 12. Connection Structure Diagram

```
┌──────────────────────────────────────────┐
│                 Axum Server               │
│  ┌──────────┐   ┌──────────────────────┐ │
│  │ Middleware│   │      Routers         │ │
│  │ auth      │   │ auth / users / tweets│ │
│  │ rate_limit│   │ feed / notif / search│ │
│  └──────────┘   └──────────┬───────────┘ │
│                             │             │
│              ┌──────────────┴──────────┐  │
│              │      Service Layer       │  │
│              └────────────┬────────────┘  │
│                  ┌────────┴────────┐      │
│           ┌──────▼──────┐  ┌──────▼────┐ │
│           │  PostgreSQL  │  │   Redis   │ │
│           │  (sqlx pool) │  │  (cache)  │ │
│           └─────────────┘  └───────────┘ │
└──────────────────────────────────────────┘
```
