# ADR-010: Física Determinística — Rapier Restrito + Fallback Ponto Fixo

**Data:** 2026-07-20
**Status:** Aceito

## Contexto

VTT "super leve" mobile-first. O rollback do **GGRS** exige simulação
**bit-idêntica** entre x86 (PC) e ARM (celulares/RPi). O IEEE 754 **não** garante
consistência bit-a-bit em operações transcendentais entre arquiteturas, causando
divergências acumuladas de estado (desync). Determinismo é **inegociável**.

## Decisão (OS "Determinismo e Colisões")

**1. Rapier restrito (tentativa deste sprint):**
- `enhanced-determinism` **ON** (conformidade estrita IEEE 754-2008, math via libm).
- Remover **`simd-stable`** e **`parallel`** — ambas quebram a consistência binária
  (SIMD muda ordem/arredondamento; paralelismo é não-determinístico).

**2. Fallback oficial — ponto fixo (se o Pi acusar desync de física):**
- **Math determinística:** trocar `f32` por inteiros escalados (crate `fixed`);
  integração estrita `x_{t+1} = x_t + v_t · Δt_fixed`.
- **Geometria simplificada:** terreno é prisma low-poly + grade regular
  (quadrada/hex) → colisão simplificada e **snap-to-grid sobre coordenadas
  inteiras** (nossa `Cell = (i32, i32)` já é inteira).
- **Performance mobile:** mais simples e rápido que um motor genérico; elimina a
  sobrecarga do Rapier em hardware fraco.

**Decisão de Gerência:** determinismo absoluto **acima** de velocidade (SIMD). Se
a física flutuante falhar no ARM, o caminho **oficial** é colisão em ponto fixo.

## Protocolo de validação (após commit que altere movimento/colisão)

- Rodar `SyncTestSession` (GGRS) para forçar rollbacks e validar convergência.
- Reportar no GitHub: **FPS médio** + `checksum_component_with_hash` dos
  componentes de posição.

## Observação de engenharia

Para este VTT (movimento em grade, `Cell` já inteira, terreno em prismas), a
**colisão em ponto fixo sobre a grade é provavelmente o encaixe primário**, não só
o fallback — o Rapier + `KinematicCharacterController` é pesado para um jogo de
grade. Seguimos a diretriz (Rapier restrito primeiro), mas prontos para pivotar
cedo se o custo/desync no ARM não compensar.

## Consequências

- Sem SIMD/paralelismo no Rapier → física single-thread escalar (aceitável no
  escopo "super leve").
- O caminho ponto-fixo reaproveita a matemática de grade já testada no `shared`.
- Todo sistema que toca posição/colisão entra na auditoria de ordenação (ADR-009
  T9) para não introduzir ambiguidade que quebre o rollback.

## Referências

- OS "Diretriz de Determinismo e Colisões (Abordagem Tática)"
- ADR-009 (fase Interface 2.0), crate `shared` (matemática de grade)
