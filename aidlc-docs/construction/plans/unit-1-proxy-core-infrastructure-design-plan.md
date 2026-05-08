# Infrastructure Design Plan — Unit 1: Proxy Core

## Plan Checkboxes

- [x] Answer questions below
- [x] Generate `infrastructure-design.md`
- [x] Generate `deployment-architecture.md`

---

## Questions

### Question 1: Default Port

What port should the proxy listen on by default?

A) 8080 — common convention for containerised HTTP services
B) 3000 — common for Node/web-adjacent tooling
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 2: Prometheus Metrics Port

Should the `/metrics` endpoint be served on the same port as the main proxy, or on a separate dedicated port?

A) Same port — simpler setup; one port to expose
B) Separate port (e.g. 9090) — allows metrics to be network-isolated from the proxy API
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 3: TLS Termination

The proxy should support HTTPS (SECURITY-03). Where should TLS be terminated?

A) External reverse proxy only — the binary listens on plain HTTP; TLS is handled by nginx/Caddy/Traefik in front of it (recommended for Docker deployments)
B) In the binary — the Rust server loads a TLS certificate and terminates TLS itself using `rustls`
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

Please fill in all `[Answer]:` tags and let me know when done.
