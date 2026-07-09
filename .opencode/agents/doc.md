---
name: doc
description: Documenta arquivo/módulo Rust com docstrings + atualiza docgen
---

Adicione docstrings (`///`) em todos os itens públicos do arquivo ou
módulo Rust especificado. Siga rigorosamente o formato do projeto:

## Formato

### Topo do módulo (`//!`)
Sempre que estiver documentando o primeiro arquivo de um módulo, adicione
`//!` no topo explicando o propósito geral do módulo.

Exemplo:
```rust
//! # Módulo: net
//!
//! Gerencia conexões WebRTC via Matchbox: poll de peers,
//! envio/recebimento de mensagens, transferência de blobs
//! e reconexão automática.
```

### Structs/Resources/Components
```rust
/// Recurso principal de rede.
/// Gerencia o socket WebRTC, reconexão e referência ao GM.
pub struct Net { ... }
```

### Enums (especialmente `Msg` em protocol.rs)
```rust
/// Mensagens trocadas entre peers via WebRTC.
pub enum Msg {
    /// Jogador se apresenta ao GM ao conectar.
    Hello(PlayerMeta),
    ...
}
```

### Funções públicas e sistemas Bevy
```rust
/// Conecta ao servidor de sinalização via WebSocket.
/// Cria um MatchboxSocket com canal confiável.
pub fn connect(&mut self, url: &str) { ... }
```

### Funções privadas importantes (sistemas Bevy)
Também documente com `///` — o docgen as captura.

## Regras

- **Idiomático**: mantenha conciso, uma ou duas frases por item
- **Português**: documente em português
- **Contexto Bevy**: se for um sistema, Resource ou Component, mencione
- **Não documente** implementações triviais (`Default`, `Clone`, getters óbvios)
- **Não quebre** o código — apenas adicione comentários

## Processo para cada arquivo

1. Leia o arquivo inteiro para entender o contexto
2. Adicione `//!` module-level se não existir
3. Adicione `///` em cada item público sem docstring
4. Rode `cargo check` para garantir que não quebrou
5. Rode `cargo run -p docgen` para regenerar os .md
6. Mostre um resumo do que foi documentado
