# AGENTS.md - TUHC-rust System Guidelines & Architecture Index

Strict behavioral rules, code encapsulation patterns, project architecture, memory hygiene constraints, and token-saving navigation protocols for AI models working in this workspace.

---

## 1. Context Efficiency & Navigation Rules

### 1.1. Read CURRENT.md First
Always read `.agents/CURRENT.md` at the start of any task before calling `list_dir` or running broad searches. It contains the active task context, architectural boundaries, and direct relative file pointers.

### 1.2. Surgical Tool Usage
- Use targeted file inspection with line bounds (`StartLine` and `EndLine`) to inspect only the function or struct you need.
- Search with exact symbol names (e.g., `AppStateInner`, `ArchiveStore`, `Vfs`, `handle_asset_request`) before reading full files.
- Avoid recursive directory listings unless mapping new modules.

### 1.3. Agent Specialization (Desktop / Rust / Tauri)
Deep architectural tasks must route to designated specialized agent personas:
- **Core Rust Backend & Axum Server:** `backend-specialist` (Tokio, Axum, Tower, Range streaming, VFS).
- **Webview & Ruffle Frontend:** `frontend-specialist` (WebKitGTK, Vue, HTML5, WebAssembly Ruffle emulator).
- **Memory Hygiene & Optimization:** `performance-optimizer` (Memory RSS < 150MB SLA, `malloc_trim`, zero-copy).
- **E2E & Contract Testing:** `qa-automation-engineer` & `test-engineer` (Dual-track tests, Tier 1-4 suites).
- **Frontend Visual & E2E Testing:** `frontend-e2e-specialist` (Playwright visual iterations, screenshots, console logs).
- **Security & Path Traversal Auditing:** `security-auditor` (VFS sanitization, bounds checks, `cargo audit`).
- **CI/CD & Release Pipeline:** `devops-engineer` (GitHub Actions, Tauri bundler, Debian/AppImage packaging).

---

## 2. Project Architecture

### 2.1. Technology Stack
- **Framework:** Tauri 2.0 (Rust 2021 edition) + WebKitGTK (Linux) / WebView2 (Windows).
- **Backend Runtime:** Rust asynchronous runtime powered by Tokio multi-threaded scheduler.
- **Embedded HTTP Server:** Axum + Tower-HTTP (permissive CORS) running on an ephemeral loopback socket (`127.0.0.1:0`).
- **Virtual File System (VFS):** Layered asset resolution overlaying Asset Pack V2 (`/storyfiles/`, `/archive/`, `/assets/`) with community mod folders.
- **Archive In-Memory Store:** High-performance in-memory indexing of 9,991 Homestuck pages (`mspa.json`) and 37 albums / 797 musical tracks (`music.json`).
- **Flash Emulation:** Ruffle WebAssembly integration with audio delay calibration tables and Flash player dimensions.
- **Configuration & State:** Persistent JSON storage adhering to `~/.config/unofficial-homestuck-collection/config.json`.
- **Packaging:** Native desktop binaries (`tuhc` executable, `.deb`, `.AppImage`).

