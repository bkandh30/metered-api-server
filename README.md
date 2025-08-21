# Metered API Server (Rust)

A secure, production-ready API server built in Rust to provide metered access and usage tracking via API keys.

---

##  ​ Project Overview

This service offers authenticated API access with usage tracking, quotas, and rate limiting. Designed using industry-standard best practices, it features structured logging, health checks, OpenAPI documentation, containerization, and CI/CD workflows.

---

##  ​ Features

- API key lifecycle management (create/delete, authenticate, track usage)
- Quotas enforcement and in-memory rate limiting
- Structured logging with request IDs
- Health and readiness probes (`/healthz`, `/readyz`)
- OpenAPI documentation (`/docs`)
- Containerized deployment via Docker Compose
- CI/CD automation using GitHub Actions

---

##  ​ Current Progress

- [x] Core API with Rust, Warp, Tokio, and SQLx  
- [x] API key management endpoints and authentication middleware  
- [x] Usage tracking with PostgreSQL-backed counting  
- [x] Quota enforcement and basic rate limiting  
- [x] Dockerfile and `docker-compose.yml` for local and deployment setup  
- [ ] Structured logging, health endpoints, OpenAPI docs, configuration management, and CI/CD integration (in progress)

---

##  ​ Roadmap

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 |  Done | Core API, API key CRUD, authentication middleware |
| Phase 2 |  Done | Usage tracking & reporting |
| Phase 3 |  Done | Quotas & rate limiting |
| Phase 4 |  Done | Docker + PostgreSQL with Docker Compose |
| Phase 5 |  In Progress | Structured logging, `/healthz`, config management, OpenAPI spec, CI pipelines |

---

##  ​ Tech Stack

- **Rust** (Warp, Tokio, sqlx, dotenvy)  
- **PostgreSQL** (via SQLx)  
- **Docker & Docker Compose**  
- **OpenAPI** (e.g., using `utoipa`)  
- **CI/CD** (GitHub Actions: linting, tests, build)

---

##  ​ Getting Started

1. Clone the repository and enter the project directory:
    ```bash
    git clone https://github.com/bkandh30/metered-api-server.git
    cd metered-api-server
    ```

2. Setup environment variables in a `.env` file:
    ```dotenv
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/metered
    ```

3. Run locally with Docker Compose:
    ```bash
    docker compose up --build
    ```

4. Access:
    - API: `http://localhost:8080`
    - OpenAPI UI (Swagger): `http://localhost:8080/docs`
    - Health checks: `http://localhost:8080/healthz` and `/readyz`

---

##  ​ Contributing

Contributions, issues, and discussions are welcome!  
Feel free to **fork**, **open an issue**, or submit a **pull request**.

---

##  ​ License

This project is licensed under the **MIT License** — see the `LICENSE` file for details.
