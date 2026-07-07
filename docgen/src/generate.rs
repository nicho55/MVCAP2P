use crate::extract::{CrateData, ModuleData};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Summary view of the crate for index navigation.
#[derive(Debug, Serialize)]
struct CrateSummary {
    name: String,
    modules: Vec<ModuleSummary>,
}

#[derive(Debug, Serialize)]
struct ModuleSummary {
    name: String,
    path: String,
    doc: Vec<String>,
    structs: Vec<String>,
    enums: Vec<String>,
    fns: Vec<String>,
    traits: Vec<String>,
    resources: Vec<String>,
    components: Vec<String>,
    systems: Vec<String>,
    events: Vec<String>,
    submodules: Vec<String>,
}

impl From<&ModuleData> for ModuleSummary {
    fn from(m: &ModuleData) -> Self {
        Self {
            name: m.name.clone(),
            path: m.path.clone(),
            doc: m.doc.clone(),
            structs: m.structs.iter().map(|s| s.name.clone()).collect(),
            enums: m.enums.iter().map(|e| e.name.clone()).collect(),
            fns: m.fns.iter().map(|f| f.name.clone()).collect(),
            traits: m.traits.iter().map(|t| t.name.clone()).collect(),
            resources: m.resources.clone(),
            components: m.components.clone(),
            systems: m.systems.clone(),
            events: m.events.clone(),
            submodules: m.submodules.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct DependencyGraph {
    nodes: Vec<DepNode>,
    edges: Vec<DepEdge>,
}

#[derive(Debug, Serialize)]
struct DepNode {
    id: String,
    label: String,
    module: String,
    kind: String,
}

#[derive(Debug, Serialize)]
struct DepEdge {
    source: String,
    target: String,
    kind: String,
}

pub fn generate_json(all_crates: &HashMap<String, CrateData>, out_dir: &str) -> Result<()> {
    let out = Path::new(out_dir);

    // Write per-module JSON
    for (crate_name, crate_data) in all_crates {
        let crate_dir = out.join(crate_name);
        fs::create_dir_all(&crate_dir)?;

        // Write the crate index
        let summary = CrateSummary {
            name: crate_name.clone(),
            modules: crate_data.modules.values().map(|m| ModuleSummary::from(m)).collect(),
        };
        let index_path = crate_dir.join("index.json");
        fs::write(&index_path, serde_json::to_string_pretty(&summary)?)?;

        // Write per-module data
        let mods_dir = crate_dir.join("modules");
        fs::create_dir_all(&mods_dir)?;

        for (mod_name, mod_data) in &crate_data.modules {
            let safe_name = mod_name.replace("::", "__");
            let mod_path = mods_dir.join(format!("{}.json", safe_name));
            fs::write(&mod_path, serde_json::to_string_pretty(&mod_data)?)?;
        }

        // Write dependency graph
        let graph = build_dependency_graph(crate_data);
        let graph_path = crate_dir.join("deps.json");
        fs::write(&graph_path, serde_json::to_string_pretty(&graph)?)?;
    }

    // Write a global index
    let global_index: Vec<(&String, &CrateData)> = all_crates.iter().collect();
    let global_path = out.join("_global.json");
    let global_summary: Vec<CrateSummary> = global_index
        .iter()
        .map(|(name, data)| CrateSummary {
            name: (*name).clone(),
            modules: data.modules.values().map(|m| ModuleSummary::from(m)).collect(),
        })
        .collect();
    fs::write(&global_path, serde_json::to_string_pretty(&global_summary)?)?;

    Ok(())
}

fn build_dependency_graph(crate_data: &CrateData) -> DependencyGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (mod_name, mod_data) in &crate_data.modules {
        nodes.push(DepNode {
            id: mod_name.clone(),
            label: mod_name.split("::").last().unwrap_or(mod_name).to_string(),
            module: mod_name.clone(),
            kind: "module".to_string(),
        });

        // Track use dependencies
        for use_line in &mod_data.uses {
            for (other_name, _) in &crate_data.modules {
                if other_name != mod_name && use_line.contains(other_name.replace("::", "__").as_str()) {
                    edges.push(DepEdge {
                        source: mod_name.clone(),
                        target: other_name.clone(),
                        kind: "import".to_string(),
                    });
                }
            }
        }

        // Submodule dependencies
        for sub in &mod_data.submodules {
            let sub_mod = format!("{}::{}", mod_name, sub);
            if crate_data.modules.contains_key(&sub_mod) {
                edges.push(DepEdge {
                    source: mod_name.clone(),
                    target: sub_mod,
                    kind: "contains".to_string(),
                });
            }
        }
    }

    DependencyGraph { nodes, edges }
}

pub fn generate_mdx(all_crates: &HashMap<String, CrateData>, out_dir: &str) -> Result<()> {
    let out = Path::new(out_dir);

    for (crate_name, crate_data) in all_crates {
        let crate_dir = out.join(crate_name);
        fs::create_dir_all(&crate_dir)?;

        // Write the crate overview
        let overview = generate_crate_overview_mdx(crate_name, crate_data);
        fs::write(crate_dir.join("index.mdx"), overview)?;

        // Write per-module mdx pages
        let mods_dir = crate_dir.join("modules");
        fs::create_dir_all(&mods_dir)?;

        // Also copy _meta.json for fumadocs navigation
        let meta = generate_meta_json(crate_name, crate_data);
        fs::write(mods_dir.join("_meta.json"), meta)?;

        for (mod_name, mod_data) in &crate_data.modules {
            let safe_name = mod_name.replace("::", "__");
            let mdx = generate_module_mdx(mod_name, mod_data);
            fs::write(mods_dir.join(format!("{}.mdx", safe_name)), mdx)?;
        }
    }

    // Write root meta for fumadocs
    let root_meta = serde_json::json!({
        "index": "Visão Geral da Arquitetura",
        "tabletop": {
            "type": "page",
            "display": "tabletop (jogo)"
        },
        "signaling": {
            "type": "page",
            "display": "signaling (sinalização)"
        }
    });
    fs::write(out.join("_meta.json"), serde_json::to_string_pretty(&root_meta)?)?;

    Ok(())
}

fn generate_crate_overview_mdx(crate_name: &str, data: &CrateData) -> String {
    let mut md = String::new();
    let title = format!("Crate: {}", crate_name);
    md.push_str(&format!("---\ntitle: \"{}\"\n---\n\n", title));
    md.push_str(&format!("# Crate: `{}`\n\n", crate_name));
    md.push_str("```mermaid\ngraph TD\n");
    md.push_str(&format!("  {}[\"{}\"]\n", crate_name, crate_name));
    for (mod_name, mod_data) in &data.modules {
        let label = mod_name.split("::").last().unwrap_or(mod_name);
        md.push_str(&format!("  {}[\"{}\"]\n", mod_name.replace("::", "_"), label));
        md.push_str(&format!("  {} --> {}\n", crate_name, mod_name.replace("::", "_")));
        for sub in &mod_data.submodules {
            let sub_mod = format!("{}::{}", mod_name, sub);
            if data.modules.contains_key(&sub_mod) {
                md.push_str(&format!(
                    "  {} --> {}\n",
                    mod_name.replace("::", "_"),
                    sub_mod.replace("::", "_")
                ));
            }
        }
    }
    md.push_str("```\n\n");

    md.push_str("## Módulos\n\n");
    for (mod_name, mod_data) in &data.modules {
        let label = mod_name.split("::").last().unwrap_or(mod_name);
        let safe = mod_name.replace("::", "__");
        md.push_str(&format!("### [`{}`](modules/{}) — `{}`\n\n", label, safe, mod_data.path));
        if !mod_data.doc.is_empty() {
            md.push_str(&format!("{}\n\n", mod_data.doc.join("\n")));
        }
        if !mod_data.resources.is_empty() {
            md.push_str(&format!("- **Resources**: {}\n", mod_data.resources.join(", ")));
        }
        if !mod_data.components.is_empty() {
            md.push_str(&format!("- **Components**: {}\n", mod_data.components.join(", ")));
        }
        if !mod_data.events.is_empty() {
            md.push_str(&format!("- **Events**: {}\n", mod_data.events.join(", ")));
        }
        if !mod_data.systems.is_empty() {
            md.push_str(&format!("- **Systems**: {}\n", mod_data.systems.join(", ")));
        }
        if !mod_data.structs.is_empty() {
            md.push_str(&format!(
                "- **Structs**: {}\n",
                mod_data.structs.iter().map(|s| format!("`{}`", s.name)).collect::<Vec<_>>().join(", ")
            ));
        }
        if !mod_data.enums.is_empty() {
            md.push_str(&format!(
                "- **Enums**: {}\n",
                mod_data.enums.iter().map(|e| format!("`{}`", e.name)).collect::<Vec<_>>().join(", ")
            ));
        }
        md.push('\n');
    }

    md
}

fn generate_meta_json(_crate_name: &str, data: &CrateData) -> String {
    let mut meta: HashMap<String, serde_json::Value> = HashMap::new();
    meta.insert("index".to_string(), serde_json::json!("Visão Geral"));
    for (mod_name, m) in &data.modules {
        let label = mod_name.split("::").last().unwrap_or(mod_name);
        if m.submodules.is_empty() {
            meta.insert(
                mod_name.replace("::", "__"),
                serde_json::json!(format!("`{}` — {} estruturas, {} sistemas", label, m.structs.len(), m.systems.len())),
            );
        } else {
            meta.insert(
                mod_name.replace("::", "__"),
                serde_json::json!({"display": label, "type": "folder"}),
            );
        }
    }
    serde_json::to_string_pretty(&meta).unwrap()
}

fn generate_module_mdx(mod_name: &str, data: &ModuleData) -> String {
    let mut md = String::new();
    let label = mod_name.split("::").last().unwrap_or(mod_name);

    md.push_str(&format!("---\ntitle: \"{}\"\n---\n\n", label));
    md.push_str(&format!("# `{}`\n\n", label));
    md.push_str(&format!("**Path**: `{}`\n\n", data.path));

    if !data.doc.is_empty() {
        md.push_str("## Descrição\n\n");
        for line in &data.doc {
            md.push_str(&format!("{}\n\n", line));
        }
    }

    // Resources
    if !data.resources.is_empty() {
        md.push_str("## Resources (Bevy)\n\n");
        for r in &data.resources {
            let info = data.structs.iter().find(|s| s.name == *r);
            if let Some(s) = info {
                md.push_str(&format!("### `{}`\n\n", r));
                if !s.doc.is_empty() {
                    for d in &s.doc {
                        md.push_str(&format!("{}\n\n", d));
                    }
                }
                if !s.fields.is_empty() {
                    md.push_str("| Campo | Tipo |\n|-------|------|\n");
                    for f in &s.fields {
                        md.push_str(&format!("| `{}` | `{}` |\n", f.name, f.ty));
                    }
                    md.push('\n');
                }
            }
        }
    }

    // Events
    if !data.events.is_empty() {
        md.push_str("## Events (Bevy)\n\n");
        for e in &data.events {
            let info = data.structs.iter().find(|s| s.name == *e);
            if let Some(s) = info {
                md.push_str(&format!("### `{}`\n\n", e));
                if !s.doc.is_empty() {
                    for d in &s.doc {
                        md.push_str(&format!("{}\n\n", d));
                    }
                }
                if !s.fields.is_empty() {
                    md.push_str("| Campo | Tipo |\n|-------|------|\n");
                    for f in &s.fields {
                        md.push_str(&format!("| `{}` | `{}` |\n", f.name, f.ty));
                    }
                    md.push('\n');
                }
            }
        }
    }

    // Components
    if !data.components.is_empty() {
        md.push_str("## Components (Bevy)\n\n");
        for c in &data.components {
            let info = data.structs.iter().find(|s| s.name == *c);
            if let Some(s) = info {
                md.push_str(&format!("### `{}`\n\n", c));
                if !s.doc.is_empty() {
                    for d in &s.doc {
                        md.push_str(&format!("{}\n\n", d));
                    }
                }
                if !s.fields.is_empty() {
                    md.push_str("| Campo | Tipo |\n|-------|------|\n");
                    for f in &s.fields {
                        md.push_str(&format!("| `{}` | `{}` |\n", f.name, f.ty));
                    }
                    md.push('\n');
                }
            }
        }
    }

    // Structs (non-resource/component/event)
    let non_special: Vec<_> = data
        .structs
        .iter()
        .filter(|s| {
            !data.resources.contains(&s.name)
                && !data.components.contains(&s.name)
                && !data.events.contains(&s.name)
        })
        .collect();
    if !non_special.is_empty() {
        md.push_str("## Structs\n\n");
        for s in &non_special {
            md.push_str(&format!("### `{}`\n\n", s.name));
            if !s.doc.is_empty() {
                for d in &s.doc {
                    md.push_str(&format!("{}\n\n", d));
                }
            }
            md.push_str(&format!("**Derives**: {}\n\n", s.derives.join(", ")));
            if !s.fields.is_empty() {
                md.push_str("| Campo | Tipo |\n|-------|------|\n");
                for f in &s.fields {
                    md.push_str(&format!("| `{}` | `{}` |\n", f.name, f.ty));
                }
                md.push('\n');
            }
        }
    }

    // Enums
    if !data.enums.is_empty() {
        md.push_str("## Enums\n\n");
        for e in &data.enums {
            md.push_str(&format!("### `{}`\n\n", e.name));
            if !e.doc.is_empty() {
                for d in &e.doc {
                    md.push_str(&format!("{}\n\n", d));
                }
            }
            if !e.derives.is_empty() {
                md.push_str(&format!("**Derives**: {}\n\n", e.derives.join(", ")));
            }
            md.push_str("| Variante | Campos |\n|----------|--------|\n");
            for v in &e.variants {
                let fields = if v.fields.is_empty() {
                    "—".to_string()
                } else {
                    v.fields.join(", ")
                };
                md.push_str(&format!("| `{}` | `{}` |\n", v.name, fields));
            }
            md.push('\n');
        }
    }

    // Public functions (non-system)
    let systems_set: std::collections::HashSet<_> = data.systems.iter().cloned().collect();
    let non_sys_fns: Vec<_> = data
        .fns
        .iter()
        .filter(|f| !systems_set.contains(&f.name))
        .collect();
    if !non_sys_fns.is_empty() {
        md.push_str("## Funções\n\n");
        for f in &non_sys_fns {
            md.push_str(&format!("### `{}`\n\n", f.name));
            md.push_str(&format!("```rust\nfn {}({}) -> {}\n```\n\n", f.name, f.inputs.join(", "), f.output));
            if !f.doc.is_empty() {
                for d in &f.doc {
                    md.push_str(&format!("{}\n\n", d));
                }
            }
        }
    }

    // Systems
    if !data.systems.is_empty() {
        md.push_str("## Systems (Bevy)\n\n");
        for s_name in &data.systems {
            let info = data.fns.iter().find(|f| f.name == *s_name);
            if let Some(f) = info {
                md.push_str(&format!("### `{}`\n\n", s_name));
                if !f.doc.is_empty() {
                    for d in &f.doc {
                        md.push_str(&format!("{}\n\n", d));
                    }
                }
                md.push_str(&format!(
                    "**Parâmetros**: {}\n\n",
                    f.inputs
                        .iter()
                        .map(|s| format!("`{}`", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    // Impl blocks
    if !data.impls.is_empty() {
        md.push_str("## Implementações\n\n");
        for i in &data.impls {
            if let Some(trait_name) = &i.trait_name {
                md.push_str(&format!("### `impl {} for {}`\n\n", trait_name, i.ty));
            } else {
                md.push_str(&format!("### `impl {}`\n\n", i.ty));
            }
            for m in &i.methods {
                md.push_str(&format!("- `{}`\n", m.name));
            }
            md.push('\n');
        }
    }

    // Constants
    if !data.consts.is_empty() {
        md.push_str("## Constantes\n\n");
        md.push_str("| Nome | Tipo | Valor |\n|------|------|-------|\n");
        for c in &data.consts {
            md.push_str(&format!("| `{}` | `{}` | `{}` |\n", c.name, c.ty, c.value));
        }
        md.push('\n');
    }

    // Type aliases
    if !data.types.is_empty() {
        md.push_str("## Type Aliases\n\n");
        md.push_str("| Nome | Tipo |\n|------|------|\n");
        for t in &data.types {
            md.push_str(&format!("| `{}` | `{}` |\n", t.name, t.ty));
        }
        md.push('\n');
    }

    // Specific Mermaid diagrams for key modules
    if let Some(diagram) = generate_mermaid_for_module(mod_name) {
        md.push_str(&diagram);
    }

    md
}

fn generate_mermaid_for_module(mod_name: &str) -> Option<String> {
    let label = mod_name.split("::").last().unwrap_or(mod_name);
    match label {
        "net" => Some(generate_net_flow_diagram()),
        "sync" => Some(generate_sync_flow_diagram()),
        "tokens" => Some(generate_tokens_flow_diagram()),
        _ => None,
    }
}

fn generate_net_flow_diagram() -> String {
    let mut md = String::new();
    md.push_str("\n## Fluxo de Conexão WebRTC\n\n");
    md.push_str("```mermaid\n");
    md.push_str("sequenceDiagram\n");
    md.push_str("    participant P as Jogador\n");
    md.push_str("    participant S as Signaling Server\n");
    md.push_str("    participant G as GM (Mestre)\n\n");
    md.push_str("    Note over P,G: 1. Sessão criada no Lobby\n");
    md.push_str("    P->>S: WebSocket connect ws://host/tabletop_{código}\n");
    md.push_str("    G->>S: WebSocket connect ws://host/tabletop_{código}\n\n");
    md.push_str("    Note over P,G: 2. Handshake WebRTC (mediado pelo signaling)\n");
    md.push_str("    S->>P: SDP Offer\n");
    md.push_str("    P->>S: SDP Answer\n");
    md.push_str("    S->>G: ICE Candidates\n");
    md.push_str("    Note over P,G: Conexão P2P direta (canal 0, confiável)\n\n");
    md.push_str("    Note over P,G: 3. Matchbox detecta peers\n");
    md.push_str("    G->>P: PeerState::Connected\n");
    md.push_str("    P->>G: PeerState::Connected\n");
    md.push_str("```\n\n");
    md
}

fn generate_sync_flow_diagram() -> String {
    let mut md = String::new();
    md.push_str("\n## Fluxo de Sincronização (Hello → Welcome)\n\n");
    md.push_str("```mermaid\n");
    md.push_str("sequenceDiagram\n");
    md.push_str("    participant P as Jogador\n");
    md.push_str("    participant G as GM (Mestre)\n\n");
    md.push_str("    Note over P,G: Peer conectado\n");
    md.push_str("    P->>G: Msg::Hello(PlayerMeta)\n\n");
    md.push_str("    rect rgb(200, 230, 255)\n");
    md.push_str("        Note over G: handle_hello()\n");
    md.push_str("        G->>G: roster.upsert(meta, peer)\n");
    md.push_str("        G->>P: Msg::PlayerJoined(meta)\n");
    md.push_str("        G->>P: Msg::BlobStart + N×BlobChunk (mapa + arte)\n");
    md.push_str("        G->>P: Msg::Welcome { players, grid, terrain, tokens, map_blob }\n");
    md.push_str("    end\n\n");
    md.push_str("    rect rgb(230, 255, 230)\n");
    md.push_str("        Note over P: handle_core() + handle_tokens()\n");
    md.push_str("        P->>P: net.gm_peer = Some(peer_id)\n");
    md.push_str("        P->>P: Carregar roster, grid, terrain\n");
    md.push_str("        P->>P: Spawnar tokens do Welcome\n");
    md.push_str("    end\n");
    md.push_str("```\n\n");
    md
}

fn generate_tokens_flow_diagram() -> String {
    let mut md = String::new();
    md.push_str("\n## Fluxo de Arrasto de Token\n\n");
    md.push_str("```mermaid\n");
    md.push_str("sequenceDiagram\n");
    md.push_str("    participant U as Usuário\n");
    md.push_str("    participant M as Máquina (Local)\n");
    md.push_str("    participant N as Rede (Peers)\n\n");
    md.push_str("    Note over U,N: PRESSÃO DO MOUSE\n");
    md.push_str("    U->>M: Clique esquerdo (just_pressed)\n");
    md.push_str("    M->>M: Ray cast → hit test nos tokens\n");
    md.push_str("    alt Token encontrado & sou dono/GM\n");
    md.push_str("        M->>M: sel.0 = token_id\n");
    md.push_str("        M->>M: drag.id = token_id\n");
    md.push_str("        M->>M: drag.grab = offset\n");
    md.push_str("    else Token encontrado & não sou dono\n");
    md.push_str("        M->>M: sel.0 = token_id (selecionar apenas)\n");
    md.push_str("    else Nenhum token\n");
    md.push_str("        M->>M: sel.0 = None (deselect)\n");
    md.push_str("    end\n\n");
    md.push_str("    Note over U,N: ARRASTO\n");
    md.push_str("    U->>M: Botão esquerdo segurado\n");
    md.push_str("    M->>M: cursor_ground() → pos\n");
    md.push_str("    M->>M: lift = altura_terreno + offset\n");
    md.push_str("    M->>M: tf.translation = (pos, lift)\n");
    md.push_str("    alt A cada 50ms (throttled)\n");
    md.push_str("        M->>N: Msg::DragPreview { id, x, y }\n");
    md.push_str("    end\n\n");
    md.push_str("    Note over U,N: SOLTAR\n");
    md.push_str("    U->>M: Botão esquerdo (just_released)\n");
    md.push_str("    M->>M: drag.id = None\n");
    md.push_str("    M->>M: Snap para centro da célula\n");
    md.push_str("    alt Eu sou GM\n");
    md.push_str("        M->>N: Msg::TokenMoved { id, cell }\n");
    md.push_str("    else Eu sou jogador\n");
    md.push_str("        M->>N: Msg::MoveTokenReq { id, cell }\n");
    md.push_str("        N->>M: (depois) Msg::TokenMoved { id, cell }\n");
    md.push_str("    end\n");
    md.push_str("```\n\n");
    md
}