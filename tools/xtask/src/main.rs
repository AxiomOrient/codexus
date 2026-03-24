use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const COMMON_RS: &str = "crates/codexus-core/protocol-inputs/openai/codex/527244910fb851cea6147334dbc08f8fbce4cb9d/codex-rs/app-server-protocol/src/protocol/common.rs";
const V2_RS: &str = "crates/codexus-core/protocol-inputs/openai/codex/527244910fb851cea6147334dbc08f8fbce4cb9d/codex-rs/app-server-protocol/src/protocol/v2.rs";
const SOURCE_REVISION: &str = "openai/codex@527244910fb851cea6147334dbc08f8fbce4cb9d";

fn main() -> Result<(), String> {
    let task = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "protocol-codegen".to_owned());
    match task.as_str() {
        "protocol-codegen" => generate_protocol(),
        "protocol-codegen-plan" => print_protocol_plan(),
        other => Err(format!("unknown xtask: {other}")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Surface {
    ClientRequest,
    ServerRequest,
    ServerNotification,
    ClientNotification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stability {
    Stable,
    Experimental,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeatureClass {
    Core,
    Experimental,
}

#[derive(Clone, Debug)]
struct Entry {
    rust_name: String,
    wire_name: String,
    surface: Surface,
    stability: Stability,
    feature: FeatureClass,
}

struct SnapshotInput {
    common_rs: String,
    v2_rs: String,
}

struct ProtocolTables {
    source_hash: String,
    client_requests: Vec<Entry>,
    server_requests: Vec<Entry>,
    server_notifications: Vec<Entry>,
    client_notifications: Vec<Entry>,
}

struct PlannedFile {
    relative_path: &'static str,
    contents: String,
}

fn generate_protocol() -> Result<(), String> {
    let repo_root = find_repo_root()?;
    let snapshot = load_snapshot(&repo_root)?;
    let tables = parse_snapshot(&snapshot)?;
    let plan = plan_outputs(&tables);
    apply_outputs(&repo_root, &plan)
}

fn print_protocol_plan() -> Result<(), String> {
    let repo_root = find_repo_root()?;
    let snapshot = load_snapshot(&repo_root)?;
    let tables = parse_snapshot(&snapshot)?;
    for file in plan_outputs(&tables) {
        println!("{}\t{} bytes", file.relative_path, file.contents.len());
    }
    Ok(())
}

fn find_repo_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|path| path.join("Cargo.toml").is_file() && path.join("crates").is_dir())
        .map(Path::to_path_buf)
        .ok_or_else(|| "failed to locate workspace root from xtask manifest directory".to_owned())
}

fn compute_source_hash(common_rs: &str, v2_rs: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(common_rs.as_bytes());
    hasher.update(b"\n--v2--\n");
    hasher.update(v2_rs.as_bytes());
    hex::encode(hasher.finalize())
}

fn load_snapshot(repo_root: &Path) -> Result<SnapshotInput, String> {
    let common_rs = fs::read_to_string(repo_root.join(COMMON_RS)).map_err(|err| err.to_string())?;
    let v2_rs = fs::read_to_string(repo_root.join(V2_RS)).map_err(|err| err.to_string())?;
    Ok(SnapshotInput { common_rs, v2_rs })
}

fn write_file(path: &Path, contents: String) -> Result<(), String> {
    fs::write(path, contents).map_err(|err| format!("write {}: {err}", path.display()))
}

fn parse_snapshot(input: &SnapshotInput) -> Result<ProtocolTables, String> {
    Ok(ProtocolTables {
        source_hash: compute_source_hash(&input.common_rs, &input.v2_rs),
        client_requests: parse_entries(
            &input.common_rs,
            "client_request_definitions!",
            Surface::ClientRequest,
        )?,
        server_requests: parse_entries(
            &input.common_rs,
            "server_request_definitions!",
            Surface::ServerRequest,
        )?,
        server_notifications: parse_entries(
            &input.common_rs,
            "server_notification_definitions!",
            Surface::ServerNotification,
        )?,
        client_notifications: parse_entries(
            &input.common_rs,
            "client_notification_definitions!",
            Surface::ClientNotification,
        )?,
    })
}

fn plan_outputs(tables: &ProtocolTables) -> Vec<PlannedFile> {
    vec![
        PlannedFile {
            relative_path: "crates/codexus-core/src/protocol/generated/client_requests.rs",
            contents: render_surface_module(&tables.client_requests),
        },
        PlannedFile {
            relative_path: "crates/codexus-core/src/protocol/generated/server_requests.rs",
            contents: render_surface_module(&tables.server_requests),
        },
        PlannedFile {
            relative_path: "crates/codexus-core/src/protocol/generated/server_notifications.rs",
            contents: render_surface_module(&tables.server_notifications),
        },
        PlannedFile {
            relative_path: "crates/codexus-core/src/protocol/generated/client_notifications.rs",
            contents: render_surface_module(&tables.client_notifications),
        },
        PlannedFile {
            relative_path: "crates/codexus-core/src/protocol/generated/methods.rs",
            contents: render_methods_module(
                &tables.client_requests,
                &tables.server_requests,
                &tables.server_notifications,
                &tables.client_notifications,
            ),
        },
        PlannedFile {
            relative_path: "crates/codexus-core/src/protocol/generated/inventory.rs",
            contents: render_inventory_module(&tables.source_hash),
        },
        PlannedFile {
            relative_path: "crates/codexus-core/src/protocol/generated/validators.rs",
            contents: render_validators_module(),
        },
    ]
}

fn apply_outputs(repo_root: &Path, plan: &[PlannedFile]) -> Result<(), String> {
    let mut written_paths = Vec::with_capacity(plan.len());
    for file in plan {
        let path = repo_root.join(file.relative_path);
        write_file(&path, file.contents.clone())?;
        written_paths.push(path);
    }
    run_rustfmt(&written_paths)?;
    Ok(())
}

fn run_rustfmt(paths: &[PathBuf]) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }

    let status = Command::new("rustfmt")
        .args(paths)
        .status()
        .map_err(|err| format!("rustfmt generated files: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "rustfmt generated files exited with status {status}"
        ))
    }
}

