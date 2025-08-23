# Testing Guide for Metered API Server

This guide provides comprehensive testing instructions for all API endpoints using both Postman and curl commands.

## Prerequisites

1. Server running at `http://localhost:3030`
2. PostgreSQL database running and accessible
3. Postman installed (for Postman tests) or terminal access (for curl)

## Phase 1: Core API Testing

### 1.1 Health Check

**Postman:**

- Method: `GET`
- URL: `http://localhost:3030/health`

**Curl:**

```bash
curl http://localhost:3030/health
```

**Expected Response:**

```json
{
  "status": "healthy"
}
```

### 1.2 Create API Key

**Postman:**

- Method: `POST`
- URL: `http://localhost:3030/admin/keys`
- Body (JSON):

```json
{
  "name": "Test API Key"
}
```

**Curl:**

```bash
curl -X POST http://localhost:3030/admin/keys \
  -H "Content-Type: application/json" \
  -d '{"name": "Test API Key"}'
```

**Expected Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "key": "sk_ABCDEFGHIJKLMNOPQRSTUVWXYZabcd",
  "name": "Test API Key"
}
```

⚠️ **Important:** Save the returned `key` value - it's only shown once!

### 1.3 List All API Keys

**Postman:**

- Method: `GET`
- URL: `http://localhost:3030/admin/keys`

**Curl:**

```bash
curl http://localhost:3030/admin/keys
```

**Expected Response:**

```json
{
  "keys": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Test API Key",
      "usage_count": 0,
      "is_active": true,
      "created_at": "2025-08-22T10:30:00Z"
    }
  ]
}
```

### 1.4 Submit Reading (Protected Endpoint)

**Postman:**

- Method: `POST`
- URL: `http://localhost:3030/readings`
- Headers:
  - `X-Api-Key`: `sk_YOUR_API_KEY_HERE`
- Body (JSON):

```json
{
  "sensor_id": "temp-sensor-01",
  "value": 23.5,
  "unit": "celsius"
}
```

**Curl:**

```bash
curl -X POST http://localhost:3030/readings \
  -H "X-Api-Key: sk_YOUR_API_KEY_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "sensor_id": "temp-sensor-01",
    "value": 23.5,
    "unit": "celsius"
  }'
```

### 1.5 Get Readings (Protected Endpoint)

**Postman:**

- Method: `GET`
- URL: `http://localhost:3030/readings`
- Headers:
  - `X-Api-Key`: `sk_YOUR_API_KEY_HERE`

**Curl:**

```bash
curl http://localhost:3030/readings \
  -H "X-Api-Key: sk_YOUR_API_KEY_HERE"
```

### 1.6 Delete API Key

**Postman:**

- Method: `DELETE`
- URL: `http://localhost:3030/admin/keys/{id}`
  - Replace `{id}` with actual UUID

**Curl:**

```bash
curl -X DELETE http://localhost:3030/admin/keys/550e8400-e29b-41d4-a716-446655440000
```

## Phase 2: Usage Tracking & Reporting Testing

### 2.1 Generate Test Data

First, make several requests to populate the usage data:

```bash
# Make 10 requests with your API key
for i in {1..10}; do
  curl -X POST http://localhost:3030/readings \
    -H "X-Api-Key: sk_YOUR_API_KEY_HERE" \
    -H "Content-Type: application/json" \
    -d "{\"sensor_id\": \"sensor-$i\", \"value\": $((20 + $i)), \"unit\": \"celsius\"}"
  sleep 1
done
```

### 2.2 Get Usage Statistics

**Postman:**

- Method: `GET`
- URL: `http://localhost:3030/admin/keys/{key}/stats`
  - Replace `{key}` with your actual API key

**Curl:**

```bash
curl http://localhost:3030/admin/keys/sk_YOUR_API_KEY_HERE/stats
```

**Expected Response:**

```json
{
  "api_key_name": "Test API Key",
  "total_requests": 15,
  "requests_today": 15,
  "requests_this_month": 15,
  "last_used": "2025-08-22T14:30:45Z"
}
```

