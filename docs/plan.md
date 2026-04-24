# ForgeCode Web Gateway Implementation Plan

## Goal Description
Build a web UI gateway for the forgecode project that provides remote access to forgecode programming capabilities through a browser-based interface. The gateway will use URL token authentication, support real-time terminal interaction, and deploy as a single container with multiple modules (gateway + forgecode).

## Acceptance Criteria

Following TDD philosophy, each criterion includes positive and negative tests for deterministic verification.

- AC-1: Gateway serves web UI with URL token authentication
  - Positive Tests (expected to PASS):
    - Accessing gateway with valid token redirects to web interface
    - Invalid tokens are rejected with appropriate HTTP status
    - Token validation works for various token formats (alphanumeric, UUID)
  - Negative Tests (expected to FAIL):
    - Access without token returns authentication error
    - Expired tokens are rejected
    - Malformed tokens are rejected
  - AC-1.1: Token renewal mechanism after initial login
    - Positive: Users can renew tokens within web interface
    - Negative: Renewal requires valid session

- AC-2: Web UI provides full forgecode terminal functionality
  - Positive Tests:
    - Terminal emulator loads and displays forgecode prompt
    - Commands execute and return output in real-time
    - Terminal supports basic operations (clear, resize, copy/paste)
  - Negative Tests:
    - Terminal rejects invalid commands appropriately
    - Connection errors are handled gracefully
    - Terminal state is preserved across refreshes

- AC-3: Real-time communication between web UI and forgecode backend
  - Positive Tests:
    - WebSocket connection establishes successfully
    - Input/output streams work bidirectionally
    - Multiple concurrent sessions are isolated
  - Negative Tests:
    - WebSocket failures trigger reconnection attempts
    - Invalid messages are rejected
    - Connection timeouts are handled gracefully

- AC-4: Single-container deployment with module isolation
  - Positive Tests:
    - Container starts with both gateway and forgecode modules
    - Modules communicate via IPC mechanisms
    - Health checks report correct status for both modules
  - Negative Tests:
    - Module failures don't crash entire container
    - Resource conflicts are prevented
    - Startup sequencing works correctly

- AC-5: Security features and rate limiting
  - Positive Tests:
    - Security headers are present in HTTP responses
    - Rate limiting prevents brute force attacks
    - CORS is configured appropriately
  - Negative Tests:
    - Excessive requests are rate-limited
    - Invalid authentication attempts are logged
    - Session data is properly isolated

## Path Boundaries

Path boundaries define the acceptable range of implementation quality and choices.

### Upper Bound (Maximum Acceptable Scope)
The implementation includes a fully-featured web gateway with Axum web framework, xterm.js terminal emulator, WebSocket-based real-time communication, comprehensive token management with renewal, file upload/download capabilities, multi-user session support, health monitoring endpoints, and production-ready security features including rate limiting and comprehensive logging.

### Lower Bound (Minimum Acceptable Scope)
The implementation includes a basic web gateway with Axum web framework, simple terminal interface using basic HTML/JavaScript, token authentication via URL parameters, single-user session support, and basic error handling. This provides core functionality while meeting all acceptance criteria.

### Allowed Choices
- Can use: Axum web framework (recommended), Warp, or Actix-web; xterm.js or similar terminal emulators; WebSocket or Server-Sent Events for real-time communication; Docker for containerization
- Cannot use: Frontend frameworks that require complex build tooling (React, Vue, etc.); external databases for token storage (use file-based storage initially); third-party authentication providers (use built-in token system)

## Feasibility Hints and Suggestions

> **Note**: This section is for reference and understanding only. These are conceptual suggestions, not prescriptive requirements.

### Conceptual Approach
Create a new crate `forge_gateway` in the workspace that:
1. Uses Axum web framework (aligns with existing Tokio runtime)
2. Implements HTTP server with URL token authentication middleware
3. Serves static HTML/JS files for web UI
4. Integrates xterm.js for terminal emulation in browser
5. Uses WebSocket for real-time communication with forgecode backend
6. Runs forgecode process as subprocess with PTY management
7. Handles token generation/rotation via simple file-based storage
8. Deploys as single Docker container with multiple processes