fn parse_entries(source: &str, macro_name: &str, surface: Surface) -> Result<Vec<Entry>, String> {
    let block = extract_macro_block(source, macro_name)?;
    let mut entries = Vec::new();
    let mut experimental = false;
    let mut pending_serde_rename: Option<String> = None;

    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("///") {
            continue;
        }
        if trimmed.starts_with("#[") && !trimmed.starts_with("#[experimental(") {
            if let Some(rename) = parse_serde_rename(trimmed) {
                pending_serde_rename = Some(rename.to_owned());
            }
            continue;
        }
        if trimmed.starts_with("#[experimental(") {
            experimental = true;
            continue;
        }

        if let Some(entry) = parse_macro_entry(trimmed, surface, experimental) {
            if include_entry(&entry.wire_name) {
                entries.push(entry);
            }
            experimental = false;
            continue;
        }

        if surface == Surface::ClientNotification {
            if let Some(entry) = parse_simple_notification_entry(trimmed, surface, experimental) {
                if include_entry(&entry.wire_name) {
                    entries.push(entry);
                }
                experimental = false;
                continue;
            }
        }

        if let Some(wire_name) = pending_serde_rename.as_deref() {
            if let Some(entry) =
                parse_renamed_variant_entry(trimmed, surface, wire_name, experimental)
            {
                if include_entry(&entry.wire_name) {
                    entries.push(entry);
                }
                experimental = false;
                pending_serde_rename = None;
            }
        }
    }

    if surface == Surface::ClientRequest {
        entries.insert(
            0,
            Entry {
                rust_name: "Initialize".to_owned(),
                wire_name: "initialize".to_owned(),
                surface,
                stability: Stability::Stable,
                feature: FeatureClass::Core,
            },
        );
    }

    if surface == Surface::ClientNotification
        && !entries.iter().any(|entry| entry.rust_name == "Initialized")
    {
        entries.push(build_entry(
            "Initialized".to_owned(),
            "initialized".to_owned(),
            surface,
            false,
        ));
    }

    Ok(entries)
}

fn include_entry(wire_name: &str) -> bool {
    !matches!(wire_name, "rawResponseItem/completed" | "thread/compacted")
}

