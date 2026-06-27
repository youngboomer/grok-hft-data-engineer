# Messaging & Event-Driven Systems

## Core Philosophy
Event-driven architecture decouples producers and consumers in time and space. This enables scalability, resilience, and independent evolution of components. The choice of messaging technology has huge implications for latency, durability, ordering guarantees, and operational complexity.

## Technology Deep Dives

### ZeroMQ (ZMQ)

**What it is**: A high-performance messaging library (not a full broker). "Sockets on steroids."

**Patterns**:
- REQ/REP
- PUB/SUB
- PUSH/PULL (pipeline)
- DEALER/ROUTER
- XPUB/XSUB for proxying

**Strengths**:
- Extremely low latency (can be sub-millisecond)
- No broker = simpler deployment, no single point of failure for the broker itself
- Supports many transports (inproc, ipc, tcp)
- Very lightweight

**Weaknesses**:
- You have to implement reliability, queuing, and durability yourself (or use higher-level patterns like CZMQ, Zyre, etc.).
- No built-in persistence.
- Can be complex to get exactly-once or at-least-once right at scale.

**When to Use**:
- Low-latency, high-throughput internal communication between services on the same machine or LAN.
- Replacing raw sockets with structured messaging.
- Building custom brokers or proxies.

**Interview Example**:
"We used ZMQ PUSH/PULL with a custom load-balancing proxy for a low-latency feed handler. This gave us sub-100µs publish latency while allowing dynamic addition of workers. We added heartbeats and sequence numbers for reliability on top."

### MQTT & NanoMQ

**MQTT**: Lightweight publish/subscribe protocol, originally for IoT. QoS 0/1/2, retained messages, last will & testament, topic wildcards.

**NanoMQ**: High-performance MQTT broker implemented in C (with Rust components in some versions). Designed for high concurrency and low resource usage.

**Use Cases in Data Systems**:
- Telemetry ingestion
- Command & control
- Edge to cloud messaging
- Replacing heavier brokers when you only need pub/sub with QoS

**Key Concepts**:
- QoS levels and their cost (QoS 2 is expensive).
- Session persistence and clean session flag.
- Shared subscriptions (load balancing across subscribers).

**Gotchas**:
- Ordering is per-topic per-client in many brokers.
- Retained messages can surprise you with old data.
- Wildcard subscriptions can create hot topics.

### AMQP (Advanced Message Queuing Protocol)

Most commonly implemented by RabbitMQ.

**Strengths**:
- Rich routing (direct, topic, fanout, headers exchanges)
- Strong durability and acknowledgments
- Good support for dead-letter queues, TTL, priorities
- Transactions and publisher confirms

**Weaknesses**:
- Heavier than MQTT or pure ZMQ
- Can become a bottleneck if not clustered properly
- More operational overhead than cloud-native alternatives (SQS, EventBridge, Kafka)

**When RabbitMQ shines**: Complex routing logic, need for strong delivery guarantees with moderate throughput, traditional enterprise integration.

### WebSockets & Socket.IO

**WebSockets**: Full-duplex TCP connection for the browser/server or service-to-service.

**Important for Systems**:
- Connection lifecycle management (ping/pong, reconnect with exponential backoff)
- Message framing and backpressure
- Authentication on upgrade + per-message auth
- Scaling: sticky sessions or connection routing (e.g., via Redis adapter for Socket.IO)

**Socket.IO**: Adds reconnection, rooms, acknowledgments, binary support on top of WebSocket (with polling fallback).

**Low-Latency Considerations**:
- Use binary frames when possible.
- Be careful with per-connection state.
- For high scale, consider a dedicated gateway layer (e.g., using NATS, or custom Rust gateway) instead of putting everything through one Node/Python process.

### Redis as Messaging

**Common Patterns**:
- Pub/Sub (fire and forget, no durability)
- Streams (Redis 5+) — like a lightweight Kafka with consumer groups, ranges, trimming
- Lists + BRPOP for simple queues
- Sorted Sets for delayed queues

**Redis Streams** is often the sweet spot for many internal event systems:
- Append-only
- Consumer groups with automatic claiming of stalled messages
- XTRIM for retention control
- Very low latency

**Tradeoffs vs Kafka**:
- Redis is simpler to operate for moderate scale.
- Kafka wins on durability, replayability, exactly-once (with idempotent producers + transactions), and massive scale.

### Event-Driven Architecture Patterns & Tradeoffs

**Key Dimensions**:
- At-least-once vs exactly-once
- Ordering guarantees (global, per-key, none)
- Durability / replay capability
- Latency vs throughput
- Operational complexity

**Important Patterns**:
- **Event Sourcing**: Store the facts (events), derive state.
- **CQRS**: Separate read and write models.
- **Saga / Choreography vs Orchestration** for distributed transactions.
- **Outbox Pattern**: Ensure events are published atomically with database changes (very important for data consistency).
- **Idempotency Keys**: Critical for safe retries.

