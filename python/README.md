# Claw-Harness Python SDK

Python client library for the Claw-Harness distributed AI agent platform.

## Installation

```bash
# Install from PyPI
pip install claw-harness

# Install with async support
pip install claw-harness[async]

# Install with gRPC support
pip install claw-harness[grpc]

# Install development dependencies
pip install claw-harness[dev]
```

## Quick Start

### Synchronous Usage

```python
from claw_harness import ClawHarness

# Initialize client
client = ClawHarness(node_url="http://localhost:8080")

# Run a prompt
response = client.run("Explain quantum computing in simple terms")
print(response)

# Use context manager
with ClawHarness() as client:
    response = client.run("What is Rust?")
    print(response)
```

### Asynchronous Usage

```python
import asyncio
from claw_harness import AsyncClawHarness

async def main():
    async with AsyncClawHarness() as client:
        # Run a prompt
        response = await client.run("Explain machine learning")
        print(response)
        
        # Create a session
        session = await client.create_session()
        print(f"Session ID: {session.id}")
        
        # List available tools
        tools = await client.list_tools()
        print(f"Available tools: {[t.name for t in tools]}")

asyncio.run(main())
```

### Quick Function

```python
from claw_harness import run

# Simplest way to run a prompt
response = await run("What is the capital of France?")
print(response)
```

## Configuration

```python
from claw_harness import Config, ClawHarness

config = Config(
    node_url="http://localhost:8080",
    api_key="your-api-key",  # Optional
    timeout=60.0,  # Request timeout in seconds
    retry_count=3  # Number of retries
)

client = ClawHarness(config=config)
```

## API Reference

### ClawHarness (Sync)

- `run(prompt: str, tools: List[str] = None) -> str` - Run a prompt
- `create_session() -> Session` - Create new session
- `get_session(session_id: str) -> Session` - Get session details
- `list_tools() -> List[Tool]` - List available tools
- `get_job(job_id: str) -> Job` - Get job status
- `close()` - Close connection

### AsyncClawHarness (Async)

Same methods as `ClawHarness` but async:

- `async run(prompt: str, tools: List[str] = None) -> str`
- `async create_session() -> Session`
- `async get_session(session_id: str) -> Session`
- `async list_tools() -> List[Tool]`
- `async get_job(job_id: str) -> Job`
- `async close()`

## Error Handling

```python
from claw_harness import ClawHarness
import httpx

client = ClawHarness()

try:
    response = client.run("Your prompt")
except httpx.HTTPError as e:
    print(f"HTTP error: {e}")
except Exception as e:
    print(f"Error: {e}")
```

## Distributed Mode

When connected to a distributed cluster, the SDK automatically load-balances across nodes:

```python
from claw_harness import AsyncClawHarness

# Connect to load balancer
client = AsyncClawHarness(node_url="http://load-balancer:8080")

# Requests are automatically distributed
response = await client.run("Process this large task")
```

## License

MIT License
