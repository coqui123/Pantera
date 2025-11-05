# Quick Start Guide

## Running with Web UI

To run the Mango Data Service with the web UI enabled, use the `web-ui` feature flag:

### Development Mode

```bash
# Build and run with web UI
cargo run --features web-ui

# Or specify the release profile
cargo run --release --features web-ui
```

### Production Build

```bash
# Build release with web UI
cargo build --release --features web-ui

# Run the binary
./target/release/mango-data-service
```

## Running without Web UI (API only)

```bash
# Run without web UI (default)
cargo run

# Or explicitly
cargo run --release
```

## Accessing the Web UI

Once the server is running with the `web-ui` feature:

1. **Dashboard**: http://localhost:3000/ or http://localhost:3000/ui
2. **Search Interface**: http://localhost:3000/ui/search
3. **Analytics Interface**: http://localhost:3000/ui/analytics

## Configuration

The server port can be configured via environment variable:

```bash
# Windows PowerShell
$env:PORT=8080; cargo run --features web-ui

# Windows CMD
set PORT=8080 && cargo run --features web-ui

# Linux/Mac
PORT=8080 cargo run --features web-ui
```

## Environment Variables

Create a `.env` file in the project root (optional):

```env
# Database
DATABASE_URL=sqlite:./data/data.db

# Server
PORT=3000
HOST=0.0.0.0

# CORS (for production, specify allowed origins)
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://yourdomain.com

# Rate Limiting
API_RATE_LIMIT_PER_MINUTE=100
YAHOO_API_RATE_LIMIT_PER_MINUTE=120  # Increased default (2 requests/second)

# Cache Configuration
CACHE_TTL_QUOTES=300
CACHE_TTL_HISTORICAL=3600
CACHE_TTL_PROFILES=86400
CACHE_MAX_SIZE_HISTORICAL=1000
CACHE_MAX_SIZE_QUOTES=500
CACHE_MAX_SIZE_PROFILES=200
```

## Troubleshooting

### Web UI not showing

1. Make sure you built with `--features web-ui`
2. Check that templates exist in the `templates/` directory
3. Verify the server started successfully (check logs)

### Port already in use

Change the port:
```bash
PORT=8080 cargo run --features web-ui
```

### Database errors

The database will be created automatically in `./data/data.db` if it doesn't exist.

