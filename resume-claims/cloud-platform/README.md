# Cloud & Platform: AWS, Docker, CI/CD, Linux

## AWS Deep Knowledge (Focus on Listed Services)

**EC2 + Networking**:
- Instance types and when to choose compute/memory/network optimized.
- Placement groups, ENI, EBS vs instance store.
- Security groups vs NACLs.
- Auto Scaling + lifecycle hooks.

**S3**:
- Storage classes and lifecycle policies.
- Event notifications (to SQS, SNS, Lambda, EventBridge).
- Versioning, encryption, Object Lock.
- Performance: multipart upload, S3 Select, Transfer Acceleration, request patterns to avoid throttling.

**EMR & Athena**:
- EMR for Spark/Hadoop workloads (vs self-managed on EC2/EKS).
- Athena for serverless SQL on S3 (Presto/Trino based). Partitioning, compression, and file formats (Parquet + snappy/zstd) are critical for cost and speed.

**Lambda**:
- Cold starts, provisioned concurrency, SnapStart.
- Memory = CPU allocation.
- VPC networking costs and latency.
- Layers, extensions, and graceful shutdowns.
- When not to use Lambda (long-running, very high memory, complex dependencies).

**Containers (ECS/Fargate, EKS)**:
- Fargate: serverless containers — good balance of simplicity and control.
- EKS: full Kubernetes control — when you need operators, complex scheduling, or multi-team standardization.
- Sidecars, init containers, resource requests/limits.
- Horizontal Pod Autoscaler + Cluster Autoscaler / Karpenter.

**Messaging & Events**:
- SNS (fan-out) + SQS (queuing, dead letters, visibility timeout).
- EventBridge for complex routing and scheduling.
- Kinesis vs MSK vs SQS/SNS depending on throughput and ordering needs.

**Observability & Ops**:
- CloudWatch Logs + Metrics + Alarms + Dashboards.
- X-Ray for distributed tracing.
- Cost optimization via right-sizing, Savings Plans, Spot, and intelligent tiering.

**General AWS Thinking**:
- Prefer managed services when they meet requirements (Athena, Fargate, RDS, etc.).
- Design for failure and cost from day one.
- Data transfer costs are often the silent killer.

## Docker

- Multi-stage builds (critical for small secure images).
- Layer caching strategies.
- Non-root users.
- Healthchecks and graceful shutdown.
- Image scanning and minimal base images (distroless, alpine, scratch where possible).

## CI/CD (GitHub Actions, Jenkins)

**GitHub Actions**:
- Matrix builds, reusable workflows, composite actions.
- Caching (actions/cache for dependencies, build outputs).
- OIDC for secure cloud access (no long-lived keys).
- Self-hosted runners for special needs (GPU, large builds).
- Environments + required reviewers.

**Jenkins**:
- Pipeline as code (Jenkinsfile).
- Shared libraries.
- Agent strategies.
- When to choose GitHub Actions vs Jenkins vs other (GitLab CI, Argo, etc.).

**Principles**:
- Fast feedback loops.
- Immutable artifacts.
- Promotion across environments.
- Security scanning in pipeline.

## Linux (Production Operations)

Must be comfortable with:
- Process management, signals, cgroups, namespaces (container basics).
- Networking (ss, netstat, tcpdump, ss -tuln, iptables/nftables basics).
- Performance (perf, iostat, vmstat, sar, pidstat, bpftrace).
- Filesystem (ext4 vs xfs, inode exhaustion, lsof).
- systemd (unit files, journalctl, resource limits).
- Shell scripting for ops (robust error handling, logging).

**Interview Tip**: Have a story about debugging a production issue using Linux tools (e.g., "high latency was actually packet loss visible only in tcpdump + retransmits in `ss -tim`").

## Resume Claim Examples
- "Architected and operated containerized services on AWS ECS Fargate and EKS, using GitHub Actions for CI/CD with OIDC, achieving >99.9% availability while reducing infrastructure cost by 35% through right-sizing and spot instances."
- "Built data pipelines on EMR and Athena with heavy S3 optimization (partitioning strategy, compression, lifecycle), reducing query costs by 60% and improving p95 query time from minutes to seconds."
