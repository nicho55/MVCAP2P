# Deploy em Celulares Físicos (GitHub Actions)

Guia prático para lançar uma versão do APK **direto nos celulares** e coletar
métricas de desempenho. A decisão de arquitetura está na **ADR-007**.

> **Por que precisa de um runner na sua máquina?** Os runners hospedados do
> GitHub ficam na nuvem e não enxergam seus celulares. O job de *deploy* roda
> num **self-hosted runner** instalado na máquina onde os aparelhos aparecem em
> `adb devices`.

## Visão geral

```
tag v* (ou botão manual)
  └─ job build   → ubuntu-latest (nuvem): compila APK, publica artifact
     └─ job deploy → self-hosted (sua máquina): adb install + medição
        └─ artifact perf-reports (fps / memória / screenshot por aparelho)
```

## Setup único: self-hosted runner

Feito **uma vez**, na máquina que tem os celulares conectados (fora de qualquer
container — o container não enxerga o USB).

1. GitHub → repo → **Settings → Actions → Runners → New self-hosted runner →
   Linux**. Copie os comandos que a página mostra (download + `config.sh`).
2. No `./config.sh`, **acrescente a label** exigida pelo workflow:

   ```bash
   ./config.sh --url https://github.com/<owner>/<repo> \
     --token <TOKEN_DA_PAGINA> \
     --labels android-farm --unattended
   ```

   > A label `android-farm` é obrigatória — o job de deploy usa
   > `runs-on: [self-hosted, android-farm]`. Sem ela, o job fica esperando.

3. Deixe o runner rodando. Preferir **serviço** (sobe sozinho no boot, reinicia
   se cair):

   ```bash
   sudo ./svc.sh install
   sudo ./svc.sh start
   sudo ./svc.sh status     # conferir
   ```

   Para teste rápido em primeiro plano (morre ao fechar o terminal):

   ```bash
   ./run.sh
   ```

   Sem systemd (ex.: WSL): `nohup ./run.sh &`.

Confirme em **Settings → Actions → Runners**: bolinha verde **Idle** + label
`android-farm`.

## Preparar os celulares

Na mesma máquina do runner:

1. Ative **Depuração USB** (Opções do desenvolvedor) em cada aparelho.
2. Conecte via USB e aceite "Confiar neste computador".
3. Confirme:

   ```bash
   adb devices     # todos precisam aparecer como "device"
   ```

   Sem `adb`: `sudo apt install -y android-tools-adb` (ou use `Sdk/platform-tools`).

### WiFi (sem cabo) — útil para aparelhos antigos

Com o aparelho no USB (Android < 11 usa o modo clássico):

```bash
adb -s <serial> tcpip 5555
adb connect 192.168.x.x:5555     # depois pode tirar o cabo
```

Guarde o IP numa **Repository variable** `J7_IP` (Settings → Secrets and
variables → Actions → Variables). O workflow reconecta automaticamente.

## Lançar uma versão

- **Manual:** aba **Actions → deploy-devices → Run workflow** (input opcional
  `launch_seconds` = tempo rodando em cada aparelho para medir fps).
- **Por versão:** `git tag v0.2.0 && git push origin v0.2.0`.

O job `build` compila na nuvem (~15–30 min na 1ª vez, com cache depois); o job
`deploy` instala e mede em todos os celulares. Baixe o artifact **perf-reports**
ao final.

## Ajustando qualidade gráfica no aparelho

Cada nova versão traz o painel **"Gráficos"** (canto superior direito, dentro da
partida). Ele liga/desliga em runtime as opções que mais pesam em GPU fraca —
**MSAA**, **sombras**, **HDR**, **árvores**, **grade** e **economia (30 fps)**.

Fluxo de certificação sugerido por aparelho: entre numa sala, abra **Gráficos**
e alterne uma opção por vez observando o `gfxinfo`/fps do relatório. Assim você
isola o custo de cada recurso no dispositivo real. Detalhes na **ADR-008**.

## Lendo os relatórios

Cada aparelho gera `MODELO_SERIAL.txt` + `.png` em `reports/`:

- `gfxinfo` — frames renderizados, janela de jitter, contagem de frames lentos.
- `meminfo` — memória total do processo.
- log recente — se o app caiu (comum com backend Vulkan em GPU antiga), aparece
  aqui; o script também marca falha no job.
- screenshot — prova visual de que a cena renderizou.

## Solução de problemas

| Sintoma | Causa provável | Ação |
|---|---|---|
| Job de deploy fica "Waiting for a runner" | runner parado ou sem a label | ver `svc.sh status`; reconfigurar com `--labels android-farm` |
| `Nenhum device conectado` | celular não autorizado/offline | `adb devices`; reautorizar no aparelho |
| `API < minSdk 24` no relatório | Android muito antigo (< 7.0) | rebaixar `minSdk` em `app/android/app/build.gradle.kts` |
| App abre e fecha (crash no log) | GPU sem Vulkan (ex.: J7 Mali-T830) | implementar fallback GLES no wgpu (passo 5) |
| `INSTALL_FAILED_UPDATE_INCOMPATIBLE` | assinatura diferente | o script já desinstala e reinstala sozinho |

## Referências

- ADR-007 — decisão de arquitetura
- `.github/workflows/deploy-devices.yml`
- `scripts/deploy-farm.sh`, `scripts/android-env.sh`
