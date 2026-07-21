# ADR-011: Revisão de Roadmap — VTT P2P Simples (sem SpacetimeDB/GGRS/Rapier)

**Data:** 2026-07-21
**Status:** Aceito
**Substitui:** ADR-009 (Interface 2.0), ADR-010 (Física Determinística)

## Contexto

O ADR-009 definiu uma "Ordem de Serviço Interface 2.0 e Movimentação Híbrida"
com um stack ambicioso: **SpacetimeDB** (persistência/identidade), **GGRS**
(rollback P2P), **bevy_rapier** (física determinística) e **bevy-inspector-egui**
(inspetor). O ADR-010 especificou o setup do Rapier com `enhanced-determinism`.

Durante a execução do P0 (workspace + shared + upgrade Bevy 0.16→0.18), o
mantenedor reavaliou a direção do projeto e concluiu que:

1. **SpacetimeDB** é um desvio do propósito original — o VTT é P2P por
   definição, sem servidor de jogo. Adicionar um banco de dados externo para
   identidade quebra a simplicidade e a portabilidade (WASM, dependência de
   rede o tempo todo).
2. **GGRS + rollback** é overkill para um VTT tático em grid. O movimento
   em células inteiras (`Cell = (i32, i32)`) com snap-to-grid e GM
   autoritativo já resolve a sincronização sem a complexidade de rollback.
3. **Rapier 3D** é excessivo para um jogo de grade. O sistema atual
   (grid center + elevação por inteiros) cobre as necessidades de terreno
   e posicionamento.

## Decisão

Voltar à essência do projeto: **VTT tático 3D low-poly peer-to-peer em
Rust/Bevy**, com GM autoritativo e snap-to-grid. O stack adicional previsto
no ADR-009/ADR-010 é descartado.

### O que é cortado

| Tecnologia | Substituição | Justificativa |
|---|---|---|
| SpacetimeDB | Identidade local (token em arquivo) | Sem servidor externo; token 256-bit persistido em `~/.local/share/tabletop/identity.json` |
| GGRS rollback | GM autoritativo (já existe) | Movimento em grid (`Cell` inteiro + `Msg::MoveTokenReq` → GM valida → `Msg::TokenMoved`) |
| bevy_rapier3d | Snap-to-grid + elevação por `i8` | Terreno em prismas low-poly com altura por célula (já implementado) |
| bevy-inspector-egui | Bevy UI nativa | Evita dependência pesada (egui é ~1s de compile extra) |
| bevy_ggrs | — | Remove por completo; não há necessidade de rollback em VTT de grade |

### O que permanece

| Tecnologia | Uso |
|---|---|
| **Bevy 0.18** | Engine de render + UI + ECS |
| **matchbox + WebRTC** | Signaling + P2P DataChannel (confiável/ordenado) |
| **GM autoritativo** | `Req` → GM valida → `broadcast` (já implementado em `sync.rs`) |
| **shared crate** | Tipos de domínio sem Bevy (reutilizável por ferramentas) |
| **docgen** | Geração automática de documentação |

### Novo Roadmap (simplificado)

```
P0 ✅ Fundações (workspace + shared + Bevy 0.18)
P1 ── Identidade persistente local
│     Token 256-bit em arquivo + username recall
│     (substitui o rand::random() por sessão)
│
P2 ── Infra de testes automatizados
│     --bench-mode + bots + logging FPS/RAM
│     (QA automation, sem GGRS)
│
P3 ── UI 2.0 mobile-first
│     UI responsiva, joystick virtual p/ Android
│     (Bevy UI nativa, sem egui)
│
P4 ── Rodada de testes manual + estabilização
│     QA em hardware real, CI maduro, docs
│
P5 ── Melhorias futuras (não planejadas)
      Transição real-time↔turnos (se necessário),
      ferramentas de GM, macros de habilidade
```

### Issues no Kanban

| Issue | Ação |
|---|---|
| #1 [Infra] Workspace + Cache | ✅ Done |
| #2 [Rede] Core de Identidade SpacetimeDB | Renomear → **"Core de Identidade Local P2P"**; remover referências a SQL/Reducer; manter só token persistente + username recall |
| #3 [Física] Setup Rapier Determinístico | ❌ **Fechar como `wontfix`** (Rapier cortado) |
| #4 [UI] Framework de Interface ECS | Renomear → **"UI 2.0 Mobile-First"**; remover egui/inspector; manter só Bevy UI nativa |
| #5 [QA] Configuração RPi3 | Manter — útil para teste em hardware fraco |
| #6 🧪 Rodada de Testes v0.1.0-alpha | Manter — QA manual ainda necessário |
| #9 [QA] Sala de Testes Automatizada | Refatorar — remover dependência de GGRS/SyncTest; manter só `--bench-mode` + bots + logging |

## Consequências

### Positivas
- Projeto volta a ser **P2P de verdade** (sem dependência de servidor externo)
- Complexidade drasticamente reduzida (sem rollback, sem física determinística)
- Build mais rápido (sem rapier, sem egui, sem spacetime SDK)
- Código existente (`sync.rs`, `tokens.rs`, `terrain.rs`) permanece sem grandes
  mudanças — só a identidade precisa de refatoração

### Negativas
- Perde-se a "prova de conceito" de rollback + física determinística (mas isso
  nunca foi o objetivo do VTT)
- Identidade local significa que cada máquina tem seu próprio "eu" — não há
  login entre dispositivos (aceitável para um VTT P2P de mesa)
- ADRs 009 e 010 ficam como registro histórico do caminho não seguido

## Referências

- ADR-009: Fase Interface 2.0 (substituído)
- ADR-010: Física Determinística (substituído)
- Issue #2: Core de Identidade (refatorada)
- SSOT §3 e §8 (serão atualizados para refletir esta revisão)
