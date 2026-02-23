# Node/TypeScript MCP Server Implementation Guide

## Overview

This document provides Node/TypeScript-specific best practices and examples for implementing MCP servers using the MCP TypeScript SDK. It covers project structure, server setup, tool registration patterns, input validation with Zod, error handling, and complete working examples.

---

## Quick Reference

### Key Imports
```typescript
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import express from "express";
import { z } from "zod";
```

### Server Initialization
```typescript
const server = new McpServer({
  name: "service-mcp-server",
  version: "1.0.0"
});
```

### Tool Registration Pattern
```typescript
server.registerTool(
  "tool_name",
  {
    title: "Tool Display Name",
    description: "What the tool does",
    inputSchema: { param: z.string() },
    outputSchema: { result: z.string() }
  },
  async ({ param }) => {
    const output = { result: `Processed: ${param}` };
    return {
      content: [{ type: "text", text: JSON.stringify(output) }],
      structuredContent: output
    };
  }
);
```

---

## MCP TypeScript SDK

**IMPORTANT - Use Modern APIs Only:**
- **DO use**: `server.registerTool()`, `server.registerResource()`, `server.registerPrompt()`
- **DO NOT use**: Old deprecated APIs such as `server.tool()`, `server.setRequestHandler(ListToolsRequestSchema, ...)`, or manual handler registration

## Server Naming Convention

Format: `{service}-mcp-server` (lowercase with hyphens)
- Examples: `github-mcp-server`, `jira-mcp-server`, `stripe-mcp-server`

## Project Structure

```
{service}-mcp-server/
├── package.json
├── tsconfig.json
├── README.md
├── src/
│   ├── index.ts          # Main entry point with McpServer initialization
│   ├── types.ts          # TypeScript type definitions and interfaces
│   ├── tools/            # Tool implementations (one file per domain)
│   ├── services/         # API clients and shared utilities
│   ├── schemas/          # Zod validation schemas
│   └── constants.ts      # Shared constants (API_URL, CHARACTER_LIMIT, etc.)
└── dist/                 # Built JavaScript files (entry point: dist/index.js)
```

## Tool Implementation

### Tool Naming

Use snake_case for tool names with service prefix to prevent conflicts:
- Use `slack_send_message` instead of just `send_message`
- Use `github_create_issue` instead of just `create_issue`

### Tool Structure

```typescript
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";

const server = new McpServer({
  name: "example-mcp",
  version: "1.0.0"
});

const UserSearchInputSchema = z.object({
  query: z.string()
    .min(2, "Query must be at least 2 characters")
    .max(200, "Query must not exceed 200 characters")
    .describe("Search string to match against names/emails"),
  limit: z.number()
    .int()
    .min(1)
    .max(100)
    .default(20)
    .describe("Maximum results to return"),
  offset: z.number()
    .int()
    .min(0)
    .default(0)
    .describe("Number of results to skip for pagination"),
}).strict();

server.registerTool(
  "example_search_users",
  {
    title: "Search Example Users",
    description: `Search for users in the Example system by name, email, or team.

Args:
  - query (string): Search string to match against names/emails
  - limit (number): Maximum results to return, between 1-100 (default: 20)
  - offset (number): Number of results to skip for pagination (default: 0)

Returns:
  {
    "total": number,
    "count": number,
    "offset": number,
    "users": [{ "id": string, "name": string, "email": string }],
    "has_more": boolean,
    "next_offset": number
  }`,
    inputSchema: UserSearchInputSchema,
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const data = await makeApiRequest("users/search", "GET", undefined, {
        q: params.query,
        limit: params.limit,
        offset: params.offset
      });

      const output = {
        total: data.total || 0,
        count: data.users?.length || 0,
        offset: params.offset,
        users: data.users || [],
        has_more: data.total > params.offset + (data.users?.length || 0),
        next_offset: data.total > params.offset + (data.users?.length || 0)
          ? params.offset + data.users.length
          : undefined
      };

      return {
        content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        structuredContent: output
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: handleApiError(error) }]
      };
    }
  }
);
```

## Zod Schemas for Input Validation

