---
name: enhance
description: Adiciona ou aprimora features no TUHC-rust (Tauri 2.0 / Rust / Axum / WebKitGTK). Focado em precisão técnica e economia de tokens.
version: 2.0.0
requires_agents: backend-specialist, frontend-specialist, qa-automation-engineer
requires_skills: clean-code, verify-changes
artifact_outputs: changed-files, verification-report
---

# /enhance — Aprimorar Funcionalidade (TUHC-rust)

$ARGUMENTS

---

## Fluxo Otimizado (Alta Eficiência de Tokens)

1. **Localização Cirúrgica (Sem varrer pastas)**
   - Consulte `.agents/CURRENT.md` para encontrar os arquivos exatos da funcionalidade.
   - Abra apenas os arquivos necessários usando `read_file` com limites estritos de linhas (`StartLine` e `EndLine`).

2. **Detecção de Especialista**
   - **Frontend:** Alterações visuais, shell do leitor, visualização de páginas e integração com Ruffle WASM em `src/index.html` ou assets web (`frontend-specialist`).
   - **Backend:** Modificações nos serviços Rust, comandos IPC (`src-tauri/src/commands.rs`), servidor loopback Axum (`src-tauri/src/server/`), motor VFS (`src-tauri/src/server/vfs.rs`), catálogo de histórias/música (`src-tauri/src/data/`) ou gerenciador de mods (`src-tauri/src/mods/`) (`backend-specialist`).

3. **Execução Direta e Segurança**
   - Aplique as alterações de código com precisão cirúrgica via `edit_file`.
   - Mantenha estrita conformidade com o SLA de memória (< 150MB RSS total).
   - Valide caminhos de arquivos contra path traversal usando `Vfs::sanitize_path`.
   - Garanta compatibilidade com as regras em `.agents/rules/`.

4. **Validação Automática (Zero Suposição)**
   - Execute o pipeline completo de verificação:
     ```bash
     python3 .agents/scripts/verify_all.py
     ```
   - Garanta 0 avisos de compilação, 0 erros de linter (clippy com `-D warnings`), 100% dos testes unitários e E2E aprovados e RSS sob controle.

5. **Atualização do Registro de Governança**
   - Atualize o status da tarefa no `.agents/CURRENT.md`.
   - Registre uma entrada concisa no `.agents/COMMITS.md`.