### 2.3 Get Monthly Report (JSON)

**Postman:**

- Method: `GET`
- URL: `http://localhost:3030/admin/keys/{key}/report`

**Curl:**

```bash
curl http://localhost:3030/admin/keys/sk_YOUR_API_KEY_HERE/report
```

### 2.4 Get Monthly Report (CSV)

**Postman:**

- Method: `GET`
- URL: `http://localhost:3030/admin/keys/{key}/report?format=csv`

**Curl:**

```bash
curl "http://localhost:3030/admin/keys/sk_YOUR_API_KEY_HERE/report?format=csv"
```

## Phase 3: Rate Limiting & Quota Testing

### 3.1 Setup Test API Key with Limits

First, create a test key and update its limits via SQL:

```sql
-- Connect to database
docker-compose exec postgres psql -U apiuser -d metered_api

-- Update limits for testing
UPDATE api_keys
SET quota_limit = 20,
    rate_limit_per_minute = 5,
    usage_count = 0
WHERE key = 'sk_YOUR_TEST_KEY';
```

### 3.2 Test Rate Limiting

**Rapid fire test (should trigger rate limit):**

```bash
# Make 10 rapid requests
for i in {1..10}; do
  echo "Request $i:"
  curl -w "\nHTTP Status: %{http_code}\n" \
    http://localhost:3030/readings \
    -H "X-Api-Key: sk_YOUR_TEST_KEY"
done
```

**Expected:**

- Requests 1-5: HTTP 200
- Requests 6-10: HTTP 429 (Rate limit exceeded)

### 3.3 Test Quota Limits

```bash
# Set usage count near quota
docker-compose exec postgres psql -U apiuser -d metered_api \
  -c "UPDATE api_keys SET usage_count = 18 WHERE key = 'sk_YOUR_TEST_KEY';"

# Make 5 requests (slowly to avoid rate limit)
for i in {1..5}; do
  echo "Request $i:"
  curl -w "\nHTTP Status: %{http_code}\n" \
    http://localhost:3030/readings \
    -H "X-Api-Key: sk_YOUR_TEST_KEY"
  sleep 15  # Wait to avoid rate limit
done
```

**Expected:**

- Requests 1-2: HTTP 200 (usage reaches 20)
- Requests 3-5: HTTP 403 (Quota exceeded)

## Phase 4: Docker Testing

### 4.1 Build and Start Services

```bash
# Build images
docker-compose build

# Start services
docker-compose up -d

# Verify containers are running
docker-compose ps
```

### 4.2 Test Container Health

```bash
# Check API health through Docker
docker-compose exec api curl http://localhost:3030/health

# View logs
docker-compose logs -f api
```

### 4.3 Database Connectivity Test

```bash
# Connect to database container
docker-compose exec postgres psql -U apiuser -d metered_api -c "SELECT COUNT(*) FROM api_keys;"
```

## Phase 5: Validation, Documentation & Metrics Testing

### 5.1 Test Input Validation

**Invalid Sensor ID:**

```bash
curl -X POST http://localhost:3030/readings \
  -H "X-Api-Key: sk_YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"sensor_id": "invalid@sensor!", "value": 23.5, "unit": "celsius"}'
```

**Expected:** HTTP 400 with validation error

**Invalid Value (too large):**

```bash
curl -X POST http://localhost:3030/readings \
  -H "X-Api-Key: sk_YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"sensor_id": "sensor1", "value": 9999999, "unit": "celsius"}'
```

**Expected:** HTTP 400 with validation error

**Empty Required Fields:**

```bash
curl -X POST http://localhost:3030/readings \
  -H "X-Api-Key: sk_YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"sensor_id": "", "value": 23.5, "unit": ""}'
```

**Expected:** HTTP 400 with validation error

### 5.2 Test Documentation Endpoint

**Browser:**
Open `http://localhost:3030/docs` in your browser

**Curl:**

