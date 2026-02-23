# Python MCP Server Implementation Guide

## Overview

This document provides Python-specific best practices and examples for implementing MCP servers using the MCP Python SDK (FastMCP). It covers server setup, tool registration patterns, input validation with Pydantic, error handling, and complete working examples.

---

## Quick Reference

### Key Imports
```python
from mcp.server.fastmcp import FastMCP
from pydantic import BaseModel, Field, field_validator, ConfigDict
from typing import Optional, List, Dict, Any
from enum import Enum
import httpx
```

### Server Initialization
```python
mcp = FastMCP("service_mcp")
```

### Tool Registration Pattern
```python
@mcp.tool(name="tool_name", annotations={...})
async def tool_function(params: InputModel) -> str:
    # Implementation
    pass
```

---

## Server Naming Convention

Format: `{service}_mcp` (lowercase with underscores)
- Examples: `github_mcp`, `jira_mcp`, `stripe_mcp`

## Tool Implementation

### Tool Structure with FastMCP

```python
from pydantic import BaseModel, Field, ConfigDict
from mcp.server.fastmcp import FastMCP

mcp = FastMCP("example_mcp")

class ServiceToolInput(BaseModel):
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra='forbid'
    )

    param1: str = Field(..., description="First parameter description", min_length=1, max_length=100)
    param2: Optional[int] = Field(default=None, description="Optional integer parameter", ge=0, le=1000)

@mcp.tool(
    name="service_tool_name",
    annotations={
        "title": "Human-Readable Tool Title",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False
    }
)
async def service_tool_name(params: ServiceToolInput) -> str:
    '''Tool description automatically becomes the description field.

    Args:
        params (ServiceToolInput): Validated input parameters

    Returns:
        str: JSON-formatted response containing operation results
    '''
    pass
```

## Pydantic v2 Key Features

- Use `model_config` instead of nested `Config` class
- Use `field_validator` instead of deprecated `validator`
- Use `model_dump()` instead of deprecated `dict()`
- Validators require `@classmethod` decorator

```python
from pydantic import BaseModel, Field, field_validator, ConfigDict

class CreateUserInput(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True, validate_assignment=True)

    name: str = Field(..., description="User's full name", min_length=1, max_length=100)
    email: str = Field(..., description="User's email address", pattern=r'^[\w\.-]+@[\w\.-]+\.\w+$')

    @field_validator('email')
    @classmethod
    def validate_email(cls, v: str) -> str:
        if not v.strip():
            raise ValueError("Email cannot be empty")
        return v.lower()
```

## Response Format Options

```python
from enum import Enum

class ResponseFormat(str, Enum):
    MARKDOWN = "markdown"
    JSON = "json"

class UserSearchInput(BaseModel):
    query: str = Field(..., description="Search query")
    response_format: ResponseFormat = Field(
        default=ResponseFormat.MARKDOWN,
        description="Output format: 'markdown' for human-readable or 'json' for machine-readable"
    )
```

## Pagination Implementation

```python
class ListInput(BaseModel):
    limit: Optional[int] = Field(default=20, description="Maximum results to return", ge=1, le=100)
    offset: Optional[int] = Field(default=0, description="Number of results to skip", ge=0)

async def list_items(params: ListInput) -> str:
    data = await api_request(limit=params.limit, offset=params.offset)
    response = {
        "total": data["total"],
        "count": len(data["items"]),
        "offset": params.offset,
        "items": data["items"],
        "has_more": data["total"] > params.offset + len(data["items"]),
        "next_offset": params.offset + len(data["items"])
            if data["total"] > params.offset + len(data["items"]) else None
    }
    return json.dumps(response, indent=2)
```

## Error Handling

```python
def _handle_api_error(e: Exception) -> str:
    if isinstance(e, httpx.HTTPStatusError):
        if e.response.status_code == 404:
            return "Error: Resource not found. Please check the ID is correct."
        elif e.response.status_code == 403:
            return "Error: Permission denied. You don't have access to this resource."
        elif e.response.status_code == 429:
            return "Error: Rate limit exceeded. Please wait before making more requests."
        return f"Error: API request failed with status {e.response.status_code}"
    elif isinstance(e, httpx.TimeoutException):
        return "Error: Request timed out. Please try again."
    return f"Error: Unexpected error occurred: {type(e).__name__}"
```

## Shared Utilities

```python
async def _make_api_request(endpoint: str, method: str = "GET", **kwargs) -> dict:
    async with httpx.AsyncClient() as client:
        response = await client.request(
            method,
            f"{API_BASE_URL}/{endpoint}",
            timeout=30.0,
            **kwargs
        )
        response.raise_for_status()
        return response.json()
```

## Transport Options

```python
# stdio transport (for local tools) - default
if __name__ == "__main__":
    mcp.run()

# Streamable HTTP transport (for remote servers)
if __name__ == "__main__":
    mcp.run(transport="streamable_http", port=8000)
```

## Complete Example

```python
#!/usr/bin/env python3
from typing import Optional
from enum import Enum
import httpx
from pydantic import BaseModel, Field, ConfigDict
from mcp.server.fastmcp import FastMCP

mcp = FastMCP("example_mcp")
API_BASE_URL = "https://api.example.com/v1"

class ResponseFormat(str, Enum):
    MARKDOWN = "markdown"
    JSON = "json"

class UserSearchInput(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True, validate_assignment=True)
    query: str = Field(..., description="Search string", min_length=2, max_length=200)
    limit: Optional[int] = Field(default=20, description="Max results", ge=1, le=100)
    offset: Optional[int] = Field(default=0, description="Skip N results", ge=0)
    response_format: ResponseFormat = Field(default=ResponseFormat.MARKDOWN)

async def _make_api_request(endpoint: str, **kwargs) -> dict:
    async with httpx.AsyncClient() as client:
        response = await client.get(f"{API_BASE_URL}/{endpoint}", timeout=30.0, **kwargs)
        response.raise_for_status()
        return response.json()

@mcp.tool(
    name="example_search_users",
    annotations={"title": "Search Users", "readOnlyHint": True, "destructiveHint": False}
)
async def example_search_users(params: UserSearchInput) -> str:
    '''Search for users in the Example system.'''
    try:
        data = await _make_api_request("users/search", params={"q": params.query, "limit": params.limit})
        users = data.get("users", [])
        if not users:
            return f"No users found matching '{params.query}'"
        if params.response_format == ResponseFormat.MARKDOWN:
            lines = [f"# Results for '{params.query}'", ""]
            for user in users:
                lines.append(f"## {user['name']} ({user['id']})")
                lines.append(f"- **Email**: {user['email']}")
                lines.append("")
            return "\n".join(lines)
        else:
            import json
            return json.dumps({"users": users, "total": data.get("total", 0)}, indent=2)
    except Exception as e:
        return f"Error: {type(e).__name__}: {str(e)}"

if __name__ == "__main__":
    mcp.run()
```

## Quality Checklist

### Tool Configuration
- [ ] All tools implement `name` and `annotations` in the decorator
- [ ] Annotations correctly set (readOnlyHint, destructiveHint, idempotentHint, openWorldHint)
- [ ] All tools use Pydantic BaseModel for input validation with Field() definitions
- [ ] All Pydantic Fields have explicit types and descriptions with constraints
- [ ] All tools have comprehensive docstrings

### Code Quality
- [ ] Pagination properly implemented where applicable
- [ ] Common functionality extracted into reusable functions
- [ ] All async functions properly defined with `async def`
- [ ] Error handling uses specific exception types
- [ ] Server runs successfully: `python your_server.py`
