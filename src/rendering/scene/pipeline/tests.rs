#![cfg(test)]

use super::geometry::{
    sandy_is_scaled, sandy_target_dimensions, sandy_vertices, sandy_video_in, sandy_video_out,
};
use super::model::{Globals, SandyUniformRaw, TransUniformRaw};
use super::primitive::sandy_target_required;
use super::textures::physical_array_layers;
use crate::rendering::wgsl_test_support::{assert_uniform_layout, parsed_wgsl};

const ITEM_WGSL: &str = include_str!("../../../../shaders/item.wgsl");
const SANDY_WGSL: &str = include_str!("../../../../shaders/sandy.wgsl");
const SANDY_BLIT_WGSL: &str = include_str!("../../../../shaders/sandy_blit.wgsl");
const TRANSITION_WGSL: &str = include_str!("../../../../shaders/transition.wgsl");

#[test]
fn array_layer_padding() {
    assert_eq!(physical_array_layers(0), 2);
    assert_eq!(physical_array_layers(1), 2);
    assert_eq!(physical_array_layers(2), 2);
    assert_eq!(physical_array_layers(6), 7);
    assert_eq!(physical_array_layers(12), 13);
    assert_eq!(physical_array_layers(7), 7);
}

#[test]
fn sandy_target_gating() {
    assert!(sandy_target_required(true, false, Some(0.5)));
    assert!(!sandy_target_required(false, false, Some(0.5)));
    assert!(!sandy_target_required(true, true, Some(0.5)));
    assert!(!sandy_target_required(true, false, Some(1.0)));
    assert!(!sandy_target_required(true, false, None));
}

#[test]
fn sandy_video_out_gates() {
    assert_eq!(sandy_video_out(true, false), 1.0);
    assert_eq!(sandy_video_out(false, false), 0.0);
    assert_eq!(sandy_video_out(true, true), 0.0);
    assert_eq!(sandy_video_out(false, true), 0.0);
}

#[test]
fn sandy_video_in_ramp() {
    assert_eq!(sandy_video_in(0.5, false), 0.0);
    assert_eq!(sandy_video_in(0.05, true), 0.0);
    assert_eq!(sandy_video_in(0.10, true), 0.0);
    assert_eq!(sandy_video_in(0.35, true), 1.0);
    assert_eq!(sandy_video_in(1.0, true), 1.0);
    let mid = sandy_video_in(0.225, true);
    assert!((mid - 0.5).abs() < 1e-6);
    assert!(sandy_video_in(0.15, true) < sandy_video_in(0.20, true));
}

#[test]
fn uniform_alignment() {
    assert_eq!(std::mem::size_of::<TransUniformRaw>() % 16, 0);
    assert_eq!(std::mem::size_of::<SandyUniformRaw>() % 16, 0);
    assert_eq!(std::mem::size_of::<Globals>() % 16, 0);
}

#[test]
fn every_shipped_shader_validates() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
    let mut validated = Vec::new();
    for entry in std::fs::read_dir(&directory).expect("read shaders directory") {
        let path = entry.expect("shader entry").path();
        if path.extension().is_none_or(|extension| extension != "wgsl") {
            continue;
        }
        let name =
            path.file_stem().and_then(|value| value.to_str()).expect("shader file stem").to_owned();
        let source = std::fs::read_to_string(&path).expect("read shader source");
        let module = parsed_wgsl(&name, &source);
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|err| panic!("{name}.wgsl validate: {err:?}"));
        validated.push(name);
    }
    validated.sort();
    assert!(validated.len() >= 5, "walked only {validated:?}");
}

