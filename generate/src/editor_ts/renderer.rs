//! generates the renderer package

use crate::editor_ts::{
    first_letter_to_uppper_case, get_dependent_plugins, templates, TYPESCRIPT_IMPORTS,
    TYPESCRIPT_TYPES, package_json_patch
};
use crate::files::{GeneratedFile, GenerationError};
use handlebars::Handlebars;
use serde_json::json;
use serlo_he_spec::Plugins;
use serlo_he_spec_meta::{identifier_from_locator, Multiplicity, Plugin, Specification};
use std::error::Error;
use std::path::PathBuf;

pub fn generate_plugin_renderer(plugin: &Plugin) -> Result<Vec<GeneratedFile>, GenerationError> {
    let spec = Plugins::whole_specification();
    Ok(vec![
        index(plugin, &spec)?,
        package_json_patch(plugin, &spec, true)?,
    ])
}

fn state_attributes(plugin: &Plugin, spec: &Specification) -> Result<Vec<String>, GenerationError> {
    plugin.attributes.iter().try_fold(vec![], |mut res, a| {
        match TYPESCRIPT_TYPES.get(&a.content_type) {
            Some(t) => {
                let t = match a.multiplicity {
                    Multiplicity::Once => t.to_string(),
                    Multiplicity::Optional => format!("{} | null", &t),
                    Multiplicity::Arbitrary | Multiplicity::MinOnce => format!("Array<{}>", &t),
                };
                res.push(format!("{}: {}", a.identifier, t))
            }
            None => {
                return Err(GenerationError::new(format!(
                    "no typescript type defined for \"{}\"!",
                    &a.content_type
                )))
            }
        };
        Ok(res)
    })
}

fn index(plugin: &Plugin, spec: &Specification) -> Result<GeneratedFile, GenerationError> {
    let mut reg = Handlebars::new();
    reg.set_strict_mode(true);
    reg.register_escape_fn(|s| s.to_string());
    let component_ident = identifier_from_locator(&plugin.identifier.name);
    let content = reg
        .render_template(
            templates::RENDERER_INDEX,
            &json!({
                "imports": state_type_imports(plugin, spec),
                "component_ident": component_ident,
                "plugin_ident": plugin.identifier,
                "attributes": state_attributes(plugin, spec)?,
                "plugin_suffix": first_letter_to_uppper_case(&component_ident)
            }),
        )
        .map_err(|e| GenerationError::new(e.description().to_string()))?;
    Ok(GeneratedFile {
        path: PathBuf::from("src/index.ts"),
        content,
    })
}

/// A generates a list of imports for types used in the plugin state.
pub fn state_type_imports(plugin: &Plugin, spec: &Specification) -> Vec<String> {
    plugin
        .attributes
        .iter()
        .map(|a| {
            TYPESCRIPT_TYPES
                .get(&a.content_type)
                .unwrap_or(&a.content_type)
        })
        .filter_map(|t| {
            TYPESCRIPT_IMPORTS
                .get(t)
                .map(|p| format!("import {{ {} }} from '{}'", t, &p))
        })
        .collect()
}