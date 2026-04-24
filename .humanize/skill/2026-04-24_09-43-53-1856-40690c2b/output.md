Based on my analysis of the forgecode project structure and the gateway proposal, here's a structured analysis:

## **CORE_RISKS**

**Authentication & Security**
- **URL token exposure risk**: URL tokens in browser history, logs, and referrer headers
- **No session management**: Initial token renewal approach lacks clear session lifecycle
- **No rate limiting**: Missing protection against brute force token guessing
- **Cross-container security**: Gateway and forgecode modules need secure IPC

**Architecture Complexity**
- **Terminal-to-Web translation**: Converting TUI/CLI interactions to web UI is non-trivial
- **Real-time requirements**: Terminal I/O needs WebSocket/SSE for responsive experience
- **State synchronization**: Maintaining terminal state across web sessions

**Deployment Assumptions**
- **Single container assumption**: Both modules in one container creates tight coupling
- **Network topology**: "内网" (intranet) deployment scope may limit external access patterns

## **MISSING_REQUIREMENTS**

**Functional Gaps**
- **Token generation/management**: No specification for token creation, rotation, revocation
- **Web UI scope**: Unclear if this provides full forgecode functionality or subset
- **File transfer**: Missing specification for file upload/download between local and remote
- **Multi-user support**: Single token implies single user, but needs clarification

**Operational Requirements**
- **Health checks**: Missing monitoring endpoints for container orchestration
- **Logging integration**: How gateway logs integrate with existing forgecode logging
- **Configuration management**: Token storage, gateway settings persistence

## **TECHNICAL_GAPS**

**Web Framework Integration**
- Current Rust stack lacks web framework dependencies (actix-web, warp, axum)
- No existing HTTP server infrastructure in forgecode
- Missing WebSocket/SSE support for real-time terminal interaction

**Terminal Emulation**
- Need full xterm.js or similar terminal emulator in browser
- PTY (pseudo-terminal) management for web sessions
- Input/output stream handling between web and forgecode process

**Container Architecture**
- IPC mechanism between gateway and forgecode modules undefined
- Resource sharing (filesystem, network) between modules
- Startup sequencing and dependency management

## **ALTERNATIVE_DIRECTIONS**

**Framework Choices** (Mature & Simple)
1. **Axum + Tokio** (Recommended): Aligns with existing tokio runtime, minimal dependencies
2. **Warp**: Functional style, but less conventional for complex applications
3. **Actix-web**: More features but heavier dependency footprint

**Authentication Alternatives**
1. **JWT tokens**: More secure than URL tokens, better session management
2. **OAuth2 flow**: For enterprise scenarios with existing identity providers
3. **API keys + session cookies**: Hybrid approach for better security

**Deployment Models**
1. **Sidecar containers**: Gateway and forgecode as separate containers with shared network
2. **Process supervisor**: Single container with process manager (supervisord)
3. **Microservices**: Fully separated services with API gateway

## **QUESTIONS_FOR_USER**

**Scope & Functionality**
1. Should the web UI provide full forgecode functionality or a specific subset (e.g., only CLI mode)?
2. Do you need file upload/download capabilities between local machine and remote forgecode instance?
3. Should multiple users be able to access the same forgecode instance concurrently?

**Technical Decisions**
4. Preferred web framework: Axum (recommended), Warp, or Actix-web?
5. Session management: Simple token renewal or full JWT-based sessions?
6. Terminal emulation: Basic terminal or full xterm.js with advanced features?

**Deployment Constraints**
7. Container resource limits: CPU/memory constraints for the combined service?
8. Network access: Should the gateway be accessible from outside the intranet?
9. Persistence: How should tokens and session data be stored (file, database, memory)?

## **CANDIDATE_CRITERIA**

**Minimum Viable Product**
- [ ] Web UI loads with URL token authentication
- [ ] Basic terminal interface with forgecode CLI access
- [ ] Token renewal mechanism after initial login
- [ ] Single-container deployment working
- [ ] Basic security headers and CORS configuration

**Production Ready**
- [ ] Secure token storage and rotation
- [ ] WebSocket-based real-time terminal
- [ ] File upload/download functionality
- [ ] Health check endpoints
- [ ] Comprehensive logging and error handling
- [ ] Rate limiting and brute force protection
- [ ] Session timeout and cleanup

**Advanced Features**
- [ ] Multi-user support with session isolation
- [ ] Advanced terminal features (copy/paste, resize)
- [ ] Configuration management UI
- [ ] Metrics and monitoring dashboard
- [ ] Backup/restore functionality

The project's existing Rust architecture and AGENTS.md guidelines provide a solid foundation, but the gateway represents a significant architectural addition that requires careful consideration of security, real-time communication, and deployment complexity.