#[test]
fn bind_groups_match_shaders() {
    let create_rs = include_str!("create.rs");
    let bindings_rs = include_str!("bindings.rs");
    let sandy_rs = include_str!("sandy_resources.rs");
    let transition_rs = include_str!("transition_resources.rs");
    for (shader, source, layout_source, layout_label, bind_source, bind_label) in [
        ("item", ITEM_WGSL, create_rs, "skwd scene layout", bindings_rs, "skwd scene bind"),
        (
            "transition",
            TRANSITION_WGSL,
            transition_rs,
            "skwd transition layout",
            transition_rs,
            "skwd transition bind",
        ),
        ("sandy", SANDY_WGSL, sandy_rs, "skwd sandy layout", sandy_rs, "skwd sandy bind"),
        (
            "sandy_blit",
            SANDY_BLIT_WGSL,
            sandy_rs,
            "skwd sandy blit layout",
            sandy_rs,
            "skwd sandy blit bind",
        ),
    ] {
        let declared = shader_bindings(&parsed_wgsl(shader, source), 0);
        assert_eq!(
            binding_indices(descriptor_block(layout_source, layout_label)),
            declared,
            "{layout_label} vs {shader}"
        );
        assert_eq!(
            binding_indices(descriptor_block(bind_source, bind_label)),
            declared,
            "{bind_label} vs {shader}"
        );
    }
}

#[test]
fn sandy_ring_curve() {
    let module = parsed_wgsl("sandy", SANDY_WGSL);
    let vertex = entry_function(&module, "vs_main");
    assert!(local_variable(vertex, "ring_curve17").is_some());
    for binding in ["ring_axes", "ring_waypoint"] {
        assert!(named_expression(vertex, binding).is_some(), "{binding}");
    }
    assert_eq!(stores_loaded_local(vertex, "guide", "ring_curve17"), 1);
}

#[test]
fn ring_size_clamp_agrees() {
    let module = parsed_wgsl("sandy", SANDY_WGSL);
    let vertex = entry_function(&module, "vs_main");
    let member = uniform_member_index(&module, "SandyUniform", "ring_size");
    let bounds = clamp_bounds_on_uniform_member(&module, vertex, member);
    assert_eq!(bounds.len(), 2);
    let (low, high) = bounds[0];
    assert_eq!(bounds[1], (low, high));

    let expected = format!(".clamp({low:?}, {high:?})");
    let accessor = include_str!("../../../infrastructure/config/picker/accessors/sandy.rs");
    let start = accessor.find("fn sandy_ring_size").expect("sandy_ring_size accessor present");
    let body = &accessor[start..start + accessor[start..].find("\n    }").expect("accessor body")];
    assert!(body.contains(&expected), "accessor missing {expected}");

    let view = include_str!("../../../app/scene/core/view_sandy.rs");
    assert!(view.contains(&format!("ring_size{expected}")), "snapshot missing {expected}");
}

#[test]
fn diamond_half_planes() {
    let module = parsed_wgsl("item", ITEM_WGSL);
    let diamond = module_function(&module, "sd_diamond");
    assert_eq!(math_uses(diamond, naga::MathFunction::Dot), 4);
    assert_eq!(math_uses(diamond, naga::MathFunction::Abs), 0);
}

#[test]
fn flip_overwrites_base() {
    let module = parsed_wgsl("item", ITEM_WGSL);
    let fragment = entry_function(&module, "fs_main");
    assert_eq!(stores_call_result(&module, fragment, "base", "flip_color"), 1);
}

#[test]
fn flip_keeps_effect_output() {
    assert!(ITEM_WGSL.contains("return col;"));
    assert!(!ITEM_WGSL.contains("mix(col, sharp, smoothstep"));
}

#[test]
fn ring_dust_into_mid() {
    let module = parsed_wgsl("sandy", SANDY_WGSL);
    let vertex = entry_function(&module, "vs_main");
    assert_eq!(stores_max_of_self_and(vertex, "extra_mid", "dust17"), 1);
}

fn descriptor_block<'a>(source: &'a str, label: &str) -> &'a str {
    let needle = format!("label: Some(\"{label}\")");
    let at = source
        .find(&needle)
        .unwrap_or_else(|| panic!("descriptor labelled {label} not found in source"));
    let open = source[..at].rfind('{').expect("descriptor opening brace");
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    for index in open..bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..=index];
                }
            }
            _ => {}
        }
    }
    panic!("descriptor labelled {label} is not closed");
}

fn binding_indices(block: &str) -> Vec<u32> {
    let mut indices: Vec<u32> = block
        .match_indices("binding: ")
        .map(|(at, needle)| {
            block[at + needle.len()..]
                .split(',')
                .next()
                .expect("binding index")
                .trim()
                .parse()
                .expect("binding index is a number")
        })
        .collect();
    indices.sort_unstable();
    indices
}