```typescript
import { z } from "zod";

const CreateUserSchema = z.object({
  name: z.string().min(1).max(100),
  email: z.string().email("Invalid email format"),
  age: z.number().int().min(0).max(150)
}).strict();

enum ResponseFormat {
  MARKDOWN = "markdown",
  JSON = "json"
}

const SearchSchema = z.object({
  response_format: z.nativeEnum(ResponseFormat)
    .default(ResponseFormat.MARKDOWN)
    .describe("Output format: 'markdown' for human-readable or 'json' for machine-readable")
});
```

## Character Limits and Truncation

```typescript
export const CHARACTER_LIMIT = 25000;

async function searchTool(params: SearchInput) {
  let result = generateResponse(data);

  if (result.length > CHARACTER_LIMIT) {
    const truncatedData = data.slice(0, Math.max(1, data.length / 2));
    response.data = truncatedData;
    response.truncated = true;
    response.truncation_message =
      `Response truncated from ${data.length} to ${truncatedData.length} items. ` +
      `Use 'offset' parameter or add filters to see more results.`;
    result = JSON.stringify(response, null, 2);
  }

  return result;
}
```

## Error Handling

```typescript
import axios, { AxiosError } from "axios";

function handleApiError(error: unknown): string {
  if (error instanceof AxiosError) {
    if (error.response) {
      switch (error.response.status) {
        case 404: return "Error: Resource not found. Please check the ID is correct.";
        case 403: return "Error: Permission denied. You don't have access to this resource.";
        case 429: return "Error: Rate limit exceeded. Please wait before making more requests.";
        default: return `Error: API request failed with status ${error.response.status}`;
      }
    } else if (error.code === "ECONNABORTED") {
      return "Error: Request timed out. Please try again.";
    }
  }
  return `Error: Unexpected error occurred: ${error instanceof Error ? error.message : String(error)}`;
}
```

## Package Configuration

### package.json
```json
{
  "name": "{service}-mcp-server",
  "version": "1.0.0",
  "type": "module",
  "main": "dist/index.js",
  "scripts": {
    "start": "node dist/index.js",
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "clean": "rm -rf dist"
  },
  "engines": { "node": ">=18" },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.6.1",
    "axios": "^1.7.9",
    "zod": "^3.23.8"
  },
  "devDependencies": {
    "@types/node": "^22.10.0",
    "tsx": "^4.19.2",
    "typescript": "^5.7.2"
  }
}
```

### tsconfig.json
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "Node16",
    "moduleResolution": "Node16",
    "lib": ["ES2022"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

## Transport Options

### stdio (For Local Integrations)
```typescript
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

async function runStdio() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("MCP server running via stdio");
}
```

### Streamable HTTP (For Remote Servers)
```typescript
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import express from "express";

async function runHTTP() {
  const app = express();
  app.use(express.json());

  app.post('/mcp', async (req, res) => {
    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true
    });
    res.on('close', () => transport.close());
    await server.connect(transport);
    await transport.handleRequest(req, res, req.body);
  });

  const port = parseInt(process.env.PORT || '3000');
  app.listen(port, () => {
    console.error(`MCP server running on http://localhost:${port}/mcp`);
  });
}

const transport = process.env.TRANSPORT || 'stdio';
if (transport === 'http') {
  runHTTP().catch(console.error);
} else {
  runStdio().catch(console.error);
}
```

## Quality Checklist

### Implementation Quality
- [ ] All tools registered using `registerTool` with complete configuration
- [ ] All tools include `title`, `description`, `inputSchema`, and `annotations`
- [ ] Annotations correctly set (readOnlyHint, destructiveHint, idempotentHint, openWorldHint)
- [ ] All tools use Zod schemas for runtime input validation with `.strict()`
- [ ] Error messages are clear, actionable, and educational

### TypeScript Quality
- [ ] TypeScript interfaces defined for all data structures
- [ ] Strict TypeScript enabled in tsconfig.json
- [ ] No use of `any` type
- [ ] All async functions have explicit return types
- [ ] Error handling uses proper type guards

### Build and Test
- [ ] `npm run build` completes successfully without errors
- [ ] `dist/index.js` created and executable
- [ ] All imports resolve correctly
