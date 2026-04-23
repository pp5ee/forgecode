# ForgeCode Web Gateway Implementation Plan

## Goal Description
Add a web UI gateway to ForgeCode that provides remote access to the AI development environment through a web interface with URL token authentication, similar to OpenClaw Gateway. The gateway will enable users to deploy ForgeCode in Docker containers and access it from other computers via web browser, supporting complete interactive conversations, command execution, file browsing, and editing.

## Acceptance Criteria

Following TDD philosophy, each criterion includes positive and negative tests for deterministic verification.

- AC-1: Web Gateway Server Initialization
  - Positive Tests (expected to PASS):
    - Server starts successfully on specified port (default 8080)
    - Server responds to health check endpoint with 200 OK
    - Server logs startup information and port binding
  - Negative Tests (expected to FAIL):
    - Server fails to start when port is already in use
    - Server crashes on invalid configuration
    - Server rejects requests without proper authentication
  - AC-1.1: Configuration Management
    - Positive: Server loads configuration from environment variables and config files
    - Negative: Server fails to start with invalid or missing required configuration

- AC-2: URL Token Authentication
  - Positive Tests: 
    - Requests with valid token in URL parameter are authenticated
    - Token validation succeeds with correct token format
    - Authentication tokens can be generated and managed via API
  - Negative Tests:
    - Requests without token parameter are rejected with 401
    - Requests with invalid/expired tokens are rejected with 403
    - Token brute force attempts are rate-limited

- AC-3: Web UI Interface
  - Positive Tests:
    - Web interface loads successfully in modern browsers
    - UI provides interactive conversation interface
    - UI supports file browsing and editing capabilities
    - UI includes command execution interface
  - Negative Tests:
    - UI fails gracefully when backend services are unavailable
    - Invalid file paths or operations show appropriate error messages
    - UI handles large file trees without performance degradation

- AC-4: Interactive Conversation Support
  - Positive Tests:
    - Users can initiate and maintain conversations through web interface
    - Conversation history is preserved across sessions
    - Real-time message streaming works correctly
    - Multiple conversation contexts are supported
  - Negative Tests:
    - Conversations with invalid parameters are rejected
    - Very large conversation payloads are handled gracefully
    - Concurrent conversations do not interfere with each other

- AC-5: File System Operations (Exposing Existing API)
  - Positive Tests:
    - Users can browse project directory structure via existing `forge_api::API::discover()`
    - File content can be viewed and edited through workspace sync/query operations
    - File operations proxy to existing `forge_api` functionality
    - Path traversal protection is enforced at gateway boundary
  - Negative Tests:
    - Attempts to access restricted paths are blocked with appropriate errors
    - Invalid file operations return proper error responses
    - Large file operations are handled with existing progress mechanisms

- AC-6: Command Execution (Exposing Existing API)
  - Positive Tests:
    - Users can execute shell commands through existing `forge_api::API::execute_shell_command()`
    - Command output is streamed using existing real-time mechanisms
    - Multiple commands can be executed through existing concurrent handling
    - Command execution respects existing security policies
  - Negative Tests:
    - Dangerous commands are blocked by existing validation mechanisms
    - Commands with very long output are handled by existing streaming
    - Failed commands return proper error responses from existing API

- AC-7: Docker Deployment Support
  - Positive Tests:
    - Gateway can be deployed as Docker container
    - Container configuration includes all required dependencies
    - Environment variables are properly passed to container
    - Container health checks work correctly
  - Negative Tests:
    - Container fails to start with missing required environment variables
    - Invalid Docker configuration is detected and reported
    - Resource constraints are respected and handled gracefully

## Path Boundaries

### Upper Bound (Maximum Acceptable Scope)
The implementation includes a full-featured web gateway with real-time WebSocket communication, comprehensive file management, advanced conversation features, user management system, plugin architecture, and extensive configuration options. The gateway supports multiple authentication methods, advanced security features, and integrates deeply with all ForgeCode capabilities.

### Lower Bound (Minimum Acceptable Scope)
The implementation includes a basic HTTP server with URL token authentication, simple web interface for conversation interaction, basic file browsing, and command execution. The gateway runs in a single Docker container with minimal configuration and provides core functionality for remote access to ForgeCode.