fn shader_bindings(module: &naga::Module, group: u32) -> Vec<u32> {
    let mut indices: Vec<u32> = module
        .global_variables
        .iter()
        .filter_map(|(_, global)| global.binding.as_ref())
        .filter(|resource| resource.group == group)
        .map(|resource| resource.binding)
        .collect();
    indices.sort_unstable();
    indices
}

fn entry_function<'a>(module: &'a naga::Module, name: &str) -> &'a naga::Function {
    let entry = module
        .entry_points
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("entry point {name} not found in shader"));
    &entry.function
}

fn module_function<'a>(module: &'a naga::Module, name: &str) -> &'a naga::Function {
    let (_, function) = module
        .functions
        .iter()
        .find(|(_, function)| function.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("fn {name} not found in shader"));
    function
}

fn module_function_handle(module: &naga::Module, name: &str) -> naga::Handle<naga::Function> {
    let (handle, _) = module
        .functions
        .iter()
        .find(|(_, function)| function.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("fn {name} not found in shader"));
    handle
}

fn local_variable(
    function: &naga::Function,
    name: &str,
) -> Option<naga::Handle<naga::LocalVariable>> {
    function
        .local_variables
        .iter()
        .find(|(_, local)| local.name.as_deref() == Some(name))
        .map(|(handle, _)| handle)
}

fn named_expression(
    function: &naga::Function,
    name: &str,
) -> Option<naga::Handle<naga::Expression>> {
    function
        .named_expressions
        .iter()
        .find(|(_, binding)| binding.as_str() == name)
        .map(|(handle, _)| *handle)
}

fn statements<'a>(block: &'a naga::Block, out: &mut Vec<&'a naga::Statement>) {
    for statement in block {
        out.push(statement);
        match statement {
            naga::Statement::Block(inner) => statements(inner, out),
            naga::Statement::If { accept, reject, .. } => {
                statements(accept, out);
                statements(reject, out);
            }
            naga::Statement::Switch { cases, .. } => {
                for case in cases {
                    statements(&case.body, out);
                }
            }
            naga::Statement::Loop { body, continuing, .. } => {
                statements(body, out);
                statements(continuing, out);
            }
            _ => {}
        }
    }
}

fn body_statements(function: &naga::Function) -> Vec<&naga::Statement> {
    let mut out = Vec::new();
    statements(&function.body, &mut out);
    out
}

fn stores_into(function: &naga::Function, local: &str) -> Vec<naga::Handle<naga::Expression>> {
    let Some(target) = local_variable(function, local) else {
        return Vec::new();
    };
    body_statements(function)
        .into_iter()
        .filter_map(|statement| match statement {
            naga::Statement::Store { pointer, value } => {
                matches!(function.expressions[*pointer], naga::Expression::LocalVariable(handle) if handle == target)
                    .then_some(*value)
            }
            _ => None,
        })
        .collect()
}

fn loads_local(
    function: &naga::Function,
    expression: naga::Handle<naga::Expression>,
    local: naga::Handle<naga::LocalVariable>,
) -> bool {
    match function.expressions[expression] {
        naga::Expression::Load { pointer } => {
            matches!(function.expressions[pointer], naga::Expression::LocalVariable(handle) if handle == local)
        }
        _ => false,
    }
}

fn stores_loaded_local(function: &naga::Function, pointer: &str, value: &str) -> usize {
    let Some(source) = local_variable(function, value) else {
        return 0;
    };
    stores_into(function, pointer)
        .into_iter()
        .filter(|stored| loads_local(function, *stored, source))
        .count()
}

fn stores_max_of_self_and(function: &naga::Function, local: &str, other: &str) -> usize {
    let Some(target) = local_variable(function, local) else {
        return 0;
    };
    let Some(addend) = named_expression(function, other) else {
        return 0;
    };
    stores_into(function, local)
        .into_iter()
        .filter(|stored| match function.expressions[*stored] {
            naga::Expression::Math { fun: naga::MathFunction::Max, arg, arg1, .. } => {
                loads_local(function, arg, target) && arg1 == Some(addend)
            }
            _ => false,
        })
        .count()
}

