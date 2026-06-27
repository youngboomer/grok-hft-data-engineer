# Backend & APIs (FastAPI, Django, REST, WebSockets, Caching)

## FastAPI

**Why It's Powerful**:
- Modern Python web framework built on Starlette + Pydantic.
- Automatic OpenAPI docs, validation, serialization.
- Async-first (can mix sync and async).
- Excellent performance (comparable to Go/Node in many cases when using async properly).
- Dependency injection system is very expressive.

**Deep Topics**:
- Pydantic v2 models for request/response (use `model_config`, `Field`, validators).
- BackgroundTasks vs proper task queues for long work.
- WebSocket support with connection management.
- Dependency overrides for testing.
- Lifespan events (startup/shutdown).
- Rate limiting, CORS, security best practices.

**Performance Tips**:
- Use async endpoints + async DB clients (asyncpg, aiomysql, etc.).
- Avoid blocking calls in async context.
- Use `response_model_exclude_unset` etc. to reduce payload size.
- Proper connection pooling.

**Interview Angle**: "I built a high-concurrency API service in FastAPI that handled both REST and WebSocket connections, with proper dependency injection for auth and caching layers."

## Django

**Strengths**: Batteries-included (ORM, admin, auth, forms, migrations). Mature ecosystem. Great for admin-heavy or content-heavy backends.

**Modern Django**:
- Async views and async ORM support (still maturing).
- Django REST Framework (DRF) for APIs — serializers, viewsets, permissions, pagination.
- Channels for WebSockets (runs on top of ASGI).

**When to Choose Django vs FastAPI**:
- Complex admin interfaces or heavy relational modeling → Django.
- High-performance microservices or APIs where you want minimal magic → FastAPI.
- Many teams use both: Django for the "heavy" parts, FastAPI for performance-sensitive services.

## REST API Best Practices (Auth, CRUD, WebSockets, Caching)

**Auth**:
- JWT vs opaque tokens vs session cookies.
- OAuth2 / OIDC flows.
- API key patterns for service-to-service.
- Proper token rotation and revocation strategies.

**CRUD + Beyond**:
- Use proper HTTP methods and status codes.
- Idempotency keys for POST/PUT.
- Pagination (cursor-based preferred over offset for large datasets).
- Filtering, sorting, field selection (avoid over-fetching).

**Caching**:
- HTTP caching headers (ETag, Cache-Control, Last-Modified).
- Application-level caching (Redis) with proper invalidation strategies.
- CDN for static or semi-static responses.
- Cache-aside vs write-through vs write-behind.

**WebSockets in APIs**:
- Authentication on the upgrade request.
- Heartbeats and reconnect logic on client.
- Backpressure and message queuing per connection.
- Scaling: Redis pub/sub or dedicated connection manager service.

**Security Must-Haves**:
- Input validation (never trust client).
- Rate limiting per user/IP/key.
- Proper CORS.
- SQL injection / NoSQL injection prevention (use parameterized queries/ORM).
- Secrets management (never in code).

## Interview-Ready Stories

Prepare examples like:
- "Redesigned a slow synchronous Django API into a FastAPI async service with Redis caching and proper pagination, improving p95 latency from 800ms to 45ms."
- "Implemented a real-time collaborative feature using FastAPI WebSockets + Redis pub/sub with proper connection lifecycle and auth."
- "Built a robust auth layer supporting both user JWTs and service-to-service mTLS/API keys with centralized policy enforcement."
