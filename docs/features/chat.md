# Chat Feature

The chat feature allows you to interact with your Monday.com boards using natural language through the Model Context Protocol (MCP) server.

## Overview

The chat feature leverages the Monday.com MCP server to provide an interactive conversational interface for querying and managing your board data. You can ask questions about entries, request summaries, and get insights about your time tracking data.

## Configuration

The chat feature requires MCP server configuration in `.bob/mcp.json`:

```json
{
  "mcpServers": {
    "monday": {
      "type": "sse",
      "url": "https://mcp.monday.com/sse",
      "headers": {
        "Authorization": "YOUR_MONDAY_API_TOKEN"
      }
    }
  }
}
```

## Usage

### Basic Chat

Start a chat session with the default board (6500270039):

```bash
claim chat
```

### Chat with Specific Board

Specify a different board ID:

```bash
claim chat --board 1234567890
```

or using the short form:

```bash
claim chat -b 1234567890
```

### Verbose Mode

Enable verbose output for debugging:

```bash
claim chat --verbose
```

## Chat Interface

Once you start a chat session, you'll see:

```
╔════════════════════════════════════════════════════════════╗
║          Monday.com Board Chat (MCP-powered)              ║
╚════════════════════════════════════════════════════════════╝

Board ID: 6500270039

You can ask questions about the board, entries, or request actions.
Type 'exit' or 'quit' to end the chat session.

You: 
```

## Example Conversations

### Query Recent Entries

```
You: Show me my entries from last week
Assistant: Here are your entries from last week:
- 2024-03-04: Customer A, Project X - 8 hours
- 2024-03-05: Customer B, Project Y - 6 hours
...
```

### Get Summaries

```
You: How many hours did I work on Customer A this month?
Assistant: You worked a total of 42 hours on Customer A this month across 3 different projects.
```

### Ask About Specific Dates

```
You: What did I work on yesterday?
Assistant: Yesterday (2024-03-10), you worked on:
- Customer C, Project Z - 4 hours (billable)
- Internal training - 2 hours (education)
```

## Commands

Within the chat session:

- Type your question or request naturally
- Type `exit` or `quit` to end the session
- Press Enter on an empty line to skip

## Features

- **Natural Language Processing**: Ask questions in plain English
- **Board Context**: The assistant understands your specific board structure
- **Real-time Responses**: Get immediate answers through the MCP SSE connection
- **Error Handling**: Clear error messages if something goes wrong

## Technical Details

### MCP Integration

The chat feature uses:
- **SSE (Server-Sent Events)**: For real-time streaming responses
- **Monday.com MCP Server**: Official MCP server for Monday.com integration
- **Authorization**: Uses the same API token from your MCP configuration

### Message Flow

1. User enters a question
2. Question is sent to MCP server with board context
3. MCP server processes the request using Monday.com API
4. Response is streamed back via SSE
5. Assistant message is displayed to user

## Troubleshooting

### "MCP configuration not found"

Ensure `.bob/mcp.json` exists in your project root with proper configuration.

### "Authorization token not found"

Check that your `.bob/mcp.json` includes the Authorization header with a valid Monday.com API token.

### Connection Timeout

If requests timeout:
- Check your internet connection
- Verify the MCP server URL is correct
- Ensure your API token is valid and not expired

### No Response

If you don't get a response:
- Try running with `--verbose` flag for debugging
- Check that the board ID is correct
- Verify your API token has access to the specified board

## Limitations

- Requires active internet connection
- Depends on Monday.com MCP server availability
- Response quality depends on the MCP server's capabilities
- Currently supports read operations; write operations may be limited

## Future Enhancements

Planned improvements:
- [ ] Support for multiple boards in one session
- [ ] Command history and recall
- [ ] Export chat transcripts
- [ ] Integration with interactive UI mode
- [ ] Custom prompts and templates
- [ ] Batch operations through chat

## See Also

- [Query Feature](../features/query.md)
- [Interactive UI](../features/interactive-ui.md)
- [Configuration Guide](../README.md)