### Relevant References
- `crates/forge_main/src/main.rs` - Main entry point and CLI handling
- `crates/forge_services/src/` - Service layer patterns and authentication
- `crates/forge_config/src/` - Configuration management patterns
- Existing Cargo.toml dependencies - Reuse existing Tokio, async patterns
- AGENTS.md - For coding conventions and project guidelines

## Dependencies and Sequence

### Milestones
1. Milestone 1: Gateway foundation and authentication
   - Phase A: Create forge_gateway crate with Axum web server
   - Phase B: Implement URL token authentication middleware
   - Phase C: Add static file serving for web UI
2. Milestone 2: Terminal integration and real-time communication
   - Phase A: Integrate xterm.js terminal emulator
   - Phase B: Implement WebSocket connection to forgecode backend
   - Phase C: Add PTY management for forgecode subprocess
3. Milestone 3: Token management and security features
   - Phase A: Implement token generation and rotation
   - Phase B: Add rate limiting and security headers
   - Phase C: Implement session management and isolation
4. Milestone 4: Container deployment and operational features
   - Phase A: Create Dockerfile for single-container deployment
   - Phase B: Add health check endpoints and logging
   - Phase C: Implement file upload/download capabilities

Dependencies: Milestone 2 depends on Milestone 1 completion. Milestone 3 depends on Milestone 2. Milestone 4 depends on all previous milestones.

## Task Breakdown

Each task must include exactly one routing tag:
- `coding`: implemented by Claude
- `analyze`: executed via Codex (`/humanize:ask-codex`)

| Task ID | Description | Target AC | Tag (`coding`/`analyze`) | Depends On |
|---------|-------------|-----------|----------------------------|------------|
| task1 | Create forge_gateway crate with basic Axum server | AC-1 | coding | - |
| task2 | Implement URL token authentication middleware | AC-1 | coding | task1 |
| task3 | Add static file serving for web UI assets | AC-1 | coding | task1 |
| task4 | Design web UI with xterm.js terminal interface | AC-2 | coding | task1 |
| task5 | Implement WebSocket server for real-time communication | AC-3 | coding | task2, task4 |
| task6 | Integrate forgecode as subprocess with PTY management | AC-2 | coding | task5 |
| task7 | Implement token generation and rotation system | AC-1.1 | coding | task2 |
| task8 | Add rate limiting and security headers | AC-5 | coding | task2 |
| task9 | Create Dockerfile for single-container deployment | AC-4 | coding | task1, task6 |
| task10 | Implement health check endpoints | AC-4 | coding | task1 |
| task11 | Add comprehensive logging and error handling | AC-5 | coding | task1 |
| task12 | Implement file upload/download capabilities | - | analyze | task6 |
| task13 | Add multi-user session support | - | analyze | task7 |
| task14 | Create documentation and usage examples | - | analyze | task9 |

## Claude-Codex Deliberation

### Agreements
- Axum web framework is the optimal choice (aligns with existing Tokio runtime)
- WebSocket-based real-time communication is necessary for terminal interaction
- Single-container deployment with process isolation is appropriate
- URL token authentication with renewal mechanism meets requirements

### Resolved Disagreements
- **Framework choice**: Codex suggested Axum/Warp/Actix-web options, Claude selected Axum for better integration with existing Tokio ecosystem
- **Terminal emulation**: Codex raised concerns about complexity, Claude confirmed xterm.js is standard and feasible
- **Deployment model**: Codex suggested sidecar containers, Claude selected single-container with multiple processes as specified in requirements

### Convergence Status
- Final Status: `partially_converged` (due to direct mode without full convergence rounds)

## Pending User Decisions

