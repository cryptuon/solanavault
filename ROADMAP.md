# SolanaVault Roadmap

> This roadmap describes direction and intent. Dates are targets, not commitments.
> SolanaVault is under active development; layouts and APIs may change between releases.

## Vision

SolanaVault aims to be the **DePIN storage and state-scaling layer for Solana** — the
place where compressed blocks and account history live, and the network that serves them
back with sub-millisecond retrieval, without changing the Solana RPC surface applications
already speak.

The bet for 2026 and beyond: **throughput growth makes state and history a shared
infrastructure problem, not a per-operator cost center.** As agentic payments, on-chain
and verifiable AI, and high-frequency RWA settlement push machine-speed read traffic
against historical and account data, the economical answer is not "everyone runs an
archive node" or "everyone trusts one RPC vendor." It is a network of independent storage
operators holding compressed data cooperatively, paid to serve it, and exposing it through
the same `getConfirmedBlock`/account methods callers already use.

Three principles constrain every milestone below:

1. **The RPC surface never changes.** Compatibility is the product. If a change requires
   callers to learn a new dialect, it is out of scope.
2. **Durability must be provable, not promised.** A decentralized storage network is only
   as good as its guarantees that data is actually there and actually correct.
3. **Honest scope.** SolanaVault is a storage-and-retrieval layer. It is not a new L1, a
   new VM, or a consensus replacement for Solana itself.

## Where we are today

The workspace ships four binaries — `vault-cli`, `vault-node`, `vault-rpc-proxy`,
`vault-rpc-decentralized` — plus the on-chain tokenomics programs
(`vault-token`, `vault-staking`, `vault-rewards`, `vault-governance`). The core crate
(`vault-core`) implements P2P networking (NNG transport), Kademlia DHT peer discovery,
Byzantine consensus for integrity, the multi-stage compression pipeline (15-25:1), and an
embeddable light-client module. Compression, retrieval latency, and RPC pass-through are
demonstrable on real Solana data. What is *not* yet production-hardened is the set of
guarantees a DePIN network needs to be trusted with data at scale — see "Cheapest path to
production" below.

## Milestones

### M1 — Compression + drop-in RPC (shipped / stabilizing)
- Multi-stage compression pipeline at 15-25:1 on real blocks
- `vault-rpc-proxy` serving standard Solana RPC methods over compressed storage
- Light-client verification module consumed by gateway and proxy
- **Exit criteria:** RPC-compat conformance suite (below) green against a reference validator

### M2 — Decentralized network hardening (in progress)
- Kademlia DHT discovery + NNG transport under adversarial/lossy conditions
- Byzantine consensus on read correctness across independent operators
- Automatic replication with configurable replication factor
- **Exit criteria:** documented durability/replication guarantees with measured recovery

### M3 — Incentive layer live on-chain
- `vault-staking` / `vault-rewards` / `vault-slashing` wired to real operator behavior
- Retrieval fee market: operators earn for served reads, clients pay per use
- Slashing driven by proof-of-retrieval challenge failures, not manual reports
- **Exit criteria:** operator can stake, serve, earn, and be slashed end-to-end on devnet

### M4 — Data-availability proofs
- Proof-of-retrieval / storage challenges that verify data is held, not just claimed
- Sampling-based availability checks a light client can run cheaply
- **Exit criteria:** a client can reject a lying operator without downloading full data

### M5 — Operator experience + observability
- One-command operator onboarding on cheap commodity storage
- Prometheus metrics (already scaffolded via `metrics-exporter-prometheus`) + dashboards
- Reputation-aware routing so slow/unreliable operators are deprioritized automatically
- **Exit criteria:** a new operator reaches "earning and healthy" without hand-holding

### M6 — Production readiness
- External review of consensus, incentive, and slashing logic
- Mainnet-scale load and failure testing
- Stable on-chain layouts and versioned upgrade path

## Cheapest path to production

The most capital-efficient route to a production DePIN storage network is to **use Solana
itself for coordination and settlement, and cheap commodity storage for the bytes** — and
to resist the temptation to build custom infrastructure where existing primitives suffice.

**Recommended architecture for the cheapest viable path:**

- **Coordination on Solana, data off-chain.** Keep operator registration, staking, reward
  epochs, and slashing on-chain in the existing `vault-*` programs — Solana is cheap and
  fast for this. Never put compressed block bytes on-chain; they live with operators.
- **P2P operators on commodity storage.** Target inexpensive high-density disk (spinning
  disk or cheap SSD) for the compressed archive, with an SSD hot tier only for the working
  set. Because compression is already 15-25:1, the effective cost floor is a fraction of a
  raw archive node. This is where the DePIN economics come from: operators monetize
  storage they can source cheaply.
- **Reuse, don't reinvent.** Kademlia DHT (discovery), NNG (transport), and Solana (ledger
  of record) already exist and are proven. Building bespoke replacements is the fastest way
  to burn runway.

**Production-viability requirements** — the network is not "done" until each of these
exists and is measured:

1. **Durability / replication guarantees.** A stated, tested replication factor with
   measured recovery time when operators drop. Data must survive the loss of *f* operators,
   and that *f* must be documented, not aspirational.
2. **Incentive layer.** Live staking, per-read retrieval fees, epoch rewards, and slashing
   that actually change operator payouts. Storing and serving honestly must be the
   profit-maximizing strategy; withholding or lying must lose money.
3. **Data-availability proofs.** Proof-of-retrieval / storage challenges so the network
   (and light clients) can verify data is truly held and correct — not merely claimed.
   Without DA proofs, "decentralized storage" is trust-me storage.
4. **RPC-compat conformance tests.** An automated suite that diffs SolanaVault responses
   against a reference Solana validator across the supported RPC methods, including edge
   cases (missing blocks, reorged slots, large accounts). Drop-in compatibility is only
   real if it is continuously proven.
5. **Operator monitoring.** Prometheus/Grafana observability (metrics already scaffolded),
   reputation-aware routing, and alerting so degraded operators are detected and
   deprioritized before clients feel it.

Ship these five, and SolanaVault crosses from "compelling demo" to "infrastructure teams
can depend on."

## Non-goals

- Replacing Solana consensus or becoming an L1/L2
- A new RPC dialect or a required client SDK swap
- On-chain storage of block/account bytes
- Custom crypto where audited, standard primitives already do the job

## Contributing to the roadmap

Direction is set in the open. Priorities, disagreements, and proposals are welcome via
GitHub issues and pull requests — see [CONTRIBUTING.md](./CONTRIBUTING.md). Milestone
ordering will shift based on operator demand and what the DePIN-storage market actually
needs first.
