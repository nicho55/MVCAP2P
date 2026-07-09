# `room_discovery`

**Path**: `src/room_discovery.rs`

## Structs

### `RoomEntry`

**Derives**: Debug, Clone, Serialize, Deserialize

| Campo | Tipo |
|-------|------|
| `id` | `i64` |
| `code` | `String` |
| `gm_name` | `String` |
| `room_url` | `String` |
| `created_at` | `String` |

## Funções

### `list_rooms`

```rust
fn list_rooms() -> Result < Vec < RoomEntry > , String >
```

### `create_room`

```rust
fn create_room(code : & str, gm_name : & str, room_url : & str) -> Result < () , String >
```

### `delete_room`

```rust
fn delete_room(code : & str) -> Result < () , String >
```

## Constantes

| Nome | Tipo | Valor |
|------|------|-------|
| `SUPABASE_URL` | `& str` | `"https://cbnbmweqyezxejipisvf.supabase.co"` |
| `SUPABASE_ANON_KEY` | `& str` | `"sb_publishable_JY_scW17hysS_cUCpnhz-g_49N_YGxr"` |