**Hot Path vs Cold Path in Event Systems**:
- Hot: Low-latency decision making from recent events (use in-memory state + fast streams like ZMQ/Redis Streams).
- Cold: Long-term storage, reprocessing, analytics (Kafka + data lake, or direct to S3/Parquet).

**Common Gotchas**:
- Dual writes (writing to DB and emitting event without outbox → inconsistency).
- Assuming global ordering when you only have per-partition ordering.
- Leaky abstractions around "fire and forget".
- Not handling poison messages (dead letter queues are mandatory in production).

## Comparison Table

| Technology     | Latency     | Durability | Ordering      | Scale          | Operational Complexity | Best For                     |
|----------------|-------------|------------|---------------|----------------|------------------------|------------------------------|
| ZMQ            | Very Low    | None       | Per connection| High (custom)  | Low                    | Internal low-latency comms   |
| MQTT / NanoMQ  | Low         | Limited    | Per topic     | High           | Medium                 | IoT / telemetry              |
| RabbitMQ (AMQP)| Low-Medium  | High       | Per queue     | Medium         | Medium                 | Complex routing + reliability|
| Redis Streams  | Very Low    | Medium     | Per stream    | High           | Low                    | Lightweight event sourcing   |
| Kafka          | Low         | Very High  | Per partition | Very High      | High                   | Large scale event backbone   |
| WebSockets     | Low         | None       | Per conn      | Medium (w/ LB) | Medium                 | Real-time client comms       |

## Practical HFT Use Cases & Scenarios

### Scenario: Low-Latency Hot-Path Event Bus for Market Making Risk
**Business Context**: In a crypto MM, the Rust feed handler normalizes ticks into an internal order book and risk state. Multiple consumers need this state: Python quoting strategy (needs <100µs updates), real-time risk engine (inventory skew + exposure), and logging/audit. During liquidation cascades (high message rate + many partial fills), we saw duplicate risk updates and occasional missed skew signals, leading to over-hedging.

**Challenge**: Producer must never block; need fan-out with backpressure; exactly-once or at-least-once with dedup for fills; mix of in-process (fast) and cross-process (durable) consumers; hot path must survive reconnects without data loss.

**Solution (How)**:
- **Hot path**: Rust parser publishes normalized events to a lock-free SPSC/MPSC ring buffer (or custom Disruptor-style with sequence cursors). Consumers read non-blocking.
- **Cross-service**: Redis Streams (or ZMQ PUSH/PULL with custom proxy) for durability. Used outbox pattern in the book updater: write to local log + stream atomically.
- **Idempotency**: Every event carries exchange seq + client order id. Consumers use a small in-memory set or Redis SETNX for dedup on reconnect.
- **Backpressure**: Bounded queues; strategy drops non-critical updates rather than letting queues grow (producer stays fast).
- **Cold path**: Full stream also lands in Kafka/Redpanda for the analytics lake and replay.

**ASCII DFD**:
```
Rust Feed + Book Updater (hot)
        │ (lock-free ring buffer)
        ├──▶ Strategy (Python, low-lat)
        ├──▶ Risk Engine (inventory)
        └──▶ Redis Streams (durable)
                     │
                     ▼ (outbox + dedup)
             Analytics / Replay (cold)
```

**Results**:
- Hot path publish latency p99 stayed <40µs even at 20k msgs/sec.
- Zero duplicate risk updates after reconnects (validated with sequence replay).
- Over-hedging incidents dropped to zero in the next two liquidation events.

**Gotchas**:
- Redis Streams consumer groups have rebalancing cost — we used manual claiming for critical consumers.
- Don't put complex logic in the hot ring buffer consumer; keep it to "update local view + decide to forward".
- When NOT: For very high fan-out (hundreds of consumers) across machines, consider Aeron or NATS JetStream instead of Redis.

**Interview Talking Points**:
- "The producer only ever does a non-blocking publish to the ring buffer. All durability and fan-out logic lives after the hot path."
- "We used sequence numbers + idempotency keys so reconnect logic could never create duplicate fills or risk signals."

## Interview Preparation

**Must-be-able-to-draw**:
- A system using multiple messaging technologies for different purposes (e.g., ZMQ for hot path, Kafka for durable replay, Redis for cache invalidation).
- The outbox pattern.
- Consumer group rebalancing and exactly-once challenges.

**Strong Claims**:
- "Implemented a hybrid event system using ZMQ for sub-millisecond hot path decisions and Redis Streams + outbox for reliable state changes that fed both real-time services and our Spark analytics layer."
- "Diagnosed and fixed ordering and duplication issues in a high-scale WebSocket + Redis pub/sub architecture by introducing sequence numbers and idempotency keys."

Prepare stories about debugging message loss, duplicate processing, or latency spikes under load.
