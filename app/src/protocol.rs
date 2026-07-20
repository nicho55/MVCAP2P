//! Ponte do cliente para o domínio compartilhado.
//!
//! Re-exporta todos os tipos do crate [`shared`] (Bevy-free) e adiciona os
//! adaptadores específicos do cliente — a conversão de cor para `bevy::Color`.
//! Assim o resto do código continua usando `crate::protocol::*` sem mudança.
//! Ver ADR-009.

pub use shared::*;

use bevy::prelude::Color;

/// Extensão do cliente: converte um índice de cor da paleta em `bevy::Color`.
pub trait ColorIdxExt {
    fn color(self) -> Color;
}

impl ColorIdxExt for ColorIdx {
    fn color(self) -> Color {
        let [r, g, b] = self.rgb();
        Color::srgb(r, g, b)
    }
}

/// Cor da paleta por índice cru — conveniência de UI (retorna `bevy::Color`).
pub fn palette_color(i: u8) -> Color {
    ColorIdx::new(i).color()
}