### 2.2. Directory Structure
```
tuhc_tauri/
├── Cargo.toml                  # Workspace root
├── Cargo.lock
├── src/                        # Frontend web assets (HTML/Vue/Ruffle shell)
│   └── index.html
├── src-tauri/                  # Rust Tauri 2.0 desktop core
│   ├── Cargo.toml
│   ├── tauri.conf.json         # Tauri application configuration & window specs
│   ├── build.rs
│   ├── icons/                  # Story & system icon bundles
│   ├── capabilities/           # Tauri security capabilities
│   └── src/
│       ├── main.rs             # Application entrypoint
│       ├── lib.rs              # Tauri builder, WebKitGTK configuration, IPC wiring
│       ├── commands.rs         # Tauri IPC commands (invoke handlers)
│       ├── server/             # Embedded Axum loopback server & VFS engine
│       │   ├── mod.rs          # HTTP routing, IPC REST mirror, fallback router
│       │   ├── vfs.rs          # Virtual File System & mod overlay resolution
│       │   └── handler.rs      # Asset streaming, MIME detection, Range requests
│       ├── data/               # In-memory archive data stores
│       │   ├── mod.rs          # ArchiveStore aggregate
│       │   ├── story.rs        # StoryStore (mspa.json parser & dynamic patcher)
│       │   └── music.rs        # MusicStore (music.json parser)
│       ├── config/             # Configuration persistence
│       │   └── mod.rs          # AppConfig, Settings, TabData, serialization
│       ├── mods/               # Community mod parser & registry
│       │   ├── mod.rs          # ModRegistry
│       │   └── parser.rs       # Legacy mod.js regex tokenizer & patch extractor
│       └── bin/
│           └── server.rs       # Standalone contract test server for CI & benchmarks
├── tests/                      # Python Dual-Track E2E Test Suite (Tier 1 to 4)
│   ├── common.py               # Test harness & assertions
│   ├── mock_server.py          # Reference oracle contract server
│   ├── e2e_runner.py           # Unified test suite runner
│   ├── tier1_features/         # Core happy-path tests (Asset server, store, IPC)
│   ├── tier2_boundaries/       # Security, missing files, traversal, bad ranges
│   ├── tier3_combinations/     # Mod overlay + translations + persistence
│   └── tier4_realworld/        # Realistic reader sessions & audio sync
└── .agents/                    # Agent governance, rules, workflows & scripts
```

---

## 3. Rust & Tauri Code Encapsulation Rules

### 3.1. Layer Responsibilities & Data Flow

```
Webview Frontend / Ruffle
      │ (HTTP /assets/* or /ipc/*)             │ (Tauri invoke IPC)
      ▼                                        ▼
Axum Loopback Server                    Tauri Commands (commands.rs)
      │                                        │
      ├──────────────────┬─────────────────────┤
      ▼                  ▼                     ▼
Virtual File System    Archive Store        App Config
(VFS Layer + Mods)   (Story / Music JSON)  (config.json)
```

**Commands (`src-tauri/src/commands.rs`)**
- Only handles parameter extraction from Tauri IPC calls and delegates to `AppState`.
- Never contains business logic or direct filesystem manipulation.
- Returns typed `Result<T, String>` to serialize transparently into JS promises.

**Server & VFS (`src-tauri/src/server/`)**
- Axum handlers (`handler.rs`) handle HTTP requests for media, SWF, WASM, and JSON.
- Must support HTTP 206 Partial Content (Range header `bytes=start-end`) for streaming large flash assets and audio.
- VFS (`vfs.rs`) resolves physical paths: checks active mod trees first, then falls back to base `Asset Pack V2`.
- **Security Check:** All incoming URI paths must be sanitized via `Vfs::sanitize_path` to prevent path traversal (`..` or null-byte attacks).

**Data Stores (`src-tauri/src/data/`)**
- `ArchiveStore` holds immutable shared references to loaded `StoryStore` and `MusicStore`.
- Mod patches (e.g., Brazilian Portuguese `translation2.json`) are merged in memory via `apply_patches()` without mutating disk archives.

**Configuration (`src-tauri/src/config/`)**
- Manages reader progress, active tabs, theme overrides, and enabled mod list.
- Guarantees backward compatibility with original Collection `config.json` schema.

---

## 4. Memory Hygiene & Resource SLA (< 150MB)

The primary reason for rewriting TUHC in Rust/Tauri 2.0 is eradicating Electron's bloat (which regularly exceeded 800MB–1.2GB RSS).

### 4.1. Strict Memory Budget: Under 150MB RSS
- Under normal reading, audio playback, and mod overlay operations, the total Resident Set Size (RSS) must remain strictly **under 150MB**.
- In automated CI gatekeeping, memory benchmarks are run; exceeding 150MB fails the build.