fn stores_call_result(
    module: &naga::Module,
    function: &naga::Function,
    local: &str,
    callee: &str,
) -> usize {
    let called = module_function_handle(module, callee);
    stores_into(function, local)
        .into_iter()
        .filter(|stored| match &function.expressions[*stored] {
            naga::Expression::Compose { components, .. } => components.iter().any(|component| {
                matches!(function.expressions[*component], naga::Expression::CallResult(handle) if handle == called)
            }),
            _ => false,
        })
        .count()
}

fn math_uses(function: &naga::Function, wanted: naga::MathFunction) -> usize {
    function
        .expressions
        .iter()
        .filter(|(_, expression)| {
            matches!(expression, naga::Expression::Math { fun, .. } if *fun == wanted)
        })
        .count()
}

fn uniform_member_index(module: &naga::Module, structure: &str, member: &str) -> u32 {
    for (_, ty) in module.types.iter() {
        if ty.name.as_deref() == Some(structure)
            && let naga::TypeInner::Struct { members, .. } = &ty.inner
        {
            let index = members
                .iter()
                .position(|field| field.name.as_deref() == Some(member))
                .unwrap_or_else(|| panic!("{structure}.{member} not found in shader"));
            return u32::try_from(index).expect("member index fits u32");
        }
    }
    panic!("struct {structure} not found in shader");
}

fn literal_f32(function: &naga::Function, expression: naga::Handle<naga::Expression>) -> f32 {
    match function.expressions[expression] {
        naga::Expression::Literal(naga::Literal::F32(value)) => value,
        naga::Expression::Literal(naga::Literal::AbstractFloat(value)) => value as f32,
        ref other => panic!("expected a float literal clamp bound, found {other:?}"),
    }
}

