# `graphics`

**Path**: `src/game/graphics.rs`

## Descrição

 Opções gráficas ajustáveis em runtime — foco em desempenho em dispositivos

 fracos (Android). Cada campo liga/desliga um custo relevante de GPU/CPU e é

 controlável pelo painel "Gráficos" do HUD. Defaults começam baixos no Android.

## Resources (Bevy)

### `GraphicsSettings`

 Estado das opções gráficas. É a fonte única — os sistemas reagem à mudança.

| Campo | Tipo |
|-------|------|
| `msaa` | `MsaaLevel` |
| `shadows` | `bool` |
| `hdr` | `bool` |
| `vegetation` | `bool` |
| `grid_overlay` | `bool` |
| `power_saver` | `bool` |

## Components (Bevy)

### `GfxUiRoot`

### `GfxPanel`

### `GfxOpenBtn`

### `GfxToggleBtn`

## Enums

### `MsaaLevel`

 Nível de anti-serrilhado (MSAA). 4x é o padrão do Bevy e o mais caro.

**Derives**: Clone, Copy, PartialEq, Eq, Debug

| Variante | Campos |
|----------|--------|
| `Off` | `—` |
| `X2` | `—` |
| `X4` | `—` |

### `GfxOption`

**Derives**: Clone, Copy, PartialEq, Eq

| Variante | Campos |
|----------|--------|
| `Msaa` | `—` |
| `Shadows` | `—` |
| `Hdr` | `—` |
| `Vegetation` | `—` |
| `Grid` | `—` |
| `PowerSaver` | `—` |

## Funções

### `sz`

```rust
fn sz(n : f32, si : & ScreenInfo) -> f32
```

### `btn_text`

```rust
fn btn_text(opt : GfxOption, s : & GraphicsSettings) -> String
```

### `gfx_open_click`

```rust
fn gfx_open_click(q : Query < & Interaction , (Changed < Interaction > , With < GfxOpenBtn >) >, mut panel : Query < & mut Visibility , With < GfxPanel > >) -> ()
```

 Abre/fecha o painel ao clicar em "Gráficos".

### `gfx_toggle_click`

```rust
fn gfx_toggle_click(q : Query < (& Interaction , & GfxToggleBtn) , Changed < Interaction > >, mut settings : ResMut < GraphicsSettings >) -> ()
```

 Aplica o toggle da opção clicada ao recurso.

### `gfx_panel_visuals`

```rust
fn gfx_panel_visuals(settings : Res < GraphicsSettings >, mut q_btn : Query < (& GfxToggleBtn , & Children , & mut BackgroundColor) >, mut q_text : Query < & mut Text >) -> ()
```

 Reflete o estado atual nos rótulos/cores dos botões do painel.

## Systems (Bevy)

### `apply_graphics`

 Aplica as opções ao mundo sempre que `GraphicsSettings` muda (e uma vez ao

 iniciar, pois o recurso conta como "changed" no primeiro acesso).

**Parâmetros**: `mut commands : Commands`, `settings : Res < GraphicsSettings >`, `cam : Query < Entity , With < MainCamera > >`, `mut msaa_q : Query < & mut Msaa , With < MainCamera > >`, `mut light : Query < & mut DirectionalLight >`, `mut veg : Query < & mut Visibility , With < Vegetation > >`, `mut winit : ResMut < WinitSettings >`

### `toggle_btn`

**Parâmetros**: `parent : & mut ChildSpawnerCommands`, `opt : GfxOption`, `s : & GraphicsSettings`, `assets : & GameAssets`, `si : & ScreenInfo`

### `spawn_gfx_ui`

 Cria o botão "Gráficos" (canto superior direito) e o painel (oculto).

**Parâmetros**: `mut commands : Commands`, `settings : Res < GraphicsSettings >`, `assets : Res < GameAssets >`, `si : Res < ScreenInfo >`

### `despawn_gfx_ui`

**Parâmetros**: `mut commands : Commands`, `q : Query < Entity , With < GfxUiRoot > >`

## Implementações

### `impl MsaaLevel`

- `next`
- `label`
- `to_msaa`

### `impl Default for GraphicsSettings`

- `default`

### `impl GfxOption`

- `name`
- `is_on`
- `value`
- `toggle`

## Constantes

| Nome | Tipo | Valor |
|------|------|-------|
| `PANEL` | `Color` | `Color :: srgba (0.10 , 0.09 , 0.14 , 0.95)` |
| `BTN_BG` | `Color` | `Color :: srgb (0.16 , 0.14 , 0.21)` |
| `ON` | `Color` | `Color :: srgb (0.18 , 0.40 , 0.22)` |
| `GOLD` | `Color` | `Color :: srgb (0.83 , 0.69 , 0.22)` |
| `TEXT` | `Color` | `Color :: srgb (0.92 , 0.90 , 0.95)` |

