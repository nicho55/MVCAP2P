# ADR-009: Fase Interface 2.0 + Movimentação Híbrida — Plano e Decisões

**Data:** 2026-07-20
**Status:** Aceito (em execução)

## Contexto

Ordem de Serviço "Interface 2.0 e Movimentação Híbrida": transição atômica entre
exploração em tempo real e combate tático, resiliente/determinística, alvo
**Android Legacy**. Stack pedida: **SpacetimeDB** (persistência/identidade),
**GGRS** (rollback P2P), **bevy_rapier** (física), **bevy-inspector-egui**
(inspetor), joystick virtual, workspace virtual client/server.

## Achado que muda o ponto de partida

O ecossistema pedido exige **Bevy 0.19**; estamos no **0.16**:

| Lib | Exige Bevy |
|---|---|
| bevy_ggrs 0.22 | ^0.19 |
| bevy_rapier3d 0.35 | ^0.19 |
| bevy-inspector-egui 0.37 | ^0.19 |
| bevy_egui 0.41 | ^0.19 |
| bevy_matchbox 0.14 | ^0.18 |

➡️ **Pré-requisito gate:** migração **Bevy 0.16 → 0.19** antes do novo stack.

## Decisões do Arquiteto (blockers resolvidos)

1. **Determinismo:** Rapier com `enhanced-determinism` **ON** e `simd-stable`
   **OFF** (as duas são mutuamente exclusivas). Math escalar via `nalgebra`/`libm`.
2. **sccache:** fora do `.cargo/config.toml` versionado (quebraria o CI que não
   tem sccache); usar **env var local** (`RUSTC_WRAPPER` no ambiente/Dockerfile).
3. **Cross-arch (x86↔ARM):** seguir traits do `nalgebra`/libm por ora; se o Pi
   acusar desync, migrar para **ponto fixo** no próximo sprint (risco monitorado).

## Arquitetura alvo (workspace virtual, modular e reutilizável)

```
shared/     tipos de domínio SEM Bevy (Msg, ColorIdx, RoomCode, grid math...)
            reutilizável por client (Bevy) e server (SpacetimeDB WASM)
client/     app Bevy (render, UI 2.0, GGRS, Rapier, joystick)  [hoje: app/]
server/     módulo SpacetimeDB WASM (reducers, tabela Player, ctx.rng())
signaling/  matchbox signaling (roda no Raspberry Pi — nó de teste)
```

Camadas de rede: **SpacetimeDB** = persistência/identidade; **GGRS** = rollback
de tempo-real/turnos (modelo "lista de solicitações"); **matchbox** = signaling.

## Tarefas e prioridade (dependência-ordenada)

**P0 — Fundação (gate):**
- **T1** Workspace virtual + extrair crate `shared` (domínio sem Bevy). ← *começa aqui*
- **T2** Upgrade Bevy 0.16 → 0.19 (client). Rede de segurança: 12 testes + CI.

**P1 — Semana 1 (Infra + UI 2.0):**
- **T3** bevy_egui + bevy-inspector-egui (Inspetor Superior, topo da tela).
- **T4** UI 2.0 mobile-first: menu flutuante modular, áreas de toque ergonômicas
  (evolui a fundação `graphics.rs`/HUD já responsiva).
- **T5** Pré-rasterização de SVG (economia de heap) — estende `svg_assets`.

**P2 — Semana 2 (Física + transições):**
- **T6** bevy_rapier3d (enhanced-determinism, sem SIMD) + KinematicCharacterController.
- **T7** Joystick virtual → vetores de força.
- **T8** Transição atômica real-time ↔ turnos (AppState/typestate).
- **T9** Auditoria de ordenação de sistemas (elimina ambiguidades p/ rollback).

**P3 — Semana 3 (Rede + persistência):**
- **T10** Server SpacetimeDB (WASM): reducers, tabela Player, `ctx.rng()`.
- **T11** GGRS rollback (bevy_ggrs, request-list).
- **T12** `spacetime generate` (bindings client/server).
- **T13** Telemetria: `deploy-farm` → template de relatório da OS.

**Transversais:** sccache via env; Pi como nó de signaling (matchbox); relatório
de telemetria por commit; `CARGO_INCREMENTAL=0` nos relatórios de performance.

## Realismo de cronograma

21 dias cobrem uma **fundação/spike sólida** (workspace + upgrade + UI 2.0 +
prova-de-conceito de física/rede), não todo o sistema em qualidade de produção.
Expectativa alinhada com a Gerência.

## Consequências

- O protocolo atual (matchbox `Msg` req→GM→broadcast em `net.rs`/`sync.rs`) será
  **largamente substituído** por GGRS+SpacetimeDB; parte migra, parte deprecia.
- `shared` desacopla domínio do Bevy → reutilizável no server WASM e imune ao
  upgrade de engine (ganho modular imediato).
- Física+rollback+inspetor em Android Legacy é pesado: os toggles de
  `graphics.rs` e o teste no Pi são as válvulas de escape.

## Referências

- OS "Interface 2.0 e Movimentação Híbrida"
- ADR-006 (Type-Driven), ADR-007 (deploy/telemetria), ADR-008 (opções gráficas)
