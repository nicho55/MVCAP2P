# `lib`

**Path**: `src/lib.rs`

## Resources (Bevy)

### `CliArgs`

| Campo | Tipo |
|-------|------|
| `gm` | `bool` |
| `join` | `Option < String >` |
| `code` | `Option < String >` |
| `nick` | `Option < String >` |
| `color` | `Option < u8 >` |
| `map` | `Option < String >` |
| `demo` | `bool` |
| `signaling` | `Option < String >` |
| `shot` | `Option < String >` |
| `shot_at` | `f32` |
| `exit_at` | `Option < f32 >` |

## Enums

### `AppState`

**Derives**: States, Debug, Clone, PartialEq, Eq, Hash, Default

| Variante | Campos |
|----------|--------|
| `Boot` | `—` |
| `Lobby` | `—` |
| `InGame` | `—` |

## Funções

### `parse_args`

```rust
fn parse_args() -> CliArgs
```

### `run_game`

```rust
fn run_game() -> ()
```

### `main`

```rust
fn main() -> ()
```

### `boot_to_lobby`

```rust
fn boot_to_lobby(mut next : ResMut < NextState < AppState > >) -> ()
```

## Systems (Bevy)

### `screenshot_hotkey`

**Parâmetros**: `keys : Res < ButtonInput < KeyCode > >`, `mut commands : Commands`, `mut n : Local < u32 >`

### `auto_shot_exit`

**Parâmetros**: `time : Res < Time >`, `args : Res < CliArgs >`, `mut commands : Commands`, `mut done : Local < bool >`, `mut exit : EventWriter < AppExit >`

## Implementações

### `impl Default for impl Default for CliArgs { fn default () -> Self { Self { gm : false , join : None , code : None , nick : None , color : None , map : None , demo : false , signaling : None , shot : None , shot_at : 6.0 , exit_at : None , } } } . self_ty`

- `default`