- DEC-1: File upload/download capability scope
  - Claude Position: Implement basic file transfer for web terminal usage
  - Codex Position: Consider comprehensive file management system
  - Tradeoff Summary: Basic functionality vs. comprehensive feature set
  - Decision Status: `PENDING`

- DEC-2: Multi-user support requirements
  - Claude Position: Single-user initially with session isolation
  - Codex Position: Consider multi-user architecture from start
  - Tradeoff Summary: Simplicity vs. scalability
  - Decision Status: `PENDING`

- DEC-3: Token storage mechanism
  - Claude Position: File-based storage for simplicity
  - Codex Position: Consider database for production scaling
  - Tradeoff Summary: Development speed vs. production readiness
  - Decision Status: `PENDING`

## Implementation Notes

### Code Style Requirements
- Implementation code and comments must NOT contain plan-specific terminology such as "AC-", "Milestone", "Step", "Phase", or similar workflow markers
- These terms are for plan documentation only, not for the resulting codebase
- Use descriptive, domain-appropriate naming in code instead

### Draft Completeness Requirement
This plan incorporates ALL information from the input draft document. The draft specifies:
- Web gateway similar to OpenClaw Gateway
- URL token authentication with renewal capability
- Web UI interface for remote forgecode access
- Single-container deployment with multiple modules
- Mature and simple web framework (Axum selected)

### Technical Decisions Made
Based on the draft and project analysis, the following technical decisions have been made:
- Web framework: Axum (mature, simple, aligns with existing Tokio runtime)
- Terminal emulation: xterm.js (industry standard for web terminals)
- Real-time communication: WebSocket (bidirectional, low-latency)
- Containerization: Single Docker container with multiple processes
- Token storage: File-based initially (simple, meets requirements)

### Security Considerations
- URL tokens will be stored in browser history - this is an accepted risk per requirements
- Rate limiting will prevent brute force attacks
- Session isolation will ensure user separation
- Security headers will be implemented for web security

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

基于我对 forgecode 项目的分析，我理解你想要构建一个类似 OpenClaw Gateway 的 Web UI 网关，主要功能是：
## 需求理解
1. **网关功能**：类似 OpenClaw Gateway 的 Web UI 界面
2. **鉴权方式**：通过 URL 中的 token 进行身份验证
3. **部署场景**：将 opencode 部署在内网容器中，可以在另一台电脑上远程操作
4. **核心用途**：通过 Web 界面使用 forgecode 进行编程
## 项目现状分析
forgecode 是一个 Rust 项目，使用 workspace 架构，包含多个 crate：
- `forge_main` - 主程序入口
- `forge_services` - 服务层，包含 MCP 服务、工具服务等
- `forge_config` - 配置管理
- 其他工具和辅助模块
## 需要确认的关键点
1. **Web 框架选择**：你希望使用什么 Web 框架？比如 Actix-web、Warp、Axum 等？
2. **前端技术栈**：Web UI 是纯后端渲染，还是需要前端框架（如 React/Vue）？
3. **token 生成机制**：token 如何生成和管理？是否需要用户管理功能？
4. **部署架构**：opencode 和 gateway 的部署关系是怎样的？是同一个容器还是分开部署？
请确认这些细节，然后我就可以开始为你制定实现计划。



### A1

web框架你定一个比较成熟简单的，token有一个初始化的 url token用户可以进入页面后再renew 一个，同一个容器他们是一体的服务只是不同模块

---

## Implementation Notes

- For any unspecified details (combat formulas, game balance, UI layout, tech choices, etc.), make reasonable decisions yourself and document them in the plan. Do NOT ask the user for clarification — proceed with sensible defaults.
- If referenced image files exist in the workspace, treat them as visual style references.

## Standard Deliverables (mandatory for every project)

- **README.md** — must be included at the project root with: project title & description, prerequisites, installation steps, usage examples with code snippets, configuration options, and project structure overview.
- **Git commits** — use conventional commit prefix `feat:` for all commits.

--- Original Design Draft End ---
