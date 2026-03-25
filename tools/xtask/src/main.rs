use quote::ToTokens;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parenthesized, Attribute, Ident, Item, LitStr, Token};

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
        "protocol-codegen-check" => check_protocol_codegen(),
        "release" => release(),
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

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stability {
    Stable,
    Experimental,
    Deprecated,
    Internal,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeatureClass {
    Core,
    Experimental,
    Compatibility,
    Internal,
}

#[derive(Clone, Debug)]
struct Entry {
    rust_name: String,
    wire_name: String,
    surface: Surface,
    stability: Stability,
    feature: FeatureClass,
    params_type_name: Option<String>,
    result_type_name: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct PendingDocMeta {
    stability: Option<Stability>,
    feature: Option<FeatureClass>,
}

struct SnapshotInput {
    common_rs: String,
    v2_rs: String,
}

struct CommonProtocolAst {
    client_requests: syn::Macro,
    server_requests: syn::Macro,
    server_notifications: syn::Macro,
    client_notifications: syn::Macro,
}

struct ProtocolTables {
    source_hash: String,
    client_requests: Vec<Entry>,
    server_requests: Vec<Entry>,
    server_notifications: Vec<Entry>,
    client_notifications: Vec<Entry>,
}

struct NormalizedProtocolTables {
    source_hash: String,
    client_requests: Vec<Entry>,
    server_requests: Vec<Entry>,
    server_notifications: Vec<Entry>,
    client_notifications: Vec<Entry>,
}

struct OutputPlan {
    files: Vec<PlannedFile>,
}

struct PlannedFile {
    relative_path: &'static str,
    contents: String,
}

fn load_and_plan(repo_root: &Path) -> Result<(NormalizedProtocolTables, OutputPlan), String> {
    let snapshot = load_snapshot(repo_root)?;
    let parsed = parse_snapshot(&snapshot)?;
    let normalized = normalize_protocol(parsed);
    let plan = plan_generated_outputs(&normalized);
    Ok((normalized, plan))
}

fn release() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["publish", "-p", "codexus"])
        .status()
        .map_err(|err| format!("failed to run cargo publish: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo publish exited with {status}"))
    }
}

fn generate_protocol() -> Result<(), String> {
    let repo_root = find_repo_root()?;
    let (_normalized, plan) = load_and_plan(&repo_root)?;
    apply_generated_outputs(&repo_root, &plan.files)
}

fn check_protocol_codegen() -> Result<(), String> {
    let repo_root = find_repo_root()?;
    let (normalized, plan) = load_and_plan(&repo_root)?;
    let temp_root = std::env::temp_dir().join(format!(
        "codexus-protocol-codegen-check-{}",
        normalized.source_hash
    ));
    fs::create_dir_all(&temp_root)
        .map_err(|err| format!("create {}: {err}", temp_root.display()))?;
    apply_generated_outputs(&temp_root, &plan.files)?;
    let mut stale = Vec::new();
    for file in &plan.files {
        let path = repo_root.join(file.relative_path);
        let current =
            fs::read_to_string(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
        let planned_path = temp_root.join(file.relative_path);
        let actual = fs::read_to_string(&planned_path)
            .map_err(|err| format!("read {}: {err}", planned_path.display()))?;
        if actual != current {
            stale.push(file.relative_path);
        }
    }
    if stale.is_empty() {
        Ok(())
    } else {
        Err(format!("protocol codegen stale for: {}", stale.join(", ")))
    }
}

fn print_protocol_plan() -> Result<(), String> {
    let repo_root = find_repo_root()?;
    let (_normalized, plan) = load_and_plan(&repo_root)?;
    for file in plan.files {
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("create {}: {err}", parent.display()))?;
    }
    fs::write(path, contents).map_err(|err| format!("write {}: {err}", path.display()))
}

fn parse_snapshot(input: &SnapshotInput) -> Result<ProtocolTables, String> {
    let common_ast = parse_common_protocol_ast(&input.common_rs)?;
    Ok(ProtocolTables {
        source_hash: compute_source_hash(&input.common_rs, &input.v2_rs),
        client_requests: parse_request_entries(
            &common_ast.client_requests,
            Surface::ClientRequest,
        )?,
        server_requests: parse_request_entries(
            &common_ast.server_requests,
            Surface::ServerRequest,
        )?,
        server_notifications: parse_notification_entries(
            &common_ast.server_notifications,
            Surface::ServerNotification,
        )?,
        client_notifications: parse_client_notification_entries(
            &common_ast.client_notifications,
            Surface::ClientNotification,
        )?,
    })
}

fn normalize_protocol(tables: ProtocolTables) -> NormalizedProtocolTables {
    NormalizedProtocolTables {
        source_hash: tables.source_hash,
        client_requests: normalize_entries(tables.client_requests),
        server_requests: normalize_entries(tables.server_requests),
        server_notifications: normalize_entries(tables.server_notifications),
        client_notifications: normalize_entries(tables.client_notifications),
    }
}

fn normalize_entries(mut entries: Vec<Entry>) -> Vec<Entry> {
    let mut seen = std::collections::HashSet::new();
    entries.retain(|entry| seen.insert(entry.wire_name.clone()));
    entries
}

fn plan_generated_outputs(tables: &NormalizedProtocolTables) -> OutputPlan {
    OutputPlan {
        files: vec![
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
                relative_path: "crates/codexus-core/src/protocol/generated/types.rs",
                contents: render_types_module(
                    &tables.client_requests,
                    &tables.server_requests,
                    &tables.server_notifications,
                    &tables.client_notifications,
                ),
            },
            PlannedFile {
                relative_path: "crates/codexus-core/src/protocol/generated/validators.rs",
                contents: render_validators_module(
                    &tables.client_requests,
                    &tables.server_requests,
                ),
            },
            PlannedFile {
                relative_path: "crates/codexus-core/src/protocol/generated/codecs.rs",
                contents: render_codecs_module(
                    &tables.server_requests,
                    &tables.server_notifications,
                ),
            },
        ],
    }
}

