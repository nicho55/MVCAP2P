# Scripts de Build & Deploy

| Script | Descrição |
|---|---|
| `build.sh` | Build completo: Linux desktop + Android APK + instala no celular via ADB |
| `android-driver.sh` | Build Android com retry automático e checkpoint |
| `android-env.sh` | Exporta variáveis de ambiente do Android SDK/NDK |

## build.sh

Builda ambos os targets e instala no celular em um comando:

```bash
./scripts/build.sh
```

Pré-requisitos:
- `cargo-ndk` instalado (`cargo install cargo-ndk`) — já incluso no Dockerfile
- Android SDK em `Sdk/` (já configurado no container)
- Celular conectado via USB com depuração USB ativada (opcional)
- Linker rápido: `mold` + `clang` (já instalados no container)
- Cache de compilação: `sccache` (já instalado e configurado)

## android-driver.sh

Build Android com detecção de dispositivo e retry automático.
Usado para deploy contínuo (CI/CD local).

```bash
./scripts/android-driver.sh
```

## Otimizações de build

O `.cargo/config.toml` já configura:
- **sccache** — cache de dependências entre builds
- **mold** — linker 5-10x mais rápido

O `Cargo.toml` já configura o perfil release para binários enxutos:
`lto = "thin"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`
