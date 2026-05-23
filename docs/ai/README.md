# Hastebin AI Integration Guide

Comprehensive guide for AI assistants to interact with Hastebin in conversational contexts.

## Overview

Hastebin is a modern pastebin service where users share code snippets. As an AI assistant, you can:
- Fetch and analyze code from URLs users share
- Create new pastes to share code back
- Fork pastes with improvements
- Add inline comments for code review

## Quick Start

### When a User Shares a URL

```
User: "Can you help with this code? https://hastebin.ianhon.com/abc123"
```

**Steps:**
1. Extract hex ID: `abc123`
2. Convert to decimal: `parseInt("abc123", 16)` = `11259375`
3. Fetch paste: `GET /paste/fetch/11259375`
4. Analyze and respond

### API Endpoints

- **Base URL**: `https://backend.ianhon.com/hastebin`
- **Frontend**: `https://hastebin.ianhon.com`
- **OpenAPI Spec**: `/openapi.json`

## ID Conversion (Critical!)

The frontend uses **hexadecimal** IDs, but the API expects **decimal integers**.

**From URL to API:**
```javascript
// URL: https://hastebin.ianhon.com/abc123
const hexId = "abc123";
const decimalId = parseInt(hexId, 16);  // 11259375
// Use decimalId in API calls
```

```python
hex_id = "abc123"
decimal_id = int(hex_id, 16)  # 11259375
```

**From API to URL:**
```javascript
// API returns: {"id": 1234}
const hexId = (1234).toString(16);  // "4d2"
const shareUrl = `https://hastebin.ianhon.com/${hexId}`;
```

```python
decimal_id = 1234
hex_id = hex(decimal_id)[2:]  # "4d2"
share_url = f"https://hastebin.ianhon.com/{hex_id}"
```

## Common Operations

### 1. Fetch a Paste

```http
GET /paste/fetch/{id}
```

**Response:**
```json
{
  "paste": {
    "id": 11259375,
    "content": "console.log('Hello');",
    "title": "My Code",
    "author": "john_doe",
    "views": 42,
    "comments_enabled": true,
    "created_at": 1716835200,
    "expires_at": null,
    "forked_from": null
  },
  "checksum_pair": null
}
```

### 2. Create a Paste

```http
POST /paste/create
Content-Type: application/json

{
  "content": "console.log('Fixed version');",
  "title": "Improved Code",
  "author": "AI Assistant",
  "comments_enabled": true,
  "expires_at": null
}
```

**Response:** `{"id": 1234}`

**Don't forget to convert to hex when sharing!**

### 3. Fork a Paste

```http
POST /paste/create
Content-Type: application/json

{
  "content": "improved code here",
  "title": "Refactored Version",
  "forked_from": 11259375,
  "comments_enabled": true
}
```

### 4. Add a Comment

```http
POST /comment/create
Content-Type: application/json

{
  "paste_id": 11259375,
  "content": "Consider using async/await here",
  "author": "AI Reviewer",
  "page_index": 0,
  "from_row": 10,
  "from_column": 0,
  "to_row": 12,
  "to_column": 50
}
```

### 5. Get All Comments

```http
GET /comment/paste/{id}
```

Returns array of comments with their text selections.

## Conversational Workflows

### Scenario 1: Code Explanation

```
User: "Explain this: https://hastebin.ianhon.com/abc123"
```

1. Extract ID → convert to decimal
2. Fetch paste content
3. Analyze and explain

### Scenario 2: Code Improvement

```
User: "Can you improve this? https://hastebin.ianhon.com/def456"
```

1. Fetch original paste
2. Improve the code
3. Create a fork (use `forked_from`)
4. Convert response ID to hex
5. Share new URL with user

### Scenario 3: Code Review

```
User: "Review this: https://hastebin.ianhon.com/review789"
```

1. Fetch paste
2. Check if `comments_enabled` is true
3. Add comments on specific lines
4. Tell user you've added inline feedback

## Important Concepts

### Multi-File Pastes

Content can be a JSON string containing an array:

```json
{
  "content": "[{\"fileName\": \"index.js\", \"content\": \"...\"}, {\"fileName\": \"utils.js\", \"content\": \"...\"}]",
  "title": "Multi-file Project"
}
```

### Timestamps

All timestamps are **Unix epoch seconds** (not milliseconds).

Example: `1716835200` represents May 28, 2024, 00:00:00 UTC.

### Expiration

- `expires_at`: Unix epoch seconds (optional)
- If `null` or omitted, paste never expires
- Expired pastes are auto-deleted when accessed

### Comments

- Only work if `comments_enabled: true`
- Attached to text selections via row/column coordinates
- `page_index`: file index in multi-file pastes (0-based)
- Row and column numbers are 0-based

### Cryptographic Signatures

- Use `checksum_passphrase` when creating to sign a paste
- API returns `checksum_pair`: `[partial, full]` (SHA256 hex strings)
- Verification: `sha256(passphrase + partial) = full`
- Passphrase is never stored or returned

## Best Practices

1. **Always convert IDs** between hex and decimal correctly
2. **Enable comments** (`comments_enabled: true`) for collaboration
3. **Use descriptive titles** to help users identify pastes
4. **Set forked_from** when creating improved versions
5. **Check comments_enabled** before trying to add comments
6. **Use appropriate expiration times** for temporary pastes

## URL Extraction Tips

Users might share URLs in various formats:
- `https://hastebin.ianhon.com/abc123`
- `https://hastebin.ianhon.com/abc123.js`
- `hastebin.ianhon.com/abc123`

Extract the hex ID (remove domain, protocol, and file extensions).

## Example Implementation

See [example.py](./example.py) for a complete Python reference implementation with all common scenarios.

## Full API Documentation

For complete endpoint details, schemas, and examples, see the OpenAPI specification:
```http
GET https://backend.ianhon.com/hastebin/openapi.json
```
