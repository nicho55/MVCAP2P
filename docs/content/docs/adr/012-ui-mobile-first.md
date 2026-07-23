# ADR-012: UI Mobile-First

## Status

Accepted

## Context

A UI do projeto não era responsiva — painéis usavam larguras fixas em pixels, o lobby não se adaptava a telas estreitas, e o Android não tinha joystick virtual. Additionally, touch events on Android were not properly guarded against UI interactions, causing tokens and terrain tools to activate when touching toolbar buttons.

## Decision

Implement UI Mobile-First with the following changes:

### 1. ScreenInfo Reactivo (`game/mod.rs`)

O resource `ScreenInfo` agora recalcula a escala automaticamente a cada frame quando `auto_scale` está ativo. Antes, a escala só era calculada uma vez no startup. Agora reage a:
- Redimensionamento de janela (desktop)
- Rotação de tela (mobile)

Mudança: `screen_update` roda em `First` e sempre recalcula quando `auto_scale == true`.

### 2. Janela Desktop Redimensionável (`lib.rs`)

Adicionado `resizable: true` na janela desktop. Antes a janela era fixa em 1366x840.

### 3. Lobby Responsivo (`lobby.rs`)

- Painéis usam `flex_grow` + `min_width` em vez de `width` fixa (440px/300px → flex)
- Row com `flex_wrap: Wrap` — empilha verticalmente em telas estreitas
- Todos os elementos usam `sz()` para escala responsiva
- Touch targets aumentados para ≥40px (swatches: 36→40px, inputs: 42→46px)
- Novo sistema `lobby_responsive` reconstrói o lobby quando a escala muda

### 4. HUD Adaptado (`game/hud.rs`)

- Botão "X" (deletar token) na toolbar — aparece quando token selecionado
- Sistemas: `delete_btn_visibility`, `delete_btn_click`
- Roster panel usa container flex column (elimina overlap com info panel)
- `min_width` do roster é percentual (não mais fixo em 170px)

### 5. Botão Sala de Teste (`lobby.rs`)

Novo botão "SALA DE TESTE (GM + 4 TOKENS)" abaixo de "CRIAR SALA (MESTRE)".
Cria uma sessão GM com `demo: true`, que spawn 4 tokens de demonstração.

### 6. Joystick Virtual Android (`game/virtual_joystick.rs`)

Novo módulo com dois joysticks virtuais:
- **Esquerdo**: Pan da câmera (arrastar o chão)
- **Direito**: Órbita da câmera (yaw/pitch)

Componentes:
- `JoystickState` — resource com deslocamento normalizado (left/right: Vec2)
- `TouchAssignments` — mapeia finger IDs para sticks
- `StickGeometry` — centros, raio, diâmetro do thumb

Sistemas (encadeados):
1. `joystick_input` — atribui touches por metade da tela
2. `update_thumb_positions` — sincroniza visual do thumb
3. `joystick_apply` — aplica forças ao CamRig

Visível apenas em Android (`#[cfg(target_os = "android")]`).

### 7. Fix Touch Android UI Guard (`game/tokens.rs`, `game/terrain.rs`)

- `touch_interact`: adicionado check `ui.0` no `TouchPhase::Started` — toque em UI não seleciona tokens
- `terrain_tool`: check `ui.0` agora bloqueia touch (não só mouse) — toque em UI não pinta terreno

## Consequences

### Positivas
- UI adaptável a qualquer resolução/orientação
- Android ganha joystick virtual funcional
- Touch events respeitam UI (sem Seleção/acão indesejada)
- Botão de teste permite validação rápida sem CLI

### Negativas
- Joystick virtual é básico (sem visual de feedback háptico)
- Escala automática pode precisar de ajuste em DPIs extremos
- Lobby reconstrói inteiro a cada mudança de escala (pode causar flicker)

### Neutras
- `delete_selected_entity()` extraída como função pública reutilizável
- `CliArgs.demo` é modificado diretamente pelo botão de teste