### Allowed Choices
- Can use: Actix-web or Warp for HTTP server, Tokio for async runtime, WebSocket for real-time communication, HTML/CSS/JavaScript for frontend
- Cannot use: External authentication services, complex frontend frameworks that significantly increase bundle size, proprietary technologies that limit deployment options

## Feasibility Hints and Suggestions

> **Note**: This section is for reference and understanding only. These are conceptual suggestions, not prescriptive requirements.

### Conceptual Approach
1. Create a new `forge_gateway` crate that wraps the existing ForgeCode API
2. Implement HTTP server with URL token authentication middleware
3. Build web interface using modern HTML/CSS/JavaScript
4. Add WebSocket support for real-time conversation streaming
5. Create Docker configuration for container deployment

### Relevant References
- `crates/forge_main/src/main.rs` - Main application entry point
- `crates/forge_services/` - Core service layer
- `crates/forge_api/` - API layer for integration
- `crates/forge_config/` - Configuration management

## Dependencies and Sequence

### Milestones
1. **Milestone 1: Core Gateway Infrastructure**
   - Phase A: Create gateway crate and basic HTTP server
   - Phase B: Implement URL token authentication system
   - Phase C: Add health check and basic error handling

2. **Milestone 2: Web Interface Development**
   - Phase A: Build conversation interface components
   - Phase B: Implement file browsing and editing features
   - Phase C: Add command execution interface

3. **Milestone 3: Integration and Deployment**
   - Phase A: Integrate with existing ForgeCode services
   - Phase B: Create Docker deployment configuration
   - Phase C: Add monitoring and logging

## Task Breakdown

Each task must include exactly one routing tag:
- `coding`: implemented by Claude
- `analyze`: executed via Codex (`/humanize:ask-codex`)

| Task ID | Description | Target AC | Tag (`coding`/`analyze`) | Depends On |
|---------|-------------|-----------|----------------------------|------------|
| task1 | Create new forge_gateway crate structure | AC-1 | coding | - |
| task2 | Implement HTTP server with Actix-web | AC-1 | coding | task1 |
| task3 | Integrate with ForgeCode API services | AC-4,5,6 | coding | task2 |
| task4 | Design URL token authentication system | AC-2 | analyze | task3 |
| task5 | Implement token validation middleware | AC-2 | coding | task4 |
| task6 | Create web interface HTML structure | AC-3 | coding | task3 |
| task7 | Implement conversation interface | AC-4 | coding | task6 |
| task8 | Add file browsing functionality | AC-5 | coding | task6 |
| task9 | Implement command execution interface | AC-6 | coding | task6 |
| task10 | Add WebSocket support for real-time updates | AC-4,6 | coding | task3 |
| task11 | Create Docker deployment configuration | AC-7 | coding | task2 |
| task12 | Implement security and rate limiting | AC-2 | coding | task5 |
| task13 | Create comprehensive test suite | All AC | coding | task10 |
| task14 | Add monitoring and logging | AC-1 | coding | task13 |

## Design Decisions

### Architecture Constraints
- Gateway implemented as `forge_gateway` crate following existing naming convention
- Uses `Arc<T>` for infrastructure dependencies (per AGENTS.md Service Implementation Guidelines)
- Takes at most one generic type parameter
- Avoids `Box<dyn ...>` trait objects
- Uses constructor pattern without type bounds on `new()`

### Technology Choices
- **Frontend**: Vanilla JavaScript (minimal dependencies, aligns with codebase philosophy)
- **Real-time Communication**: WebSocket with SSE fallback consideration
- **HTTP Server**: Actix-web or Warp (leverage existing Tokio async runtime)
- **Streaming**: Consider `forge_stream::MpscStream` for real-time message streaming

### Integration Strategy
- Gateway exposes existing `forge_api` functionality rather than reimplementing
- File operations proxy to `forge_api::API::discover()` and workspace operations
- Command execution uses existing `forge_api::API::execute_shell_command()`
- Conversation management leverages existing conversation services

### Test Requirements
- All tests follow three-step pattern: setup → actual → expected
- Uses `pretty_assertions` for test assertions
- Comprehensive test coverage for all acceptance criteria

## Pending User Decisions

- DEC-1: Default port configuration
  - **Recommendation**: Port 8080 default with environment variable override
  - **Justification**: 8080 is standard for production services, while allowing flexibility for development
  - **Decision Status**: `PENDING`