fn extract_macro_block<'a>(source: &'a str, macro_name: &str) -> Result<&'a str, String> {
    let start = source
        .find(macro_name)
        .ok_or_else(|| format!("missing macro block: {macro_name}"))?;
    let after = &source[start..];
    let open_offset = after
        .find('{')
        .ok_or_else(|| format!("missing opening brace for {macro_name}"))?;
    let mut depth = 0usize;
    let mut body_start = None;
    for (offset, ch) in after[open_offset..].char_indices() {
        match ch {
            '{' => {
                depth += 1;
                if depth == 1 {
                    body_start = Some(open_offset + offset + 1);
                }
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let start_idx = body_start.ok_or("block start missing")?;
                    let end_idx = open_offset + offset;
                    return Ok(&after[start_idx..end_idx]);
                }
            }
            _ => {}
        }
    }
    Err(format!("unclosed macro block: {macro_name}"))
}

fn parse_serde_rename(trimmed: &str) -> Option<&str> {
    let prefix = "#[serde(rename = \"";
    let suffix = "\")]";
    trimmed
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
}

fn parse_macro_entry(trimmed: &str, surface: Surface, experimental: bool) -> Option<Entry> {
    let (rust_name, tail) = trimmed.split_once("=>")?;
    let rust_name = rust_name.trim().to_owned();
    let wire_start = tail.find('"')?;
    let wire_tail = &tail[wire_start + 1..];
    let wire_end = wire_tail.find('"')?;
    let wire_name = wire_tail[..wire_end].to_owned();
    Some(build_entry(rust_name, wire_name, surface, experimental))
}

fn parse_renamed_variant_entry(
    trimmed: &str,
    surface: Surface,
    wire_name: &str,
    experimental: bool,
) -> Option<Entry> {
    if trimmed.starts_with('#') {
        return None;
    }
    let rust_name = trimmed.split_once('(')?.0.trim();
    if rust_name.is_empty() {
        return None;
    }
    Some(build_entry(
        rust_name.to_owned(),
        wire_name.to_owned(),
        surface,
        experimental,
    ))
}

fn parse_simple_notification_entry(
    trimmed: &str,
    surface: Surface,
    experimental: bool,
) -> Option<Entry> {
    let rust_name = trimmed.strip_suffix(',')?.trim();
    if rust_name.is_empty()
        || !rust_name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(build_entry(
        rust_name.to_owned(),
        rust_name.to_ascii_lowercase(),
        surface,
        experimental,
    ))
}

fn build_entry(
    rust_name: String,
    wire_name: String,
    surface: Surface,
    experimental: bool,
) -> Entry {
    let (stability, feature) = if experimental {
        (Stability::Experimental, FeatureClass::Experimental)
    } else {
        (Stability::Stable, FeatureClass::Core)
    };
    Entry {
        rust_name,
        wire_name,
        surface,
        stability,
        feature,
    }
}

fn render_surface_module(entries: &[Entry]) -> String {
    let mut out = String::new();
    out.push_str("use serde_json::Value;\n\n");
    out.push_str("use super::types::*;\n\n");
    writeln!(
        &mut out,
        "macro_rules! define_{}_specs {{",
        module_name(entries.first().expect("entries present").surface)
    )
    .unwrap();
    out.push_str(
        "    ($($name:ident => $wire:literal, $stability:ident, $feature:ident),* $(,)?) => {\n",
    );
    out.push_str("        $(\n");
    out.push_str("            pub struct $name;\n\n");
    out.push_str("            impl $name {\n");
    out.push_str("                pub const METHOD: &'static str = $wire;\n");
    out.push_str("                pub const META: MethodMeta = MethodMeta::new(\n");
    out.push_str("                    stringify!($name),\n");
    out.push_str("                    $wire,\n");
    writeln!(
        &mut out,
        "                    MethodSurface::{},",
        method_surface_name(entries[0].surface)
    )
    .unwrap();
    out.push_str("                    Stability::$stability,\n");
    out.push_str("                    FeatureClass::$feature,\n");
    out.push_str("                    \"serde_json::Value\",\n");
    if matches!(
        entries[0].surface,
        Surface::ClientRequest | Surface::ServerRequest
    ) {
        out.push_str("                    Some(\"serde_json::Value\"),\n");
    } else {
        out.push_str("                    None,\n");
    }
    out.push_str("                );\n");
    out.push_str("            }\n\n");
    out.push_str("            impl MethodSpec for $name {\n");
    out.push_str("                const META: MethodMeta = $name::META;\n");
    out.push_str("            }\n\n");
    match entries[0].surface {
        Surface::ClientRequest => {
            out.push_str("            impl ClientRequestSpec for $name {\n");
            out.push_str("                type Params = Value;\n");
            out.push_str("                type Response = Value;\n");
            out.push_str("            }\n");
        }
        Surface::ServerRequest => {
            out.push_str("            impl ServerRequestSpec for $name {\n");
            out.push_str("                type Params = Value;\n");
            out.push_str("                type Response = Value;\n");
            out.push_str("            }\n");
        }
        Surface::ServerNotification => {
            out.push_str("            impl ServerNotificationSpec for $name {\n");
            out.push_str("                type Params = Value;\n");
            out.push_str("            }\n");
        }
        Surface::ClientNotification => {
            out.push_str("            impl ClientNotificationSpec for $name {\n");
            out.push_str("                type Params = Value;\n");
            out.push_str("            }\n");
        }
    }
    out.push_str("        )*\n\n");
    out.push_str("        pub const SPECS: &[MethodMeta] = &[\n");
    out.push_str("            $( $name::META, )*\n");
    out.push_str("        ];\n");
    out.push_str("    };\n");
    out.push_str("}\n\n");

    writeln!(
        &mut out,
        "define_{}_specs! {{",
        module_name(entries[0].surface)
    )
    .unwrap();
    for entry in entries {
        writeln!(
            &mut out,
            "    {} => \"{}\", {}, {},",
            entry.rust_name,
            entry.wire_name,
            stability_name(entry.stability),
            feature_name(entry.feature)
        )
        .unwrap();
    }
    out.push_str("}\n");
    out
}

