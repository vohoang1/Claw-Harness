"""
Claw-Harness Client

Provides both sync and async clients for interacting with Claw-Harness nodes.
"""

import asyncio
import httpx
from typing import Optional, Dict, Any, List, Union
from pydantic import BaseModel, Field
import logging

logger = logging.getLogger(__name__)


class Session(BaseModel):
    """Represents a conversation session"""
    id: str
    created_at: str
    updated_at: str
    messages: List[Dict[str, Any]] = Field(default_factory=list)


class Tool(BaseModel):
    """Represents a tool definition"""
    name: str
    description: str
    inputs: List[Dict[str, Any]] = Field(default_factory=list)
    outputs: List[Dict[str, Any]] = Field(default_factory=list)


class Job(BaseModel):
    """Represents a distributed job"""
    id: str
    session_id: str
    prompt: str
    state: str
    result: Optional[str] = None
    error: Optional[str] = None


class JobState:
    """Job state constants"""
    PENDING = "pending"
    RUNNING = "running"
    AWAIT_TOOL_CALL = "await_tool_call"
    COMPLETED = "completed"
    FAILED = "failed"


class Config(BaseModel):
    """Client configuration"""
    node_url: str = "http://localhost:8080"
    api_key: Optional[str] = None
    timeout: float = 30.0
    retry_count: int = 3


class ClawHarness:
    """
    Synchronous Claw-Harness Client
    
    Example:
        >>> from claw_harness import ClawHarness
        >>> client = ClawHarness(node_url="http://localhost:8080")
        >>> response = client.run("Explain quantum computing")
        >>> print(response)
    """
    
    def __init__(self, config: Optional[Config] = None, **kwargs):
        """
        Initialize Claw-Harness client
        
        Args:
            config: Configuration object
            **kwargs: Configuration overrides (node_url, api_key, timeout, etc.)
        """
        self.config = config or Config(**kwargs)
        self._client = httpx.Client(
            base_url=self.config.node_url,
            timeout=self.config.timeout,
            headers=self._get_headers()
        )
    
    def _get_headers(self) -> Dict[str, str]:
        """Get request headers"""
        headers = {"Content-Type": "application/json"}
        if self.config.api_key:
            headers["Authorization"] = f"Bearer {self.config.api_key}"
        return headers
    
    def run(self, prompt: str, tools: Optional[List[str]] = None) -> str:
        """
        Run a prompt through Claw-Harness
        
        Args:
            prompt: The prompt to execute
            tools: Optional list of tool names to use
            
        Returns:
            The AI response text
        """
        payload = {
            "prompt": prompt,
            "tools": tools or []
        }
        
        response = self._client.post("/api/v1/run", json=payload)
        response.raise_for_status()
        
        result = response.json()
        return result.get("result", "")
    
    def create_session(self) -> Session:
        """Create a new conversation session"""
        response = self._client.post("/api/v1/sessions")
        response.raise_for_status()
        return Session(**response.json())
    
    def get_session(self, session_id: str) -> Session:
        """Get a session by ID"""
        response = self._client.get(f"/api/v1/sessions/{session_id}")
        response.raise_for_status()
        return Session(**response.json())
    
    def list_tools(self) -> List[Tool]:
        """List available tools"""
        response = self._client.get("/api/v1/tools")
        response.raise_for_status()
        return [Tool(**t) for t in response.json()]
    
    def get_job(self, job_id: str) -> Job:
        """Get job status"""
        response = self._client.get(f"/api/v1/jobs/{job_id}")
        response.raise_for_status()
        return Job(**response.json())
    
    def close(self):
        """Close the client connection"""
        self._client.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()


class AsyncClawHarness:
    """
    Asynchronous Claw-Harness Client
    
    Example:
        >>> import asyncio
        >>> from claw_harness import AsyncClawHarness
        >>> 
        >>> async def main():
        ...     client = AsyncClawHarness()
        ...     response = await client.run("Explain quantum computing")
        ...     print(response)
        ... 
        >>> asyncio.run(main())
    """
    
    def __init__(self, config: Optional[Config] = None, **kwargs):
        """
        Initialize async Claw-Harness client
        
        Args:
            config: Configuration object
            **kwargs: Configuration overrides
        """
        self.config = config or Config(**kwargs)
        self._client: Optional[httpx.AsyncClient] = None
    
    def _get_headers(self) -> Dict[str, str]:
        """Get request headers"""
        headers = {"Content-Type": "application/json"}
        if self.config.api_key:
            headers["Authorization"] = f"Bearer {self.config.api_key}"
        return headers
    
    async def _get_client(self) -> httpx.AsyncClient:
        """Get or create async client"""
        if self._client is None:
            self._client = httpx.AsyncClient(
                base_url=self.config.node_url,
                timeout=self.config.timeout,
                headers=self._get_headers()
            )
        return self._client
    
    async def run(self, prompt: str, tools: Optional[List[str]] = None) -> str:
        """
        Run a prompt asynchronously
        
        Args:
            prompt: The prompt to execute
            tools: Optional list of tool names
            
        Returns:
            The AI response text
        """
        client = await self._get_client()
        payload = {
            "prompt": prompt,
            "tools": tools or []
        }
        
        response = await client.post("/api/v1/run", json=payload)
        response.raise_for_status()
        
        result = response.json()
        return result.get("result", "")
    
    async def create_session(self) -> Session:
        """Create a new conversation session"""
        client = await self._get_client()
        response = await client.post("/api/v1/sessions")
        response.raise_for_status()
        return Session(**response.json())
    
    async def get_session(self, session_id: str) -> Session:
        """Get a session by ID"""
        client = await self._get_client()
        response = await client.get(f"/api/v1/sessions/{session_id}")
        response.raise_for_status()
        return Session(**response.json())
    
    async def list_tools(self) -> List[Tool]:
        """List available tools"""
        client = await self._get_client()
        response = await client.get("/api/v1/tools")
        response.raise_for_status()
        return [Tool(**t) for t in response.json()]
    
    async def get_job(self, job_id: str) -> Job:
        """Get job status"""
        client = await self._get_client()
        response = await client.get(f"/api/v1/jobs/{job_id}")
        response.raise_for_status()
        return Job(**response.json())
    
    async def close(self):
        """Close the client connection"""
        if self._client:
            await self._client.aclose()
            self._client = None
    
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.close()


# Convenience function for quick usage
async def run(prompt: str, node_url: str = "http://localhost:8080", **kwargs) -> str:
    """
    Quick async run function
    
    Args:
        prompt: The prompt to execute
        node_url: Claw-Harness node URL
        **kwargs: Additional configuration
        
    Returns:
        The AI response text
    """
    async with AsyncClawHarness(node_url=node_url, **kwargs) as client:
        return await client.run(prompt)