### 4.2. Memory Management Directives:
1. **WebKitGTK Settings:**
   - WebContext cache model is explicitly set to `webkit2gtk::CacheModel::DocumentViewer`.
   - Page cache and offline web application cache are disabled to prevent hidden cache bloat.
2. **Explicit Heap Trimming:**
   - On Linux systems, invoke `libc::malloc_trim(0)` upon closing heavy tabs or after large archive indexing.
3. **Zero-Copy Streaming:**
   - Use streaming response bodies (`axum::body::Body::from_stream`) with chunked file reading instead of buffering entire files in memory.
4. **No In-Memory Asset Duplication:**
   - The Asset Pack contains ~19,000 files (~4GB). Assets are NEVER loaded into memory wholesale. They are streamed directly from disk via VFS.

---

## 5. Adding New Features (Checklist)

1. **Domain Model / Serialization:** Update or add structs in `data/`, `config/`, or `mods/` with `#[derive(Serialize, Deserialize)]`.
2. **Core Logic:** Implement business functions with robust unit tests in the same module.
3. **IPC / HTTP Endpoint:** Wire command in `commands.rs` and mirror in `server/mod.rs` if needed for browser compatibility.
4. **Register in Tauri Builder:** Expose command in `tauri::generate_handler![...]` in `lib.rs`.
5. **E2E Test Contract:** Add relevant test cases in `tests/tier1_features/` or `tests/tier2_boundaries/`.

---

## 6. Mandatory Update Rules

After any code modification or addition:
1. Update `.agents/CURRENT.md`: refresh task status, executive summary, and add new files to the Navigation Index.
2. Append entry to `.agents/COMMITS.md` using the standard format:
   ```markdown
   ## `YYYY-MM-DD HH:MM` - `type(scope)`: Short title

   - **Description:** Summary of changes...
   - **Files Modified:**
     - `path/to/file`
   - **Impact / Next Steps:** Outcome and next steps.
   ```
   > **CRITICAL TOKEN RULE:** NEVER read the entirety of `COMMITS.md` before appending. Append directly to the end.

---

## 7. Governança Rígida de Branches, Commits, CI/CD e Blindagem de Testes

### 7.1. Axioma de Commits (Sintaxe Obrigatória e Isolamento por Chat)
- **Formato Estrito:** `<category>/<short-description>` ou `<category>(<scope>): <short-description>`.
- **Categorias Permitidas:** `feat`, `fix`, `test`, `refactor`, `chore`, `docs`.
- **Regra de Sintaxe:** Mensagens fora desta sintaxe ou commits sem propósito atômico claro são rejeitados pelo pipeline de CI (`pr-gatekeeper.yml`).
- **Isolamento de Escopo por Chat (Commit & Push Cirúrgico):** Ao receber comando para commitar e/ou dar push, o agente deve selecionar cirurgicamente APENAS os arquivos que foram modificados ou criados na sessão do chat atual. Nunca utilizar `git add .` ou incluir alterações originadas de outros chats ou tarefas simultâneas na working tree.

### 7.2. Axioma de Criação de Branches (Regra Prévia Obrigatória)
- **Criação Prévia Mandatória:** O modelo DEVE OBRIGATORIAMENTE E SEMPRE criar uma branch temática a partir de `dev` (`git checkout -b <categoria>/<nome-da-task> dev`) ANTES de realizar qualquer modificação em qualquer arquivo do projeto.
- **Proibição Absoluta de Trabalho Direto em `main` ou `dev`:** É EXPRESSAMENTE PROIBIDO realizar alterações diretamente na branch `main` ou na branch `dev`. A working tree deve ser ramificada imediatamente no início de qualquer tarefa.
- **Branch Base Obrigatória:** Toda nova tarefa, feature ou correção deve nascer exclusivamente a partir da branch `dev`.
- **Nomenclatura Obrigatória:** `<categoria>/<nome-da-task>` (ex: `feat/ruffle-flash-sync`, `fix/vfs-range-request`, `test/memory-audit`).
- **Proibição:** É expressamente proibido ramificar a partir de `main`, com a única exceção de hotfixes emergenciais de produção (`fix/hotfix-...`).