```bash
curl http://localhost:3030/docs | head -20
```

**Expected:** HTML documentation page

### 5.3 Test Metrics Endpoint

**Postman:**

- Method: `GET`
- URL: `http://localhost:3030/metrics`

**Curl:**

```bash
curl http://localhost:3030/metrics | jq '.'
```

**Expected Response Structure:**

```json
{
  "total_requests": 156,
  "total_api_keys": 3,
  "active_api_keys": 2,
  "avg_response_time_ms": 45.2,
  "requests_last_hour": 23,
  "requests_last_24h": 156,
  "top_endpoints": [
    {
      "endpoint": "/readings",
      "count": 120
    }
  ],
  "status_distribution": {
    "success_2xx": 140,
    "client_error_4xx": 15,
    "server_error_5xx": 1
  },
  "database_pool_stats": {
    "size": 5,
    "num_idle": 3
  }
}
```

## Error Testing

### Test Missing API Key

```bash
curl -X POST http://localhost:3030/readings \
  -H "Content-Type: application/json" \
  -d '{"sensor_id": "test", "value": 23.5, "unit": "celsius"}'
```

**Expected:** HTTP 401 Unauthorized

### Test Invalid API Key

```bash
curl -X GET http://localhost:3030/readings \
  -H "X-Api-Key: sk_invalid_key_12345"
```

**Expected:** HTTP 401 Unauthorized

### Test Non-existent Endpoint

```bash
curl http://localhost:3030/nonexistent
```

**Expected:** HTTP 404 Not Found

### Test Method Not Allowed

```bash
curl -X PUT http://localhost:3030/health
```

**Expected:** HTTP 405 Method Not Allowed

## Load Testing

### Simple Load Test

```bash
# Create a test script
cat > load_test.sh << 'EOF'
#!/bin/bash
API_KEY="sk_YOUR_API_KEY"
URL="http://localhost:3030/readings"
CONCURRENT=10
TOTAL=100

echo "Starting load test: $TOTAL requests with $CONCURRENT concurrent"

for i in $(seq 1 $CONCURRENT); do
  (
    for j in $(seq 1 $(($TOTAL / $CONCURRENT))); do
      curl -s -o /dev/null -w "%{http_code}\n" \
        -H "X-Api-Key: $API_KEY" \
        $URL
    done
  ) &
done

wait
echo "Load test complete"
EOF

chmod +x load_test.sh
./load_test.sh
```

## Postman Collection Setup

### Environment Variables

Create a Postman environment with:

- `base_url`: `http://localhost:3030`
- `api_key`: (set after creating a key)
- `api_key_id`: (UUID from key creation)

### Collection Runner

1. Create a collection with all endpoints
2. Set up test scripts to validate responses
3. Use Collection Runner for automated testing
4. Configure iterations and delays between requests

### Example Postman Test Script

```javascript
// For Create API Key endpoint
pm.test("Status is 201", function () {
  pm.response.to.have.status(201);
});

pm.test("Response has key", function () {
  var jsonData = pm.response.json();
  pm.expect(jsonData).to.have.property("key");
  pm.environment.set("api_key", jsonData.key);
  pm.environment.set("api_key_id", jsonData.id);
});
```

## Troubleshooting

### Common Issues

1. **Connection Refused**

   - Check if server is running: `docker-compose ps`
   - Check logs: `docker-compose logs api`

2. **Database Connection Failed**

   - Verify PostgreSQL is running: `docker-compose ps postgres`
   - Check database credentials in `.env`

3. **Rate Limit Hit Too Quickly**

   - Wait 60 seconds for rate limit window to reset
   - Check rate limit configuration in database

4. **Quota Exceeded**
   - Reset usage count: `UPDATE api_keys SET usage_count = 0 WHERE key = 'sk_YOUR_KEY';`

## Clean Up

```bash
# Stop all services
docker-compose down

# Remove all data (fresh start)
docker-compose down -v

# Remove Docker images
docker-compose down --rmi all
```