fn render_methods_module(
    client_requests: &[Entry],
    server_requests: &[Entry],
    server_notifications: &[Entry],
    client_notifications: &[Entry],
) -> String {
    let mut out = String::new();
    for entry in client_requests
        .iter()
        .chain(server_requests)
        .chain(server_notifications)
        .chain(client_notifications)
    {
        writeln!(
            &mut out,
            "pub const {}: &str = super::{}::{}::METHOD;",
            constant_name(&entry.wire_name),
            module_path(entry.surface),
            entry.rust_name
        )
        .unwrap();
    }
    out.push_str("\n");
    out.push_str("/// Internal approval-ack wire method: runtime response to server-request approval cycle.\n");
    out.push_str("pub const APPROVAL_ACK: &str = \"approval/ack\";\n\n");
    out.push_str(
        "/// Turn lifecycle terminal-state notifications kept outside generated server notifications.\n",
    );
    out.push_str("pub const TURN_FAILED: &str = \"turn/failed\";\n");
    out.push_str("pub const TURN_CANCELLED: &str = \"turn/cancelled\";\n");
    out.push_str("pub const TURN_INTERRUPTED: &str = \"turn/interrupted\";\n");
    out
}

fn render_inventory_module(source_hash: &str) -> String {
    let mut out = String::new();
    out.push_str("use std::sync::OnceLock;\n\n");
    out.push_str("use super::client_notifications;\n");
    out.push_str("use super::client_requests;\n");
    out.push_str("use super::server_notifications;\n");
    out.push_str("use super::server_requests;\n");
    out.push_str("use super::types::*;\n\n");
    writeln!(
        &mut out,
        "pub const SOURCE_REVISION: &str = \"{SOURCE_REVISION}\";"
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const SOURCE_HASH: &str = \"{source_hash}\";\n"
    )
    .unwrap();
    out.push_str("pub const CLIENT_REQUESTS: &[MethodMeta] = client_requests::SPECS;\n");
    out.push_str("pub const SERVER_REQUESTS: &[MethodMeta] = server_requests::SPECS;\n");
    out.push_str("pub const SERVER_NOTIFICATIONS: &[MethodMeta] = server_notifications::SPECS;\n");
    out.push_str(
        "pub const CLIENT_NOTIFICATIONS: &[MethodMeta] = client_notifications::SPECS;\n\n",
    );
    out.push_str("static ALL_METHODS: OnceLock<&'static [MethodMeta]> = OnceLock::new();\n");
    out.push_str("static PROTOCOL_INVENTORY: OnceLock<ProtocolInventory> = OnceLock::new();\n\n");
    out.push_str("fn build_all_methods() -> &'static [MethodMeta] {\n");
    out.push_str("    ALL_METHODS.get_or_init(|| {\n");
    out.push_str("        let mut all = Vec::with_capacity(\n");
    out.push_str("            CLIENT_REQUESTS.len()\n");
    out.push_str("                + SERVER_REQUESTS.len()\n");
    out.push_str("                + SERVER_NOTIFICATIONS.len()\n");
    out.push_str("                + CLIENT_NOTIFICATIONS.len(),\n");
    out.push_str("        );\n");
    out.push_str("        all.extend_from_slice(CLIENT_REQUESTS);\n");
    out.push_str("        all.extend_from_slice(SERVER_REQUESTS);\n");
    out.push_str("        all.extend_from_slice(SERVER_NOTIFICATIONS);\n");
    out.push_str("        all.extend_from_slice(CLIENT_NOTIFICATIONS);\n");
    out.push_str("        Box::leak(all.into_boxed_slice())\n");
    out.push_str("    })\n");
    out.push_str("}\n\n");
    out.push_str("pub fn inventory() -> &'static ProtocolInventory {\n");
    out.push_str("    PROTOCOL_INVENTORY.get_or_init(|| ProtocolInventory {\n");
    out.push_str("        source_revision: SOURCE_REVISION,\n");
    out.push_str("        source_hash: SOURCE_HASH,\n");
    out.push_str("        all_methods: build_all_methods(),\n");
    out.push_str("        client_requests: CLIENT_REQUESTS,\n");
    out.push_str("        server_requests: SERVER_REQUESTS,\n");
    out.push_str("        server_notifications: SERVER_NOTIFICATIONS,\n");
    out.push_str("        client_notifications: CLIENT_NOTIFICATIONS,\n");
    out.push_str("    })\n");
    out.push_str("}\n");
    out
}