fn clamp_bounds_on_uniform_member(
    module: &naga::Module,
    function: &naga::Function,
    member: u32,
) -> Vec<(f32, f32)> {
    let reads_member = |expression: naga::Handle<naga::Expression>| {
        let naga::Expression::Load { pointer } = function.expressions[expression] else {
            return false;
        };
        let naga::Expression::AccessIndex { base, index } = function.expressions[pointer] else {
            return false;
        };
        let naga::Expression::GlobalVariable(global) = function.expressions[base] else {
            return false;
        };
        index == member && module.global_variables[global].space == naga::AddressSpace::Uniform
    };
    function
        .expressions
        .iter()
        .filter_map(|(_, expression)| match expression {
            naga::Expression::Math { fun: naga::MathFunction::Clamp, arg, arg1, arg2, .. }
                if reads_member(*arg) =>
            {
                Some((
                    literal_f32(function, arg1.expect("clamp low bound")),
                    literal_f32(function, arg2.expect("clamp high bound")),
                ))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn globals_layout() {
    use std::mem::offset_of;
    assert_uniform_layout(
        "item",
        ITEM_WGSL,
        "Globals",
        &[
            ("resolution", offset_of!(Globals, resolution)),
            ("time", offset_of!(Globals, time)),
            ("vis", offset_of!(Globals, vis)),
            ("clip", offset_of!(Globals, clip)),
        ],
        std::mem::size_of::<Globals>(),
    );
}

#[test]
fn trans_uniform_layout() {
    use std::mem::offset_of;
    assert_uniform_layout(
        "transition",
        TRANSITION_WGSL,
        "TransUniform",
        &[
            ("resolution", offset_of!(TransUniformRaw, resolution)),
            ("origin", offset_of!(TransUniformRaw, origin)),
            ("progress", offset_of!(TransUniformRaw, progress)),
            ("kind", offset_of!(TransUniformRaw, kind)),
            ("_pad0", offset_of!(TransUniformRaw, _pad0)),
            ("_pad1", offset_of!(TransUniformRaw, _pad1)),
            ("accent", offset_of!(TransUniformRaw, accent)),
        ],
        std::mem::size_of::<TransUniformRaw>(),
    );
}

#[test]
fn sandy_uniform_layout() {
    use std::mem::offset_of;
    assert_uniform_layout(
        "sandy",
        SANDY_WGSL,
        "SandyUniform",
        &[
            ("center", offset_of!(SandyUniformRaw, center)),
            ("resolution", offset_of!(SandyUniformRaw, resolution)),
            ("hero", offset_of!(SandyUniformRaw, hero)),
            ("dir", offset_of!(SandyUniformRaw, dir)),
            ("layer_a", offset_of!(SandyUniformRaw, layer_a)),
            ("layer_b", offset_of!(SandyUniformRaw, layer_b)),
            ("progress", offset_of!(SandyUniformRaw, progress)),
            ("time", offset_of!(SandyUniformRaw, time)),
            ("vis", offset_of!(SandyUniformRaw, vis)),
            ("seed", offset_of!(SandyUniformRaw, seed)),
            ("strands", offset_of!(SandyUniformRaw, strands)),
            ("twist", offset_of!(SandyUniformRaw, twist)),
            ("orbit", offset_of!(SandyUniformRaw, orbit)),
            ("turbulence", offset_of!(SandyUniformRaw, turbulence)),
            ("waist", offset_of!(SandyUniformRaw, waist)),
            ("front", offset_of!(SandyUniformRaw, front)),
            ("fan", offset_of!(SandyUniformRaw, fan)),
            ("carry", offset_of!(SandyUniformRaw, carry)),
            ("layer_b2", offset_of!(SandyUniformRaw, layer_b2)),
            ("layer_b3", offset_of!(SandyUniformRaw, layer_b3)),
            ("bcut", offset_of!(SandyUniformRaw, bcut)),
            ("bmix", offset_of!(SandyUniformRaw, bmix)),
            ("arc", offset_of!(SandyUniformRaw, arc)),
            ("swap_loop", offset_of!(SandyUniformRaw, swap_loop)),
            ("swirl", offset_of!(SandyUniformRaw, swirl)),
            ("wave_flag", offset_of!(SandyUniformRaw, wave_flag)),
            ("ring_spin", offset_of!(SandyUniformRaw, ring_spin)),
            ("ring_wave", offset_of!(SandyUniformRaw, ring_wave)),
            ("ring_soft", offset_of!(SandyUniformRaw, ring_soft)),
            ("grid", offset_of!(SandyUniformRaw, grid)),
            ("swap_style", offset_of!(SandyUniformRaw, swap_style)),
            ("video_in", offset_of!(SandyUniformRaw, video_in)),
            ("video_out", offset_of!(SandyUniformRaw, video_out)),
            ("ring_size", offset_of!(SandyUniformRaw, ring_size)),
            ("_pad0", offset_of!(SandyUniformRaw, _pad0)),
            ("_pad1", offset_of!(SandyUniformRaw, _pad1)),
        ],
        std::mem::size_of::<SandyUniformRaw>(),
    );
}

#[test]
fn sandy_vert_count() {
    assert_eq!(sandy_vertices([16.0, 9.0]), 16 * 9 * 2 * 6, "instancing measured 5x slower");
    assert_eq!(sandy_vertices([0.0, 9.0]), 9 * 12);
    assert_eq!(sandy_vertices([0.0, 0.0]), 12);
}

#[test]
fn sandy_scaled_only_below_full() {
    assert!(!sandy_is_scaled(1.0));
    assert!(!sandy_is_scaled(0.999));
    assert!(sandy_is_scaled(0.75));
    assert!(sandy_is_scaled(0.25));
}

#[test]
fn sandy_target_dims_ceil() {
    assert_eq!(sandy_target_dimensions(1920.0, 1080.0, 1.0, 1.0), (1920, 1080));
    assert_eq!(sandy_target_dimensions(1920.0, 1080.0, 1.0, 0.5), (960, 540));
    assert_eq!(sandy_target_dimensions(1000.0, 1000.0, 1.0, 0.33), (330, 330));
    assert_eq!(sandy_target_dimensions(1920.0, 1080.0, 2.0, 0.5), (1920, 1080));
    let (w, h) = sandy_target_dimensions(1.0, 1.0, 1.0, 0.25);
    assert!(w >= 1 && h >= 1);
}

#[test]
fn chamfer_flag_matches_shader() {
    let flag = crate::contracts::rendering::CHAMFER;
    assert!(ITEM_WGSL.contains(&format!("const CHAMFER: u32 = {flag}u;")));
    let module = parsed_wgsl("item", ITEM_WGSL);
    for name in ["sd_card", "sd_sheared_chamfered_box", "sd_sheared_rounded_box"] {
        module_function(&module, name);
    }
}
