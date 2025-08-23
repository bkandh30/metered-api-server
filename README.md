# Metered API Server

A production-ready API server built with Rust that provides metered access with usage tracking, rate limiting, and quota management. Perfect for SaaS applications that need to monitor and control API usage per customer.

## Features

- **API Key Management**: Create, list, and delete API keys for customer access
- **Usage Tracking**: Automatic tracking of every API request with detailed metrics
- **Rate Limiting**: Configurable per-minute rate limits per API key
- **Quota Management**: Set maximum request quotas per API key
- **Detailed Analytics**: Usage statistics and monthly reports with CSV export
- **Request Logging**: Complete audit trail of all API requests
- **Input Validation**: Comprehensive validation of all inputs with detailed error messages
- **Health Monitoring**: Built-in health checks and metrics endpoints
- **Docker Support**: Full containerization with Docker and docker-compose

## Tech Stack

- **Rust** - Systems programming language for performance and safety
- **Warp** - Modern, composable web framework
- **SQLx** - Compile-time checked SQL queries
- **PostgreSQL** - Reliable relational database
- **Tokio** - Asynchronous runtime
- **Docker** - Containerization for easy deployment

## Quick Start

### Using Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/bkandh30/metered-api-server.git
cd metered-api-server

# Copy environment file
cp .env.example .env
# Edit .env with your database credentials

# Start the services
docker-compose up -d

# Check if services are running
docker-compose ps

# View logs
docker-compose logs -f
```

The API will be available at `http://localhost:3030`

### Local Development

```bash
# Clone and setup
git clone https://github.com/bkandh30/metered-api-server.git
cd metered-api-server

# Setup environment
cp .env.example .env
# Edit .env with your database credentials

# Run migrations
sqlx migrate run

# Run the server
cargo run
```

## API Endpoints

### Public Endpoints

- `GET /health` - Health check
- `GET /docs` - API documentation
- `GET /metrics` - System metrics

### Admin Endpoints

- `POST /admin/keys` - Create new API key
- `GET /admin/keys` - List all API keys
- `DELETE /admin/keys/{id}` - Delete API key
- `GET /admin/keys/{key}/stats` - Get usage statistics
- `GET /admin/keys/{key}/report` - Get monthly report

### Protected Endpoints (Require API Key)

- `POST /readings` - Submit sensor reading
- `GET /readings` - Get all readings

## Configuration

Environment variables can be set in `.env` file:

```env
# Database
DATABASE_URL=postgresql://user:password@localhost/api_key_db

# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=3030

# Logging
RUST_LOG=info
```

## Database Schema

The system uses three main tables:

- **api_keys** - Stores API keys with usage counts and limits
- **readings** - Business data (sensor readings in this example)
- **requests** - Complete request audit log

## Rate Limiting & Quotas

Each API key can have:

- **Rate Limit**: Requests per minute (default: 60)
- **Quota Limit**: Total requests allowed (optional)

When limits are exceeded:

- Rate limit: Returns `429 Too Many Requests`
- Quota exceeded: Returns `403 Forbidden`

## Testing

For comprehensive testing instructions, see [TESTING.md](./TESTING.md).

## Docker Commands

```bash
# Build images
docker-compose build

# Start services
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker-compose logs -f api

# Access database
docker-compose exec postgres psql -U apiuser -d metered_api

# Run migrations
docker-compose exec api sqlx migrate run

# Clean everything (including data)
docker-compose down -v
```

## Project Structure

<!-- ```
metered-api-server/
├── src/
│   ├── main.rs           # Application entry point
│   ├── db/               # Database connection module
│   ├── handlers/         # Request handlers
│   │   ├── admin.rs      # Admin endpoints
│   │   ├── business.rs   # Business logic endpoints
│   │   ├── usage.rs      # Usage tracking endpoints
│   │   └── metrics.rs    # Metrics endpoint
│   ├── middleware/       # Middleware components
│   │   ├── auth.rs       # Authentication & authorization
│   │   ├── rate_limiter.rs # Rate limiting
│   │   └── validation.rs # Input validation
│   └── models/           # Data models
├── migrations/           # SQL migration files
├── static/              # Static files (docs.html)
├── Cargo.toml           # Rust dependencies
├── Dockerfile           # Docker image definition
├── docker-compose.yml   # Docker services configuration
└── .env.example         # Environment variables template
``` -->

## Development Phases

This project was built in 5 phases:

1. **Phase 1**: Core API with key management and protected endpoints
2. **Phase 2**: Usage tracking and reporting
3. **Phase 3**: Rate limiting and quota management
4. **Phase 4**: Docker containerization
5. **Phase 5**: Input validation and metrics

## Performance

- Efficient connection pooling with configurable limits
- Asynchronous request handling with Tokio
- Compile-time SQL validation with SQLx
- In-memory rate limiting for minimal latency
- Optimized Docker images with multi-stage builds

## Security

- API keys are generated with cryptographically secure random values
- Keys are only shown once at creation time
- All inputs are validated and sanitized
- SQL injection prevention through parameterized queries
- Rate limiting prevents abuse
- Non-root Docker container user

## Error Handling

All errors follow a consistent JSON format:

```json
{
  "code": "ERROR_CODE",
  "message": "Human-readable error message"
}
```

## License

MIT License - see LICENSE file for details
