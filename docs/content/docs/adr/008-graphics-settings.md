# ADR-008: Opções Gráficas em Runtime para Dispositivos Fracos

**Data:** 2026-07-19
**Status:** Aceito

## Contexto

O VTT é 3D (PBR, sombras, MSAA) e precisa rodar em celulares antigos (ex.: Galaxy
J7 com Mali-T830, sem Vulkan decente). Em vez de fixar qualidade por `#[cfg]` de
plataforma, decidimos expor as opções que mais pesam **como toggles em runtime**,
ajustáveis pelo próprio aparelho. Isso também serve à meta de **certificar
desempenho**: liga/desliga cada custo e mede o impacto no dispositivo real.

## Auditoria — condições gráficas que impactam desempenho

| Condição | Impacto em GPU fraca | Onde no código | Toggle |
|---|---|---|---|
| **MSAA** (4x é o padrão do `Camera3d`) | **Alto** — banda em GPUs por tiles | `camera::setup_camera` (`Msaa`) | ✅ Off/2x/4x |
| **Sombras** direcionais em cascata | **Alto** — extra render pass + shadow maps | `setup_lighting` (`DirectionalLight::shadows_enabled`) | ✅ on/off |
| **Render contínuo** (sem cap) | **Alto** — calor/thermal throttling | `WinitSettings` (era ausente) | ✅ Economia 30fps |
| **HDR + tonemapping** | Médio — passe extra + LUTs | `Camera::hdr` | ✅ on/off |
| **Grade via gizmos** (todo frame) | Médio — muitas linhas/draw por frame | `grid::draw_grid` | ✅ on/off |
| **Vegetação** (árvores low-poly) | Médio — draw calls/vértices | `lowpoly::spawn_tree` | ✅ on/off (Visibility) |
| Overlays de terreno (mesh por célula) | Médio — cresce com pintura/elevação | `terrain::terrain_render` | ⏳ futuro |
| Resolução de render (fill-rate) | **Alto** — custo por pixel | (não implementado) | ⏳ futuro (render scale) |
| PBR vs. unlit (StandardMaterial) | Médio-Alto — shading por pixel | materiais em `lowpoly` | ⏳ futuro (modo "flat") |
| Resolução de rasterização de SVG | Baixo-Médio — memória/banda de textura | `svg_assets` | ⏳ futuro |

> **Não usados hoje** (logo, sem custo): Bloom, SMAA/FXAA como pós, DoF, SSAO.

## Decisão

Módulo `game/graphics.rs` com:

1. **`GraphicsSettings` (Resource)** — fonte única do estado. Campos: `msaa`,
   `shadows`, `hdr`, `vegetation`, `grid_overlay`, `power_saver`. Defaults
   **baixos no Android**, **cheios no desktop** (via `cfg!`).
2. **`apply_graphics`** — sistema que, a cada mudança do recurso, aplica ao mundo:
   MSAA e HDR na câmera, `shadows_enabled` na luz, `Visibility` nas árvores e o
   modo do `WinitSettings` (contínuo × `UpdateMode::reactive(1/30s)`). O modo
   reativo **acorda ao menos a cada 33 ms**, então updates de rede continuam
   fluindo — economia sem congelar o jogo.
3. **Painel "Gráficos" no HUD** — botão no canto superior direito abre um painel
   com um botão por opção (rótulo + estado ON/OFF, MSAA cicla Off→2x→4x). Verde =
   ligado. `gfx_toggle_click` altera o recurso; `gfx_panel_visuals` reflete o
   estado; `apply_graphics` propaga a mudança.

A grade é *gate* por `grid_overlay`; as árvores ganharam o marcador `Vegetation`.

## Consequências

### Positivas
- Ajuste por aparelho sem recompilar; ideal para medir desempenho por opção.
- Substitui os `#[cfg]` de render por um único ponto de verdade (o recurso).
- Base pronta para as próximas alavancas (render scale, modo flat).

### Negativas / limitações
- Ainda **não** há render scale (a maior alavanca de fill-rate) nem modo unlit —
  ficam como próximos passos.
- `power_saver` reativo depende do `wait`; ~30 FPS é um teto, não economia total.
- Toggling de HDR/MSAA reconstrói pipeline (pequeno hitch pontual ao alternar).

## Referências

- `app/src/game/graphics.rs` — recurso, `apply_graphics` e painel
- `app/src/game/camera.rs`, `mod.rs::setup_lighting`, `grid.rs::draw_grid`
- ADR-007 — deploy/medição em celulares físicos (onde estes toggles são medidos)