### 7.3. Axioma de Merge Autônomo e Validação Obrigatória de PRs (Regra Absoluta)
- **Proibição de Merge Local:** Agentes e modelos de IA **NUNCA** executam `git merge` localmente em `dev` ou `main`.
- **Fluxo Exclusivo de Integração:**
  1. Criar branch temática a partir de `dev` antes de tocar no código: `git checkout -b <categoria>/<nome-da-task> dev`.
  2. Implementar e validar código e testes localmente (`cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check`, `python3 tests/e2e_runner.py`).
  3. Enviar branch para o repositório remoto: `git push origin <categoria>/<nome-da-task>`.
  4. Abrir Pull Request apontando para `dev`: `gh pr create --base dev --title "..." --body "..."`.
  5. **Verificação Obrigatória de Testes:** O agente DEVE acompanhar ativamente a esteira de CI (`gh pr checks <PR> --watch` ou consulta equivalente) para verificar se todos os testes passaram (`SUCCESS`). A tarefa NUNCA deve ser considerada finalizada sem essa verificação.
  6. **Merge via GitHub CLI (`gh`):**
     - Se os testes passarem: o agente DEVE obrigatoriamente utilizar a CLI `gh` da máquina do usuário para realizar o merge do PR na branch `dev` (`gh pr merge --squash --auto` ou `gh pr merge <PR> --squash`).
     - Se os testes falharem: o agente DEVE investigar a causa raiz da falha na esteira, corrigir o código, subir novo commit cirúrgico e aguardar a nova execução do CI até que seja aprovado.
- **Pré-requisito Mandatório da CLI `gh` (Instalação e Login):**
  - O agente deve verificar se o GitHub CLI está autenticado (`gh auth status`).

### 7.4. Proteção contra Testes Falsos (Anti-Tautologia) e Zonas Protegidas
- **Proibição de Testes Tautológicos:** Proibido criar testes com asserções genéricas ou vazias (ex: `assert!(true)`), testes sem validação de mutação de estado real ou mocks excessivos que disfarcem falhas de integração.
- **Barreira de Cobertura de Código:** Cobertura de testes unitários em todas as rotas e funções críticas do VFS, parser e commands.
- **Zonas Protegidas (Arquivos Intocáveis por IA para Burlar CI):**
  - Diretórios estruturais de teste: `tests/`, `tests/tier1_features/`, `tests/tier2_boundaries/`, `tests/tier3_combinations/`, `tests/tier4_realworld/` e `.github/workflows/`.
  - **Regra Inegociável:** Modelos e agentes de IA **NÃO têm permissão** para relaxar thresholds de performance (limite de memória < 150MB, timeouts), alterar asserções para falsos-positivos ou desativar steps de validação para forçar a passagem de pipelines de CI. Se um teste falhar, a causa raiz na implementação deve ser investigada e corrigida.

### 7.5. Axioma de Títulos e Descrições de Pull Requests (PRs)
- **Estilo Idêntico a Mensagens de Commit (Proibição de Slop de IA):** Expressamente proibido o uso de templates corporativos prolixos gerados por IA em títulos ou bodies de PR (sem `### Resumo`, sem listas com negrito `**`, sem emojis, sem tabelas e sem qualquer formatação de markdown).
- **Regras Estritas de Escrita:**
  - **Idioma:** Exclusivamente em inglês (`en-US`).
  - **Caixa:** Totalmente em minúsculas (`lowercase`), tanto no título quanto no `--body`.
  - **Sem Markdown:** Texto puro, corrido, sem headers, bullet points rebuscados ou negrito.
  - **Tom:** Informal, direto e despojado, com vocabulário tecnicamente denso e preciso.
- **Exemplo de Título:**
  `feat(vfs): add zero-copy streaming for large swf files`
- **Exemplo de Body (`--body`):**
  `switched vfs asset handler to use stream body with range headers, verified memory footprint stays at 68mb during cascade playback and passed tier3 combination tests`