- DEC-2: Token expiration policy
  - **Recommendation**: Configurable expiration with 24-hour default, allowing "never expire" for trusted internal networks
  - **Justification**: Balances security needs with convenience for development environments
  - **Decision Status**: `PENDING`

## Implementation Notes

### Code Style Requirements
- Implementation code and comments must NOT contain plan-specific terminology such as "AC-", "Milestone", "Step", "Phase", or similar workflow markers
- These terms are for plan documentation only, not for the resulting codebase
- Use descriptive, domain-appropriate naming in code instead

### Security Considerations
- All file operations must validate paths to prevent directory traversal attacks
- Command execution must be sandboxed and validated
- Token generation must use cryptographically secure random number generation

## Output File Convention

This template is used to produce the main output file (e.g., `plan.md`).

### Translated Language Variant

When `alternative_plan_language` resolves to a supported language name through merged config loading, a translated variant of the output file is also written after the main file. Humanize loads config from merged layers in this order: default config, optional user config, then optional project config; `alternative_plan_language` may be set at any of those layers. The variant filename is constructed by inserting `_<code>` (the ISO 639-1 code from the built-in mapping table) immediately before the file extension:

- `plan.md` becomes `plan_<code>.md` (e.g. `plan_zh.md` for Chinese, `plan_ko.md` for Korean)
- `docs/my-plan.md` becomes `docs/my-plan_<code>.md`
- `output` (no extension) becomes `output_<code>`

The translated variant file contains a full translation of the main plan file's current content in the configured language. All identifiers (`AC-*`, task IDs, file paths, API names, command flags) remain unchanged, as they are language-neutral.

When `alternative_plan_language` is empty, absent, set to `"English"`, or set to an unsupported language, no translated variant is written. Humanize does not auto-create `.humanize/config.json` when no project config file is present.

--- Original Design Draft Start ---

# Requirement

帮我给这个项目加一个gateway，类似openclaw的gateway，使用url中的token鉴权，是一个web ui的，我们把opencode 部署在内网的容器中后可以在另外一台电脑上操作和使用forgecode 本身进行编程，我需要这样一个web 页面

## Clarification Q&A



### Q1

现在我理解了你的需求！你想要为 ForgeCode 项目添加一个类似 OpenClaw Gateway 的 Web UI 网关，具有以下特性：
## 需求总结
1. **Web UI 网关** - 类似 OpenClaw Gateway 的 Web 界面
2. **URL Token 鉴权** - 通过 URL 参数中的 token 进行身份验证
3. **内网容器部署** - 将 OpenCode 部署在内网容器环境中
4. **远程访问** - 允许其他电脑通过 Web 界面操作 ForgeCode
## 技术方案建议
基于现有代码库结构，我建议采用以下架构：
- **前端**：使用现代 Web 技术（HTML/CSS/JavaScript）构建 Web UI
- **后端**：基于现有的 Rust 代码库，添加 HTTP 服务器层
- **鉴权**：URL token 参数验证机制
- **通信**：WebSocket 或 SSE 用于实时交互
## 需要澄清的关键点
为了确保实现符合你的期望，我需要确认几个细节：
1. **Web UI 功能范围**：你希望 Web UI 支持哪些 ForgeCode 功能？
   - 完整的交互式对话
   - 特定的命令执行
   - 文件浏览和编辑
   - 其他功能？
2. **部署环境**：你计划使用哪种容器技术？
   - Docker
   - Kubernetes
   - 其他容器编排平台？
请提供这些细节，然后我可以为你制定详细的实现计划。



### A1

完整的交互式对话

特定的命令执行

文件浏览和编辑 命令行执行；docker 部署

---

## Implementation Notes

- For any unspecified details (combat formulas, game balance, UI layout, tech choices, etc.), make reasonable decisions yourself and document them in the plan. Do NOT ask the user for clarification — proceed with sensible defaults.
- If referenced image files exist in the workspace, treat them as visual style references.

## Standard Deliverables (mandatory for every project)

- **README.md** — must be included at the project root with: project title & description, prerequisites, installation steps, usage examples with code snippets, configuration options, and project structure overview.
- **Git commits** — use conventional commit prefix `feat:` for all commits.

--- Original Design Draft End ---
