# `camera`

**Path**: `src/game/camera.rs`

## Resources (Bevy)

### `CamRig`

 Câmera orbital estilo RPG tático: foco no chão, yaw/pitch/distância.

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
fn pan_zoom(mut wheel : MessageReader < MouseWheel >, mut motion : MessageReader < MouseMotion >, buttons : Res < ButtonInput < MouseButton > >, keys : Res < ButtonInput < KeyCode > >, time : Res < Time >, mut rig : ResMut < CamRig >, ui : Res < UiHovered >) -> ()
```

### `cursor_ray`

```rust
fn cursor_ray(win : & Window, cam : & Camera, cam_gt : & GlobalTransform) -> Option < Ray3d >
```

 Raio do cursor no mundo.

### `ray_ground`

```rust
fn ray_ground(ray : Ray3d, y : f32) -> Option < Vec3 >
```

 Interseção do raio com o plano do chão (y = altura dada).

### `cursor_ground`

```rust
fn cursor_ground(win : & Window, cam : & Camera, cam_gt : & GlobalTransform) -> Option < Vec2 >
```

 Ponto do chão sob o cursor, como Vec2 = (x, z) — casa com a matemática do grid.

### `ray_point_dist`

```rust
fn ray_point_dist(ray : & Ray3d, p : Vec3) -> f32
```

 Distância mínima entre um raio e um ponto (para picking de tokens 3D).

### `touch_pan_zoom`

```rust
fn touch_pan_zoom(mut touch_ev : MessageReader < TouchInput >, mut state : ResMut < TouchState >, mut rig : ResMut < CamRig >, ui : Res < UiHovered >, drag : Res < TouchDrag >) -> ()
```

 Touch-based camera pan/zoom/orbit (Android).

 - 1 finger drag → pan (right-click equivalent)

 - 2 finger drag → orbit (middle-click equivalent)

 - pinch → zoom (scroll equivalent)

## Systems (Bevy)

### `setup_camera`

**Parâmetros**: `mut commands : Commands`

## Implementações

### `impl Default for CamRig`

- `default`