fn apply_generated_outputs(repo_root: &Path, plan: &[PlannedFile]) -> Result<(), String> {
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

fn parse_common_protocol_ast(source: &str) -> Result<CommonProtocolAst, String> {
    let file = syn::parse_file(source).map_err(|err| format!("parse common.rs AST: {err}"))?;
    let mut client_requests = None;
    let mut server_requests = None;
    let mut server_notifications = None;
    let mut client_notifications = None;

    for item in file.items {
        let Item::Macro(item_macro) = item else {
            continue;
        };
        if item_macro.mac.path.is_ident("client_request_definitions") {
            client_requests = Some(item_macro.mac);
            continue;
        }
        if item_macro.mac.path.is_ident("server_request_definitions") {
            server_requests = Some(item_macro.mac);
            continue;
        }
        if item_macro
            .mac
            .path
            .is_ident("server_notification_definitions")
        {
            server_notifications = Some(item_macro.mac);
            continue;
        }
        if item_macro
            .mac
            .path
            .is_ident("client_notification_definitions")
        {
            client_notifications = Some(item_macro.mac);
        }
    }

    Ok(CommonProtocolAst {
        client_requests: client_requests.ok_or_else(|| {
            "missing client_request_definitions! invocation in common.rs".to_owned()
        })?,
        server_requests: server_requests.ok_or_else(|| {
            "missing server_request_definitions! invocation in common.rs".to_owned()
        })?,
        server_notifications: server_notifications.ok_or_else(|| {
            "missing server_notification_definitions! invocation in common.rs".to_owned()
        })?,
        client_notifications: client_notifications.ok_or_else(|| {
            "missing client_notification_definitions! invocation in common.rs".to_owned()
        })?,
    })
}

fn parse_request_entries(macro_def: &syn::Macro, surface: Surface) -> Result<Vec<Entry>, String> {
    let parsed = syn::parse2::<RequestEntryList>(macro_def.tokens.clone())
        .map_err(|err| format!("parse request entries for {:?}: {err}", surface))?;
    let mut entries = parsed
        .entries
        .into_iter()
        .filter_map(|entry| request_entry_to_entry(entry, surface))
        .collect::<Vec<_>>();

    if surface == Surface::ClientRequest {
        entries.insert(
            0,
            Entry {
                rust_name: "Initialize".to_owned(),
                wire_name: "initialize".to_owned(),
                surface,
                stability: Stability::Stable,
                feature: FeatureClass::Core,
                params_type_name: Some("v1::InitializeParams".to_owned()),
                result_type_name: Some("v1::InitializeResponse".to_owned()),
            },
        );
    }

    Ok(entries)
}

fn parse_notification_entries(
    macro_def: &syn::Macro,
    surface: Surface,
) -> Result<Vec<Entry>, String> {
    let parsed = syn::parse2::<NotificationEntryList>(macro_def.tokens.clone())
        .map_err(|err| format!("parse notification entries for {:?}: {err}", surface))?;
    Ok(parsed
        .entries
        .into_iter()
        .map(|entry| notification_entry_to_entry(entry, surface))
        .collect())
}

fn parse_client_notification_entries(
    macro_def: &syn::Macro,
    surface: Surface,
) -> Result<Vec<Entry>, String> {
    let parsed = syn::parse2::<ClientNotificationEntryList>(macro_def.tokens.clone())
        .map_err(|err| format!("parse client notification entries for {:?}: {err}", surface))?;
    let mut entries = parsed
        .entries
        .into_iter()
        .map(|entry| client_notification_entry_to_entry(entry, surface))
        .collect::<Vec<_>>();
    if !entries.iter().any(|entry| entry.rust_name == "Initialized") {
        entries.push(build_entry(
            "Initialized".to_owned(),
            "initialized".to_owned(),
            surface,
            false,
        ));
    }
    Ok(entries)
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
        params_type_name: None,
        result_type_name: None,
    }
}

fn request_entry_to_entry(parsed: RequestEntrySyntax, surface: Surface) -> Option<Entry> {
    let rename = extract_serde_rename(&parsed.attrs);
    let wire_name = parsed.wire_name.or(rename)?;
    let mut entry = build_entry(
        parsed.rust_name.to_string(),
        wire_name,
        surface,
        has_experimental_attr(&parsed.attrs),
    );
    apply_attribute_doc_meta(&parsed.attrs, &mut entry);
    entry.params_type_name = Some(parsed.params_type_name);
    entry.result_type_name = Some(parsed.result_type_name);
    Some(entry)
}

fn notification_entry_to_entry(parsed: NotificationEntrySyntax, surface: Surface) -> Entry {
    let wire_name = parsed
        .wire_name
        .or_else(|| extract_serde_rename(&parsed.attrs))
        .unwrap_or_else(|| parsed.rust_name.to_string().to_ascii_lowercase());
    let mut entry = build_entry(
        parsed.rust_name.to_string(),
        wire_name,
        surface,
        has_experimental_attr(&parsed.attrs),
    );
    apply_attribute_doc_meta(&parsed.attrs, &mut entry);
    entry.params_type_name = Some(parsed.payload_type_name);
    entry
}

fn client_notification_entry_to_entry(
    parsed: ClientNotificationEntrySyntax,
    surface: Surface,
) -> Entry {
    let wire_name = extract_serde_rename(&parsed.attrs)
        .unwrap_or_else(|| parsed.rust_name.to_string().to_ascii_lowercase());
    let mut entry = build_entry(
        parsed.rust_name.to_string(),
        wire_name,
        surface,
        has_experimental_attr(&parsed.attrs),
    );
    apply_attribute_doc_meta(&parsed.attrs, &mut entry);
    entry
}

fn apply_attribute_doc_meta(attrs: &[Attribute], entry: &mut Entry) {
    let mut pending = PendingDocMeta::default();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            let syn::Meta::NameValue(meta) = &attr.meta else {
                continue;
            };
            let syn::Expr::Lit(expr_lit) = &meta.value else {
                continue;
            };
            let syn::Lit::Str(doc) = &expr_lit.lit else {
                continue;
            };
            update_pending_doc_meta(&doc.value(), &mut pending);
        }
    }
    if let Some(stability) = pending.stability {
        entry.stability = stability;
    }
    if let Some(feature) = pending.feature {
        entry.feature = feature;
    }
}

