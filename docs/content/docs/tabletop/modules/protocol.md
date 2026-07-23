# `protocol`

**Path**: `src/protocol.rs`

## Descrição

 Ponte do cliente para o domínio compartilhado.

 Re-exporta todos os tipos do crate [`shared`] (Bevy-free) e adiciona os

 adaptadores específicos do cliente — a conversão de cor para `bevy::Color`.

 Assim o resto do código continua usando `crate::protocol::*` sem mudança.

 Ver ADR-009.

## Funções

### `palette_color`

```rust
fn palette_color(i : u8) -> Color
```

 Cor da paleta por índice cru — conveniência de UI (retorna `bevy::Color`).

## Implementações

### `impl ColorIdxExt for ColorIdx`

- `color`

