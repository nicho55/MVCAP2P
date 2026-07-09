# `camera`

**Path**: `src/game/camera.rs`

## Resources (Bevy)

### `CamRig`

| Campo | Tipo |
|-------|------|
| `focus` | `Vec3` |
| `yaw` | `f32` |
| `pitch` | `f32` |
| `dist` | `f32` |

### `TouchState`

| Campo | Tipo |
|-------|------|
| `fingers` | `HashMap < u64 , Vec2 >` |
| `last_pinch` | `Option < f32 >` |

## Components (Bevy)

### `MainCamera`

## Funções

### `apply_rig`

```rust
fn apply_rig(rig : Res < CamRig >, mut q : Query < & mut Transform , With < MainCamera > >) -> ()
```

### `pan_zoom`

```rust
fn pan_zoom(mut wheel : EventReader < MouseWheel >, mut motion : EventReader < MouseMotion >, buttons : Res < ButtonInput < MouseButton > >, keys : Res < ButtonInput < KeyCode > >, time : Res < Time >, mut rig : ResMut < CamRig >, ui : Res < UiHovered >) -> ()
```

### `cursor_ray`

```rust
fn cursor_ray(win : & Window, cam : & Camera, cam_gt : & GlobalTransform) -> Option < Ray3d >
```

### `ray_ground`

```rust
fn ray_ground(ray : Ray3d, y : f32) -> Option < Vec3 >
```

### `cursor_ground`

```rust
fn cursor_ground(win : & Window, cam : & Camera, cam_gt : & GlobalTransform) -> Option < Vec2 >
```

### `ray_point_dist`

```rust
fn ray_point_dist(ray : & Ray3d, p : Vec3) -> f32
```

### `touch_pan_zoom`

```rust
fn touch_pan_zoom(mut touch_ev : EventReader < TouchInput >, mut state : ResMut < TouchState >, mut rig : ResMut < CamRig >, ui : Res < UiHovered >, drag : Res < TouchDrag >) -> ()
```

## Systems (Bevy)

### `setup_camera`

**Parâmetros**: `mut commands : Commands`

## Implementações

### `impl Default for impl Default for CamRig { fn default () -> Self { Self { focus : Vec3 :: ZERO , yaw : 0.0 , pitch : 0.95 , dist : 1400.0 } } } . self_ty`

- `default`