fn has_experimental_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("experimental"))
}

fn extract_serde_rename(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut rename = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                rename = Some(lit.value());
            }
            Ok(())
        });
        if rename.is_some() {
            return rename;
        }
    }
    None
}

fn token_stream_to_string(tokens: &proc_macro2::TokenStream) -> String {
    tokens.to_token_stream().to_string().replace(" :: ", "::")
}

fn parse_field_value_tokens(input: ParseStream<'_>) -> syn::Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();
    while !input.is_empty() && !input.peek(Token![,]) {
        let token: proc_macro2::TokenTree = input.parse()?;
        tokens.extend(std::iter::once(token));
    }
    Ok(tokens)
}

struct RequestEntryList {
    entries: Vec<RequestEntrySyntax>,
}

fn parse_comma_separated<T: Parse>(input: ParseStream<'_>) -> syn::Result<Vec<T>> {
    let mut entries = Vec::new();
    while !input.is_empty() {
        entries.push(input.parse()?);
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(entries)
}

impl Parse for RequestEntryList {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            entries: parse_comma_separated(input)?,
        })
    }
}

struct RequestEntrySyntax {
    attrs: Vec<Attribute>,
    rust_name: Ident,
    wire_name: Option<String>,
    params_type_name: String,
    result_type_name: String,
}

impl Parse for RequestEntrySyntax {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let rust_name: Ident = input.parse()?;
        let wire_name = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            Some(input.parse::<LitStr>()?.value())
        } else {
            None
        };
        let content;
        braced!(content in input);
        let mut params_type_name = None;
        let mut result_type_name = None;
        while !content.is_empty() {
            let field_name: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            let field_value = parse_field_value_tokens(&content)?;
            let field_value = token_stream_to_string(&field_value);
            match field_name.to_string().as_str() {
                "params" => params_type_name = Some(field_value),
                "response" => result_type_name = Some(field_value),
                _ => {}
            }
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            attrs,
            rust_name: rust_name.clone(),
            wire_name,
            params_type_name: params_type_name
                .ok_or_else(|| syn::Error::new(rust_name.span(), "missing params field"))?,
            result_type_name: result_type_name
                .ok_or_else(|| syn::Error::new(rust_name.span(), "missing response field"))?,
        })
    }
}

struct NotificationEntryList {
    entries: Vec<NotificationEntrySyntax>,
}

impl Parse for NotificationEntryList {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            entries: parse_comma_separated(input)?,
        })
    }
}

struct NotificationEntrySyntax {
    attrs: Vec<Attribute>,
    rust_name: Ident,
    wire_name: Option<String>,
    payload_type_name: String,
}

impl Parse for NotificationEntrySyntax {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let rust_name: Ident = input.parse()?;
        let wire_name = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            Some(input.parse::<LitStr>()?.value())
        } else {
            None
        };
        let payload;
        parenthesized!(payload in input);
        let payload_type_name =
            token_stream_to_string(&payload.parse::<proc_macro2::TokenStream>()?);
        Ok(Self {
            attrs,
            rust_name,
            wire_name,
            payload_type_name,
        })
    }
}

struct ClientNotificationEntryList {
    entries: Vec<ClientNotificationEntrySyntax>,
}

impl Parse for ClientNotificationEntryList {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            entries: parse_comma_separated(input)?,
        })
    }
}

struct ClientNotificationEntrySyntax {
    attrs: Vec<Attribute>,
    rust_name: Ident,
}

impl Parse for ClientNotificationEntrySyntax {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let rust_name: Ident = input.parse()?;
        Ok(Self { attrs, rust_name })
    }
}

fn update_pending_doc_meta(doc: &str, pending: &mut PendingDocMeta) {
    if doc.contains("internal-only") {
        pending.stability = Some(Stability::Internal);
        pending.feature = Some(FeatureClass::Internal);
    }
    if doc.contains("Deprecated:") {
        pending.stability = Some(Stability::Deprecated);
        pending.feature = Some(FeatureClass::Compatibility);
    }
}