fn render_validators_module() -> String {
    let mut out = String::new();
    out.push_str(
        "use crate::protocol::generated::inventory::{\n    CLIENT_REQUESTS, SERVER_NOTIFICATIONS, SERVER_REQUESTS,\n};\n\n",
    );
    out.push_str("pub fn is_known_client_request(method: &str) -> bool {\n");
    out.push_str("    CLIENT_REQUESTS.iter().any(|meta| meta.wire_name == method)\n");
    out.push_str("}\n\n");
    out.push_str("pub fn is_known_server_request(method: &str) -> bool {\n");
    out.push_str("    SERVER_REQUESTS.iter().any(|meta| meta.wire_name == method)\n");
    out.push_str("}\n\n");
    out.push_str("pub fn is_known_server_notification(method: &str) -> bool {\n");
    out.push_str("    SERVER_NOTIFICATIONS\n");
    out.push_str("        .iter()\n");
    out.push_str("        .any(|meta| meta.wire_name == method)\n");
    out.push_str("}\n");
    out
}

fn module_name(surface: Surface) -> &'static str {
    match surface {
        Surface::ClientRequest => "client_request",
        Surface::ServerRequest => "server_request",
        Surface::ServerNotification => "server_notification",
        Surface::ClientNotification => "client_notification",
    }
}

fn module_path(surface: Surface) -> &'static str {
    match surface {
        Surface::ClientRequest => "client_requests",
        Surface::ServerRequest => "server_requests",
        Surface::ServerNotification => "server_notifications",
        Surface::ClientNotification => "client_notifications",
    }
}

fn method_surface_name(surface: Surface) -> &'static str {
    match surface {
        Surface::ClientRequest => "ClientRequest",
        Surface::ServerRequest => "ServerRequest",
        Surface::ServerNotification => "ServerNotification",
        Surface::ClientNotification => "ClientNotification",
    }
}

fn stability_name(stability: Stability) -> &'static str {
    match stability {
        Stability::Stable => "Stable",
        Stability::Experimental => "Experimental",
    }
}

fn feature_name(feature: FeatureClass) -> &'static str {
    match feature {
        FeatureClass::Core => "Core",
        FeatureClass::Experimental => "Experimental",
    }
}

fn constant_name(wire_name: &str) -> String {
    let mut out = String::new();
    let mut prev_is_lower_or_digit = false;
    for ch in wire_name.chars() {
        match ch {
            'a'..='z' => {
                out.push(ch.to_ascii_uppercase());
                prev_is_lower_or_digit = true;
            }
            'A'..='Z' => {
                if prev_is_lower_or_digit && !out.ends_with('_') {
                    out.push('_');
                }
                out.push(ch);
                prev_is_lower_or_digit = false;
            }
            '0'..='9' => {
                out.push(ch);
                prev_is_lower_or_digit = true;
            }
            _ => {
                if !out.ends_with('_') {
                    out.push('_');
                }
                prev_is_lower_or_digit = false;
            }
        }
    }
    out.trim_matches('_').to_owned()
}
