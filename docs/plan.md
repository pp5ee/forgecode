# Forgecode Gateway Implementation Plan

## Goal Description
Implement a web UI gateway for forgecode similar to OpenClaw Gateway, allowing users to access forgecode programming capabilities through a web interface with URL-based token authentication. The gateway will enable deployment of opencode in internal network containers and remote access from other computers.

## Acceptance Criteria

Following TDD philosophy, each criterion includes positive and negative tests for deterministic verification.

- AC-1: Gateway provides web UI interface accessible via browser
  - Positive Tests (expected to PASS):
    - User can access gateway homepage via web browser
    - Web UI loads without errors and displays forgecode interface
    - All forgecode functionality is accessible through web UI
  - Negative Tests (expected to FAIL):
    - Gateway returns error when accessed without valid token
    - Invalid URLs return appropriate HTTP error codes
    - Web UI fails to load with network connectivity issues

- AC-2: URL token authentication system
  - Positive Tests (expected to PASS):
    - Users can access gateway with valid URL token parameter
    - Initial token allows access to gateway interface
    - Users can renew tokens from within the web interface
    - Renewed tokens provide continued access
  - Negative Tests (expected to FAIL):
    - Invalid tokens are rejected with appropriate error message
    - Expired tokens cannot access gateway functionality
    - Token renewal fails with invalid input

- AC-3: Integration with forgecode core functionality
  - Positive Tests (expected to PASS):
    - All existing forgecode commands work through web interface
    - Code execution results are displayed in web UI
    - File operations (read/write) function correctly
    - Terminal output is streamed to web interface
  - Negative Tests (expected to FAIL):
    - Invalid commands return appropriate error messages
    - Permission denied errors are properly handled
    - Network timeouts are gracefully managed

- AC-4: Container deployment support
  - Positive Tests (expected to PASS):
    - Gateway can be deployed alongside forgecode in same container
    - Internal network access is properly configured
    - Multiple instances can run without conflicts
    - Gateway handles container lifecycle events
  - Negative Tests (expected to FAIL):
    - Gateway fails to start when required ports are occupied
    - Container resource constraints are properly handled
    - Network isolation prevents unauthorized access

## Path Boundaries

Path boundaries define the acceptable range of implementation quality and choices.

### Upper Bound (Maximum Acceptable Scope)
The implementation includes a complete web UI gateway with token-based authentication, real-time terminal streaming, file management interface, session management, and comprehensive integration with all forgecode functionality. The gateway supports multiple concurrent users with proper isolation and includes monitoring and logging capabilities.

### Lower Bound (Minimum Acceptable Scope)
The implementation includes a basic web interface that allows token-based access to core forgecode functionality. Users can execute commands and view results through a simple web terminal interface. Basic token management and container deployment support are included.

### Allowed Choices
- Can use: Actix-web or Axum web frameworks (mature and well-supported for Rust)
- Can use: WebSocket or Server-Sent Events for real-time communication
- Can use: JWT or simple UUID tokens for authentication
- Cannot use: External authentication providers (must be self-contained)
- Cannot use: Complex frontend frameworks if simple HTML/JS suffices

## Feasibility Hints and Suggestions

> **Note**: This section is for reference and understanding only. These are conceptual suggestions, not prescriptive requirements.

### Conceptual Approach
The gateway can be implemented as a new crate in the forgecode workspace that provides:
1. HTTP server with token authentication middleware
2. WebSocket/SSE endpoint for real-time terminal communication
3. REST API endpoints for forgecode operations
4. Static file serving for web UI assets
5. Integration with existing forgecode services through internal APIs

### Relevant References
- `crates/forge_services/` - Contains MCP services and tool services
- `crates/forge_api/` - API layer for forgecode functionality
- `crates/forge_config/` - Configuration management
- `crates/forge_main/` - Main application entry point

## Dependencies and Sequence

### Milestones
1. **Gateway Foundation**: Create basic HTTP server with token authentication
   - Phase A: Set up web framework and routing
   - Phase B: Implement token validation middleware
   - Phase C: Create basic web interface structure

2. **Forgecode Integration**: Connect gateway to forgecode functionality
   - Step 1: Expose forgecode commands through REST API
   - Step 2: Implement real-time terminal streaming
   - Step 3: Add file management capabilities

3. **Web UI Enhancement**: Improve user experience and functionality
   - Phase A: Enhance web interface with better UX
   - Phase B: Add session management and token renewal
   - Phase C: Implement monitoring and logging

## Task Breakdown

Each task must include exactly one routing tag:
- `coding`: implemented by Claude
- `analyze`: executed via Codex (`/humanize:ask-codex`)

| Task ID | Description | Target AC | Tag (`coding`/`analyze`) | Depends On |
|---------|-------------|-----------|----------------------------|------------|
| task1 | Create gateway crate structure | AC-1 | coding | - |
| task2 | Implement HTTP server with Actix-web | AC-1 | coding | task1 |
| task3 | Design token authentication system | AC-2 | analyze | task2 |
| task4 | Implement token validation middleware | AC-2 | coding | task3 |
| task5 | Create basic web UI interface | AC-1 | coding | task2 |
| task6 | Integrate with forgecode API layer | AC-3 | coding | task4 |
| task7 | Implement command execution endpoint | AC-3 | coding | task6 |
| task8 | Add real-time terminal streaming | AC-3 | coding | task7 |
| task9 | Implement token renewal functionality | AC-2 | coding | task4 |
| task10 | Add container deployment configuration | AC-4 | coding | task2 |
| task11 | Create comprehensive test suite | All AC | coding | task10 |

## Claude-Codex Deliberation

### Agreements
- Gateway should be implemented as a new crate in the existing workspace
- Token authentication is appropriate for this use case
- Integration with existing forgecode architecture is essential

### Resolved Disagreements
- Framework choice: Both agree Actix-web is suitable given forgecode's existing async ecosystem
- Authentication approach: Simple URL tokens with renewal mechanism is appropriate

### Convergence Status
- Final Status: `converged`

## Pending User Decisions

- DEC-1: Web UI complexity level
  - Claude Position: Start with simple terminal-like interface, enhance based on user feedback
  - Codex Position: N/A - open question
  - Tradeoff Summary: Simple interface is faster to implement but may require more iterations
  - Decision Status: `PENDING`

## Implementation Notes

### Code Style Requirements
- Implementation code and comments must NOT contain plan-specific terminology such as "AC-", "Milestone", "Step", "Phase", or similar workflow markers
- These terms are for plan documentation only, not for the resulting codebase
- Use descriptive, domain-appropriate naming in code instead

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