fn render_surface_module(entries: &[Entry]) -> String {
    let mut out = String::new();
    out.push_str("use super::types::*;\n\n");
    writeln!(
        &mut out,
        "macro_rules! define_{}_specs {{",
        module_name(entries.first().expect("entries present").surface)
    )
    .unwrap();
    match entries[0].surface {
        Surface::ClientRequest | Surface::ServerRequest => {
            out.push_str(
                "    ($($name:ident => $wire:literal, $stability:ident, $feature:ident, $params_ty:expr, $result_ty:expr, $spec_params_ty:ty, $spec_result_ty:ty),* $(,)?) => {\n",
            );
        }
        Surface::ServerNotification | Surface::ClientNotification => {
            out.push_str(
                "    ($($name:ident => $wire:literal, $stability:ident, $feature:ident, $params_ty:expr, $result_ty:expr, $spec_params_ty:ty),* $(,)?) => {\n",
            );
        }
    }
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
    out.push_str("                    $params_ty,\n");
    out.push_str("                    $result_ty,\n");
    out.push_str("                );\n");
    out.push_str("            }\n\n");
    out.push_str("            impl MethodSpec for $name {\n");
    out.push_str("                const META: MethodMeta = $name::META;\n");
    out.push_str("            }\n\n");
    match entries[0].surface {
        Surface::ClientRequest => {
            out.push_str("            impl ClientRequestSpec for $name {\n");
            out.push_str("                type Params = $spec_params_ty;\n");
            out.push_str("                type Response = $spec_result_ty;\n");
            out.push_str("            }\n");
        }
        Surface::ServerRequest => {
            out.push_str("            impl ServerRequestSpec for $name {\n");
            out.push_str("                type Params = $spec_params_ty;\n");
            out.push_str("                type Response = $spec_result_ty;\n");
            out.push_str("            }\n");
        }
        Surface::ServerNotification => {
            out.push_str("            impl ServerNotificationSpec for $name {\n");
            out.push_str("                type Params = $spec_params_ty;\n");
            out.push_str("            }\n");
        }
        Surface::ClientNotification => {
            out.push_str("            impl ClientNotificationSpec for $name {\n");
            out.push_str("                type Params = $spec_params_ty;\n");
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
        let params_ty = entry
            .params_type_name
            .as_deref()
            .unwrap_or("serde_json::Value");
        let result_ty = match entry.result_type_name.as_deref() {
            Some(name) => format!("Some({name:?})"),
            None => "None".to_owned(),
        };
        let params_ty = format!("{params_ty:?}");
        match entry.surface {
            Surface::ClientRequest | Surface::ServerRequest => {
                writeln!(
                    &mut out,
                    "    {} => \"{}\", {}, {}, {}, {}, {}, {},",
                    entry.rust_name,
                    entry.wire_name,
                    stability_name(entry.stability),
                    feature_name(entry.feature),
                    params_ty,
                    result_ty,
                    generated_params_type_name(entry),
                    generated_result_type_name(entry),
                )
                .unwrap();
            }
            Surface::ServerNotification | Surface::ClientNotification => {
                writeln!(
                    &mut out,
                    "    {} => \"{}\", {}, {}, {}, {}, {},",
                    entry.rust_name,
                    entry.wire_name,
                    stability_name(entry.stability),
                    feature_name(entry.feature),
                    params_ty,
                    result_ty,
                    generated_params_type_name(entry),
                )
                .unwrap();
            }
        }
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
    out.push('\n');
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

fn render_validators_module(client_requests: &[Entry], server_requests: &[Entry]) -> String {
    let mut out = String::new();
    out.push_str("use serde_json::Value;\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub enum ClientRequestParamsContract {\n");
    out.push_str("    Object,\n");
    out.push_str("    ProcessId,\n");
    out.push_str("    ThreadId,\n");
    out.push_str("    ThreadIdAndTurnId,\n");
    out.push_str("    CommandExec,\n");
    out.push_str("    CommandExecWrite,\n");
    out.push_str("    CommandExecResize,\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub enum ClientRequestResultContract {\n");
    out.push_str("    Object,\n");
    out.push_str("    ThreadObject,\n");
    out.push_str("    TurnObject,\n");
    out.push_str("    CommandExec,\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub struct ClientRequestValidator {\n");
    out.push_str("    pub wire_name: &'static str,\n");
    out.push_str("    pub params: ClientRequestParamsContract,\n");
    out.push_str("    pub result: ClientRequestResultContract,\n");
    out.push_str("}\n\n");
    out.push_str("pub const CLIENT_REQUEST_VALIDATORS: &[ClientRequestValidator] = &[\n");
    for entry in client_requests {
        writeln!(
            &mut out,
            "    ClientRequestValidator {{ wire_name: \"{}\", params: ClientRequestParamsContract::{}, result: ClientRequestResultContract::{} }},",
            entry.wire_name,
            client_request_params_contract_name(&entry.wire_name),
            client_request_result_contract_name(&entry.wire_name)
        )
        .unwrap();
    }
    out.push_str("];\n\n");
    out.push_str("pub fn is_known_server_request(method: &str) -> bool {\n");
    out.push_str("    matches!(\n");
    out.push_str("        method,\n");
    let arms: Vec<String> = server_requests
        .iter()
        .map(|entry| format!("        {:?}", entry.wire_name))
        .collect();
    out.push_str(&arms.join("\n        | "));
    out.push('\n');
    out.push_str("    )\n");
    out.push_str("}\n\n");
    out.push_str(
        "pub fn client_request_validator(method: &str) -> Option<&'static ClientRequestValidator> {\n",
    );
    out.push_str("    CLIENT_REQUEST_VALIDATORS\n");
    out.push_str("        .iter()\n");
    out.push_str("        .find(|validator| validator.wire_name == method)\n");
    out.push_str("}\n\n");
    out.push_str("pub fn classify_client_request_params_contract(method: &str) -> Option<ClientRequestParamsContract> {\n");
    out.push_str("    client_request_validator(method).map(|validator| validator.params)\n");
    out.push_str("}\n\n");
    out.push_str("pub fn classify_client_request_result_contract(method: &str) -> Option<ClientRequestResultContract> {\n");
    out.push_str("    client_request_validator(method).map(|validator| validator.result)\n");
    out.push_str("}\n\n");
    out.push_str("pub fn validate_client_request_params(method: &str, params: &Value) -> Result<(), String> {\n");
    out.push_str(
        "    let Some(contract) = classify_client_request_params_contract(method) else {\n",
    );
    out.push_str("        return Ok(());\n");
    out.push_str("    };\n");
    out.push_str("    match contract {\n");
    out.push_str("        ClientRequestParamsContract::Object => {\n");
    out.push_str("            require_object(params, \"params\")?;\n");
    out.push_str("            Ok(())\n");
    out.push_str("        }\n");
    out.push_str("        ClientRequestParamsContract::ProcessId => {\n");
    out.push_str("            require_non_empty_string(params, \"params\", \"processId\")\n");
    out.push_str("        }\n");
    out.push_str("        ClientRequestParamsContract::ThreadId => {\n");
    out.push_str("            require_non_empty_string(params, \"params\", \"threadId\")\n");
    out.push_str("        }\n");
    out.push_str("        ClientRequestParamsContract::ThreadIdAndTurnId => {\n");
    out.push_str("            require_non_empty_string(params, \"params\", \"threadId\")?;\n");
    out.push_str("            require_non_empty_string(params, \"params\", \"turnId\")\n");
    out.push_str("        }\n");
    out.push_str("        ClientRequestParamsContract::CommandExec => validate_command_exec_request(params),\n");
    out.push_str("        ClientRequestParamsContract::CommandExecWrite => validate_command_exec_write_request(params),\n");
    out.push_str("        ClientRequestParamsContract::CommandExecResize => validate_command_exec_resize_request(params),\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out.push_str("pub fn validate_client_request_result(method: &str, result: &Value) -> Result<(), String> {\n");
    out.push_str(
        "    let Some(contract) = classify_client_request_result_contract(method) else {\n",
    );
    out.push_str("        return Ok(());\n");
    out.push_str("    };\n");
    out.push_str("    match contract {\n");
    out.push_str("        ClientRequestResultContract::Object => {\n");
    out.push_str("            require_object(result, \"result\")?;\n");
    out.push_str("            Ok(())\n");
    out.push_str("        }\n");
    out.push_str("        ClientRequestResultContract::ThreadObject => require_nested_object(result, \"thread\", \"result.thread\"),\n");
    out.push_str("        ClientRequestResultContract::TurnObject => require_nested_object(result, \"turn\", \"result.turn\"),\n");
    out.push_str("        ClientRequestResultContract::CommandExec => validate_command_exec_response(result),\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out.push_str("fn require_object<'a>(value: &'a Value, field_name: &str) -> Result<&'a serde_json::Map<String, Value>, String> {\n");
    out.push_str(
        "    value.as_object().ok_or_else(|| format!(\"{field_name} must be an object\"))\n",
    );
    out.push_str("}\n\n");
    out.push_str("fn require_non_empty_string(value: &Value, field_name: &str, key: &str) -> Result<(), String> {\n");
    out.push_str("    let obj = require_object(value, field_name)?;\n");
    out.push_str("    match obj.get(key).and_then(Value::as_str) {\n");
    out.push_str("        Some(v) if !v.trim().is_empty() => Ok(()),\n");
    out.push_str("        _ => Err(format!(\"{field_name}.{key} must be a non-empty string\")),\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out.push_str("fn require_nested_object(value: &Value, key: &str, field_name: &str) -> Result<(), String> {\n");
    out.push_str("    let obj = require_object(value, \"result\")?;\n");
    out.push_str("    obj.get(key)\n");
    out.push_str("        .and_then(Value::as_object)\n");
    out.push_str("        .map(|_| ())\n");
    out.push_str("        .ok_or_else(|| format!(\"{field_name} must be an object\"))\n");
    out.push_str("}\n\n");
    out.push_str("fn require_string_field(obj: &serde_json::Map<String, Value>, key: &str, field_name: &str) -> Result<(), String> {\n");
    out.push_str("    if obj.get(key).and_then(Value::as_str).is_some() {\n");
    out.push_str("        Ok(())\n");
    out.push_str("    } else {\n");
    out.push_str("        Err(format!(\"{field_name}.{key} must be a string\"))\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out.push_str("fn validate_command_exec_request(params: &Value) -> Result<(), String> {\n");
    out.push_str("    let obj = require_object(params, \"params\")?;\n");
    out.push_str("    let command = obj.get(\"command\").and_then(Value::as_array).ok_or_else(|| \"params.command must be an array\".to_owned())?;\n");
    out.push_str("    if command.is_empty() {\n");
    out.push_str("        return Err(\"params.command must not be empty\".to_owned());\n");
    out.push_str("    }\n");
    out.push_str("    if command.iter().any(|value| value.as_str().is_none()) {\n");
    out.push_str("        return Err(\"params.command items must be strings\".to_owned());\n");
    out.push_str("    }\n");
    out.push_str("    let process_id = optional_non_empty_string(obj, \"processId\")?;\n");
    out.push_str("    let tty = get_bool(obj, \"tty\");\n");
    out.push_str("    let stream_stdin = get_bool(obj, \"streamStdin\");\n");
    out.push_str("    let stream_stdout_stderr = get_bool(obj, \"streamStdoutStderr\");\n");
    out.push_str("    let effective_stream_stdin = tty || stream_stdin;\n");
    out.push_str("    let effective_stream_stdout_stderr = tty || stream_stdout_stderr;\n");
    out.push_str("    if (tty || effective_stream_stdin || effective_stream_stdout_stderr) && process_id.is_none() {\n");
    out.push_str("        return Err(\"params.processId is required when tty or streaming is enabled\".to_owned());\n");
    out.push_str("    }\n");
    out.push_str(
        "    if get_bool(obj, \"disableOutputCap\") && obj.get(\"outputBytesCap\").is_some() {\n",
    );
    out.push_str("        return Err(\"params.disableOutputCap cannot be combined with params.outputBytesCap\".to_owned());\n");
    out.push_str("    }\n");
    out.push_str(
        "    if get_bool(obj, \"disableTimeout\") && obj.get(\"timeoutMs\").is_some() {\n",
    );
    out.push_str("        return Err(\"params.disableTimeout cannot be combined with params.timeoutMs\".to_owned());\n");
    out.push_str("    }\n");
    out.push_str(
        "    if let Some(timeout_ms) = obj.get(\"timeoutMs\").and_then(Value::as_i64) {\n",
    );
    out.push_str("        if timeout_ms < 0 {\n");
    out.push_str("            return Err(\"params.timeoutMs must be >= 0\".to_owned());\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("    if let Some(output_bytes_cap) = obj.get(\"outputBytesCap\").and_then(Value::as_u64) {\n");
    out.push_str("        if output_bytes_cap == 0 {\n");
    out.push_str("            return Err(\"params.outputBytesCap must be > 0\".to_owned());\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("    if let Some(size) = obj.get(\"size\") {\n");
    out.push_str("        if !tty {\n");
    out.push_str("            return Err(\"params.size is only valid when params.tty is true\".to_owned());\n");
    out.push_str("        }\n");
    out.push_str("        validate_command_exec_size(size)?;\n");
    out.push_str("    }\n");
    out.push_str("    Ok(())\n");
    out.push_str("}\n\n");
    out.push_str(
        "fn validate_command_exec_write_request(params: &Value) -> Result<(), String> {\n",
    );
    out.push_str("    require_non_empty_string(params, \"params\", \"processId\")?;\n");
    out.push_str("    let obj = require_object(params, \"params\")?;\n");
    out.push_str(
        "    let has_delta = obj.get(\"deltaBase64\").and_then(Value::as_str).is_some();\n",
    );
    out.push_str("    let close_stdin = get_bool(obj, \"closeStdin\");\n");
    out.push_str("    if !has_delta && !close_stdin {\n");
    out.push_str(
        "        Err(\"params must include deltaBase64, closeStdin, or both\".to_owned())\n",
    );
    out.push_str("    } else {\n");
    out.push_str("        Ok(())\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out.push_str(
        "fn validate_command_exec_resize_request(params: &Value) -> Result<(), String> {\n",
    );
    out.push_str("    require_non_empty_string(params, \"params\", \"processId\")?;\n");
    out.push_str("    let obj = require_object(params, \"params\")?;\n");
    out.push_str("    let size = obj.get(\"size\").ok_or_else(|| \"params.size must be an object\".to_owned())?;\n");
    out.push_str("    validate_command_exec_size(size)\n");
    out.push_str("}\n\n");
    out.push_str("fn validate_command_exec_response(result: &Value) -> Result<(), String> {\n");
    out.push_str("    let obj = require_object(result, \"result\")?;\n");
    out.push_str("    match obj.get(\"exitCode\").and_then(Value::as_i64) {\n");
    out.push_str("        Some(code) if i32::try_from(code).is_ok() => {}\n");
    out.push_str("        _ => return Err(\"result.exitCode must be an i32-compatible integer\".to_owned()),\n");
    out.push_str("    }\n");
    out.push_str("    require_string_field(obj, \"stdout\", \"result\")?;\n");
    out.push_str("    require_string_field(obj, \"stderr\", \"result\")\n");
    out.push_str("}\n\n");
    out.push_str("fn validate_command_exec_size(size: &Value) -> Result<(), String> {\n");
    out.push_str("    let obj = size.as_object().ok_or_else(|| \"params.size must be an object\".to_owned())?;\n");
    out.push_str("    let rows = obj.get(\"rows\").and_then(Value::as_u64).unwrap_or(0);\n");
    out.push_str("    let cols = obj.get(\"cols\").and_then(Value::as_u64).unwrap_or(0);\n");
    out.push_str("    if rows == 0 {\n");
    out.push_str("        return Err(\"params.size.rows must be > 0\".to_owned());\n");
    out.push_str("    }\n");
    out.push_str("    if cols == 0 {\n");
    out.push_str("        return Err(\"params.size.cols must be > 0\".to_owned());\n");
    out.push_str("    }\n");
    out.push_str("    Ok(())\n");
    out.push_str("}\n\n");
    out.push_str("fn optional_non_empty_string<'a>(obj: &'a serde_json::Map<String, Value>, key: &str) -> Result<Option<&'a str>, String> {\n");
    out.push_str("    match obj.get(key) {\n");
    out.push_str(
        "        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text)),\n",
    );
    out.push_str("        Some(Value::String(_)) => Err(format!(\"params.{key} must be a non-empty string\")),\n");
    out.push_str("        Some(_) => Err(format!(\"params.{key} must be a string\")),\n");
    out.push_str("        None => Ok(None),\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out.push_str("fn get_bool(obj: &serde_json::Map<String, Value>, key: &str) -> bool {\n");
    out.push_str("    obj.get(key).and_then(Value::as_bool).unwrap_or(false)\n");
    out.push_str("}\n\n");
    out
}

fn client_request_params_contract_name(wire_name: &str) -> &'static str {
    match wire_name {
        "command/exec" => "CommandExec",
        "command/exec/write" => "CommandExecWrite",
        "command/exec/resize" => "CommandExecResize",
        "command/exec/terminate" => "ProcessId",
        "thread/resume" => "ThreadId",
        "thread/fork" => "ThreadId",
        "thread/archive" => "ThreadId",
        "thread/read" => "ThreadId",
        "thread/rollback" => "ThreadId",
        "turn/interrupt" => "ThreadIdAndTurnId",
        _ => "Object",
    }
}

fn client_request_result_contract_name(wire_name: &str) -> &'static str {
    match wire_name {
        "command/exec" => "CommandExec",
        "thread/start" => "ThreadObject",
        "turn/start" => "TurnObject",
        _ => "Object",
    }
}

fn render_types_module(
    client_requests: &[Entry],
    server_requests: &[Entry],
    server_notifications: &[Entry],
    client_notifications: &[Entry],
) -> String {
    let mut out = String::new();
    out.push_str("#![allow(dead_code)]\n");
    out.push_str("use std::str::FromStr;\n");
    out.push_str("use serde::de::DeserializeOwned;\n");
    out.push_str("use serde::{Deserialize, Deserializer, Serialize, Serializer};\n");
    out.push_str("use serde_json::{Map, Value};\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub enum Stability {\n");
    out.push_str("    Stable,\n");
    out.push_str("    Experimental,\n");
    out.push_str("    Deprecated,\n");
    out.push_str("    Internal,\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub enum FeatureClass {\n");
    out.push_str("    Core,\n");
    out.push_str("    Experimental,\n");
    out.push_str("    Compatibility,\n");
    out.push_str("    Internal,\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub enum MethodSurface {\n");
    out.push_str("    ClientRequest,\n");
    out.push_str("    ServerRequest,\n");
    out.push_str("    ServerNotification,\n");
    out.push_str("    ClientNotification,\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub struct MethodMeta {\n");
    out.push_str("    pub rust_name: &'static str,\n");
    out.push_str("    pub wire_name: &'static str,\n");
    out.push_str("    pub surface: MethodSurface,\n");
    out.push_str("    pub stability: Stability,\n");
    out.push_str("    pub feature: FeatureClass,\n");
    out.push_str("    pub params_type: &'static str,\n");
    out.push_str("    pub result_type: Option<&'static str>,\n");
    out.push_str("}\n\n");
    out.push_str("impl MethodMeta {\n");
    out.push_str("    pub const fn new(\n");
    out.push_str("        rust_name: &'static str,\n");
    out.push_str("        wire_name: &'static str,\n");
    out.push_str("        surface: MethodSurface,\n");
    out.push_str("        stability: Stability,\n");
    out.push_str("        feature: FeatureClass,\n");
    out.push_str("        params_type: &'static str,\n");
    out.push_str("        result_type: Option<&'static str>,\n");
    out.push_str("    ) -> Self {\n");
    out.push_str("        Self {\n");
    out.push_str("            rust_name,\n");
    out.push_str("            wire_name,\n");
    out.push_str("            surface,\n");
    out.push_str("            stability,\n");
    out.push_str("            feature,\n");
    out.push_str("            params_type,\n");
    out.push_str("            result_type,\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n");
    out.push_str("pub struct ProtocolInventory {\n");
    out.push_str("    pub source_revision: &'static str,\n");
    out.push_str("    pub source_hash: &'static str,\n");
    out.push_str("    pub all_methods: &'static [MethodMeta],\n");
    out.push_str("    pub client_requests: &'static [MethodMeta],\n");
    out.push_str("    pub server_requests: &'static [MethodMeta],\n");
    out.push_str("    pub server_notifications: &'static [MethodMeta],\n");
    out.push_str("    pub client_notifications: &'static [MethodMeta],\n");
    out.push_str("}\n\n");
    out.push_str("pub type WireValue = Value;\n\n");
    out.push_str("pub type WireObject = Map<String, Value>;\n\n");
    out.push_str("pub trait MethodSpec {\n");
    out.push_str("    const META: MethodMeta;\n");
    out.push_str("}\n\n");
    out.push_str("pub trait ClientRequestSpec: MethodSpec {\n");
    out.push_str("    type Params: Serialize;\n");
    out.push_str("    type Response: DeserializeOwned;\n");
    out.push_str("}\n\n");
    out.push_str("pub trait ServerRequestSpec: MethodSpec {\n");
    out.push_str("    type Params: Serialize;\n");
    out.push_str("    type Response: DeserializeOwned;\n");
    out.push_str("}\n\n");
    out.push_str("pub trait ServerNotificationSpec: MethodSpec {\n");
    out.push_str("    type Params: Serialize + DeserializeOwned;\n");
    out.push_str("}\n\n");
    out.push_str("pub trait ClientNotificationSpec: MethodSpec {\n");
    out.push_str("    type Params: Serialize + DeserializeOwned;\n");
    out.push_str("}\n\n");
    out.push_str("pub fn decode_notification<N>(params: Value) -> serde_json::Result<N::Params>\n");
    out.push_str("where\n");
    out.push_str("    N: ServerNotificationSpec,\n");
    out.push_str("{\n");
    out.push_str("    serde_json::from_value(params)\n");
    out.push_str("}\n");
    out.push_str("\nmacro_rules! define_protocol_object_type {\n");
    out.push_str("    ($name:ident) => {\n");
    out.push_str("        #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]\n");
    out.push_str("        #[serde(rename_all = \"camelCase\")]\n");
    out.push_str("        pub struct $name {\n");
    out.push_str("            #[serde(flatten)]\n");
    out.push_str("            pub extra: WireObject,\n");
    out.push_str("        }\n");
    out.push_str("        impl From<WireObject> for $name {\n");
    out.push_str("            fn from(extra: WireObject) -> Self {\n");
    out.push_str("                Self { extra }\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        impl From<Value> for $name {\n");
    out.push_str("            fn from(value: Value) -> Self {\n");
    out.push_str("                match value {\n");
    out.push_str("                    Value::Object(extra) => Self { extra },\n");
    out.push_str("                    other => Self {\n");
    out.push_str("                        extra: WireObject::from_iter([(String::from(\"value\"), other)]),\n");
    out.push_str("                    },\n");
    out.push_str("                }\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        impl std::ops::Deref for $name {\n");
    out.push_str("            type Target = WireObject;\n");
    out.push_str("            fn deref(&self) -> &Self::Target {\n");
    out.push_str("                &self.extra\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        impl PartialEq<Value> for $name {\n");
    out.push_str("            fn eq(&self, other: &Value) -> bool {\n");
    out.push_str("                &Value::Object(self.extra.clone()) == other\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        impl PartialEq<$name> for Value {\n");
    out.push_str("            fn eq(&self, other: &$name) -> bool {\n");
    out.push_str("                self == &Value::Object(other.extra.clone())\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("    };\n");
    out.push_str("}\n\n");
    out.push_str("macro_rules! define_protocol_null_type {\n");
    out.push_str("    ($name:ident) => {\n");
    out.push_str(
        "        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]\n",
    );
    out.push_str("        pub struct $name;\n");
    out.push_str("        impl From<()> for $name {\n");
    out.push_str("            fn from(_: ()) -> Self {\n");
    out.push_str("                Self\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("    };\n");
    out.push_str("}\n\n");

    render_custom_protocol_types(&mut out);

    for entry in client_requests
        .iter()
        .chain(server_requests)
        .chain(server_notifications)
        .chain(client_notifications)
    {
        render_protocol_type_decl(
            &mut out,
            generated_params_type_name(entry),
            entry.params_type_name.as_deref(),
        );
        if matches!(
            entry.surface,
            Surface::ClientRequest | Surface::ServerRequest
        ) {
            render_protocol_type_decl(
                &mut out,
                generated_result_type_name(entry),
                entry.result_type_name.as_deref(),
            );
        }
    }
    out
}

fn render_codecs_module(server_requests: &[Entry], server_notifications: &[Entry]) -> String {
    let mut out = String::new();
    out.push_str("use serde_json::Value;\n\n");
    out.push_str("use super::types::*;\n\n");
    out.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    out.push_str("pub struct UnknownServerRequest {\n");
    out.push_str("    pub method: String,\n");
    out.push_str("    pub params: Value,\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    out.push_str("pub struct UnknownNotification {\n");
    out.push_str("    pub method: String,\n");
    out.push_str("    pub params: Value,\n");
    out.push_str("}\n\n");
    out.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    out.push_str("pub enum ServerRequestEnvelope {\n");
    for entry in server_requests {
        writeln!(
            &mut out,
            "    {}({}),",
            entry.rust_name,
            generated_params_type_name(entry)
        )
        .unwrap();
    }
    out.push_str("    Unknown(UnknownServerRequest),\n");
    out.push_str("}\n\n");

    out.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    out.push_str("pub enum ServerRequestResponse {\n");
    for entry in server_requests {
        writeln!(
            &mut out,
            "    {}({}),",
            entry.rust_name,
            generated_result_type_name(entry)
        )
        .unwrap();
    }
    out.push_str("    Unknown(Value),\n");
    out.push_str("}\n\n");

    out.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    out.push_str("pub enum ServerNotificationEnvelope {\n");
    for entry in server_notifications {
        writeln!(
            &mut out,
            "    {}({}),",
            entry.rust_name,
            generated_params_type_name(entry)
        )
        .unwrap();
    }
    out.push_str("    Unknown(UnknownNotification),\n");
    out.push_str("}\n\n");

    out.push_str("pub fn decode_server_request(method: &str, params: Value) -> Option<ServerRequestEnvelope> {\n");
    out.push_str("    match method {\n");
    for entry in server_requests {
        writeln!(
            &mut out,
            "        \"{}\" => serde_json::from_value::<{}>(params).ok().map(ServerRequestEnvelope::{}),",
            entry.wire_name,
            generated_params_type_name(entry),
            entry.rust_name
        )
        .unwrap();
    }
    out.push_str("        _ => Some(ServerRequestEnvelope::Unknown(UnknownServerRequest {\n");
    out.push_str("            method: method.to_owned(),\n");
    out.push_str("            params,\n");
    out.push_str("        })),\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str("pub fn encode_server_request_response(request: &ServerRequestEnvelope, response: ServerRequestResponse) -> Result<Value, String> {\n");
    out.push_str("    match (request, response) {\n");
    for entry in server_requests {
        writeln!(
            &mut out,
            "        (ServerRequestEnvelope::{}(_), ServerRequestResponse::{}(value)) => serde_json::to_value(value).map_err(|err| err.to_string()),",
            entry.rust_name,
            entry.rust_name
        )
        .unwrap();
    }
    out.push_str("        (ServerRequestEnvelope::Unknown(_), ServerRequestResponse::Unknown(value)) => Ok(value),\n");
    out.push_str("        (request, response) => Err(format!(\"server request/response mismatch: request={request:?} response={response:?}\")),\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str("pub fn decode_server_notification(method: &str, params: Value) -> Option<ServerNotificationEnvelope> {\n");
    out.push_str("    match method {\n");
    for entry in server_notifications {
        writeln!(
            &mut out,
            "        \"{}\" => serde_json::from_value::<{}>(params).ok().map(ServerNotificationEnvelope::{}),",
            entry.wire_name,
            generated_params_type_name(entry),
            entry.rust_name
        )
        .unwrap();
    }
    out.push_str("        _ => Some(ServerNotificationEnvelope::Unknown(UnknownNotification {\n");
    out.push_str("            method: method.to_owned(),\n");
    out.push_str("            params,\n");
    out.push_str("        })),\n");
    out.push_str("    }\n");
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

fn generated_params_type_name(entry: &Entry) -> String {
    match entry.surface {
        Surface::ClientRequest | Surface::ServerRequest => format!("{}Params", entry.rust_name),
        Surface::ServerNotification | Surface::ClientNotification => {
            format!("{}Notification", entry.rust_name)
        }
    }
}

fn generated_result_type_name(entry: &Entry) -> String {
    format!("{}Response", entry.rust_name)
}

fn is_null_like_type(type_name: Option<&str>) -> bool {
    let Some(type_name) = type_name else {
        return false;
    };
    let normalized: String = type_name.chars().filter(|ch| !ch.is_whitespace()).collect();
    normalized == "()" || normalized == "Option<()>" || normalized.contains("type=\"undefined\"")
}

fn render_protocol_type_decl(
    out: &mut String,
    generated_name: String,
    upstream_name: Option<&str>,
) {
    if has_custom_protocol_type(&generated_name) {
        return;
    }
    if is_null_like_type(upstream_name) {
        writeln!(out, "define_protocol_null_type!({generated_name});").unwrap();
    } else {
        writeln!(out, "define_protocol_object_type!({generated_name});").unwrap();
    }
}

fn has_custom_protocol_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "ThreadReadParams"
            | "ThreadListSortKey"
            | "ThreadListParams"
            | "ThreadListResponse"
            | "ThreadLoadedListParams"
            | "ThreadLoadedListResponse"
            | "ThreadRollbackParams"
            | "ThreadRollbackResponse"
            | "SkillsListParams"
            | "SkillsListExtraRootsForCwd"
            | "SkillsListResponse"
            | "SkillsListEntry"
            | "SkillScope"
            | "SkillMetadata"
            | "SkillInterface"
            | "SkillDependencies"
            | "SkillToolDependency"
            | "SkillErrorInfo"
            | "ThreadTurnStatus"
            | "ThreadItemType"
            | "ThreadAgentMessageItemView"
            | "ThreadCommandExecutionItemView"
            | "ThreadItemPayloadView"
            | "ThreadItemView"
            | "ThreadTurnErrorView"
            | "ThreadTurnView"
            | "ThreadView"
            | "ThreadReadResponse"
    )
}

fn render_custom_protocol_types(out: &mut String) {
    out.push_str(CUSTOM_PROTOCOL_TYPES);
}

const CUSTOM_PROTOCOL_TYPES: &str = include_str!("protocol_custom_types.rs.inc");

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
        Stability::Deprecated => "Deprecated",
        Stability::Internal => "Internal",
    }
}

fn feature_name(feature: FeatureClass) -> &'static str {
    match feature {
        FeatureClass::Core => "Core",
        FeatureClass::Experimental => "Experimental",
        FeatureClass::Compatibility => "Compatibility",
        FeatureClass::Internal => "Internal",
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
