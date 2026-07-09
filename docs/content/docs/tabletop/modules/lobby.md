# `lobby`

**Path**: `src/lobby.rs`

## Resources (Bevy)

### `RoomList`

| Campo | Tipo |
|-------|------|
| `rooms` | `Vec < room_discovery :: RoomEntry >` |
| `loading` | `bool` |
| `error` | `Option < String >` |
| `pending` | `Arc < Mutex < Option < Result < Vec < room_discovery :: RoomEntry > , String > > > >` |
| `timer` | `Timer` |

### `LobbyForm`

| Campo | Tipo |
|-------|------|
| `nick` | `String` |
| `code` | `String` |
| `color` | `u8` |
| `focus` | `Focus` |
| `status` | `String` |

## Components (Bevy)

### `LobbyRoot`

### `NickField`

### `CodeField`

### `NickText`

### `CodeText`

### `StatusText`

### `Swatch`

### `CreateBtn`

### `JoinBtn`

### `RoomsPanel`

### `RoomsContainer`

### `RoomEntryBtn`

| Campo | Tipo |
|-------|------|
| `code` | `String` |
| `signaling` | `String` |

### `RoomEmptyLabel`

## Structs

### `LobbyPlugin`

**Derives**: 

## Enums

### `Focus`

**Derives**: PartialEq, Clone, Copy, Default

| Variante | Campos |
|----------|--------|
| `Nick` | `—` |
| `Code` | `—` |

## Funções

### `lobby_typing`

```rust
fn lobby_typing(mut form : ResMut < LobbyForm >, mut keys : EventReader < KeyboardInput >) -> ()
```

### `lobby_reflect`

```rust
fn lobby_reflect(form : Res < LobbyForm >, mut q_nick_text : Query < & mut Text , (With < NickText > , Without < CodeText > , Without < StatusText >) >, mut q_code_text : Query < & mut Text , (With < CodeText > , Without < NickText > , Without < StatusText >) >, mut q_status : Query < & mut Text , (With < StatusText > , Without < NickText > , Without < CodeText >) >, mut q_nick_b : Query < & mut BorderColor , (With < NickField > , Without < CodeField > , Without < Swatch >) >, mut q_code_b : Query < & mut BorderColor , (With < CodeField > , Without < NickField > , Without < Swatch >) >, mut q_swatches : Query < (& Swatch , & mut BorderColor) , (Without < NickField > , Without < CodeField >) >) -> ()
```

### `age_str`

```rust
fn age_str(created_at : & str) -> String
```

### `detect_lan_ip`

```rust
fn detect_lan_ip() -> Option < String >
```

 Detecta o IP da LAN conectando um UDP socket a um endereço externo

 (sem enviar dados) e lendo o endereço local.

### `random_code`

```rust
fn random_code() -> String
```

## Systems (Bevy)

### `setup_lobby`

**Parâmetros**: `mut commands : Commands`, `assets : Res < GameAssets >`

### `cleanup_lobby`

**Parâmetros**: `mut commands : Commands`, `q : Query < Entity , With < LobbyRoot > >`

### `start_session`

**Parâmetros**: `gm : bool`, `join_code : Option < String >`, `signaling_override : Option < & str >`, `form : & mut LobbyForm`, `args : & CliArgs`, `net : & mut Net`, `roster : & mut Roster`, `commands : & mut Commands`, `next : & mut NextState < AppState >`

### `lobby_auto`

**Parâmetros**: `mut ran : Local < bool >`, `args : Res < CliArgs >`, `mut form : ResMut < LobbyForm >`, `mut net : ResMut < Net >`, `mut roster : ResMut < Roster >`, `mut commands : Commands`, `mut next : ResMut < NextState < AppState > >`

### `lobby_clicks`

**Parâmetros**: `mut form : ResMut < LobbyForm >`, `q_nick : Query < & Interaction , (Changed < Interaction > , With < NickField >) >`, `q_code : Query < & Interaction , (Changed < Interaction > , With < CodeField >) >`, `q_swatch : Query < (& Interaction , & Swatch) , Changed < Interaction > >`, `q_create : Query < & Interaction , (Changed < Interaction > , With < CreateBtn >) >`, `q_join : Query < & Interaction , (Changed < Interaction > , With < JoinBtn >) >`, `q_room_entry : Query < (& Interaction , & RoomEntryBtn) , Changed < Interaction > >`, `args : Res < CliArgs >`, `mut net : ResMut < Net >`, `mut roster : ResMut < Roster >`, `mut commands : Commands`, `mut next : ResMut < NextState < AppState > >`

### `room_poll`

**Parâmetros**: `mut list : ResMut < RoomList >`, `q_container : Query < Entity , With < RoomsContainer > >`, `q_entries : Query < Entity , With < RoomEntryBtn > >`, `q_empty : Query < Entity , With < RoomEmptyLabel > >`, `mut commands : Commands`, `time : Res < Time >`, `assets : Res < GameAssets >`

## Implementações

### `impl Default for RoomList`

- `default`

### `impl Plugin for LobbyPlugin`

- `build`

### `impl Default for LobbyForm`

- `default`

## Constantes

| Nome | Tipo | Valor |
|------|------|-------|
| `GOLD` | `Color` | `Color :: srgb (0.83 , 0.69 , 0.22)` |
| `PANEL` | `Color` | `Color :: srgba (0.10 , 0.09 , 0.14 , 0.96)` |
| `FIELD_BG` | `Color` | `Color :: srgb (0.16 , 0.14 , 0.21)` |
| `TEXT` | `Color` | `Color :: srgb (0.92 , 0.90 , 0.95)` |
| `MUTED` | `Color` | `Color :: srgb (0.58 , 0.55 , 0.66)` |
| `ROW_BG` | `Color` | `Color :: srgba (0.20 , 0.18 , 0.26 , 0.80)` |

