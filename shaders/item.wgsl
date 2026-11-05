struct Globals {
    resolution: vec2<f32>,
    time: f32,
    vis: f32,
    clip: vec4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var near_tex: texture_2d_array<f32>;
@group(0) @binding(2) var far_tex: texture_2d_array<f32>;
@group(0) @binding(3) var samp: sampler;
@group(0) @binding(4) var preview_tex: texture_2d<f32>;

struct Instance {
    @location(0) rect: vec4<f32>,
    @location(1) radii: vec4<f32>,
    @location(2) fill: vec4<f32>,
    @location(3) tint: vec4<f32>,
    @location(4) border: vec4<f32>,
    @location(5) params: vec4<f32>,
    @location(6) uv: vec4<f32>,
    @location(7) crop: vec4<f32>,
    @location(8) misc: vec4<u32>,
    @location(9) flip: vec4<f32>,
    @location(10) shape: vec4<f32>,
    @location(11) quad_a: vec4<f32>,
    @location(12) quad_b: vec4<f32>,
    @location(13) quad_w: vec4<f32>,
    @location(14) quad_l: vec4<f32>,
}

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) half_ext: vec2<f32>,
    @location(2) radii: vec4<f32>,
    @location(3) fill: vec4<f32>,
    @location(4) tint: vec4<f32>,
    @location(5) border: vec4<f32>,
    @location(6) params: vec4<f32>,
    @location(7) uv: vec4<f32>,
    @location(8) crop: vec4<f32>,
    @location(9) @interpolate(flat) misc: vec4<u32>,
    @location(10) world: vec2<f32>,
    @location(11) flip: vec4<f32>,
    @location(12) shape: vec4<f32>,
    @location(13) @interpolate(flat) quad_l: vec4<f32>,
}

const PROJECTED: u32 = 128u;
const RIBBON_COLUMNS: u32 = 4096u;
const GHOST: u32 = 8192u;
const BACKDROP: u32 = 16384u;
const BACKFACE: u32 = 32768u;
const MUTED: u32 = 65536u;
const UNFRAMED_RECT: u32 = 131072u;
const CHAMFER: u32 = 262144u;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: Instance) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
        vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0),
    );
    let c = corners[vi];
    let half_bb = vec2(inst.rect.z, inst.rect.w) + vec2(2.0, 2.0);
    var local = c * half_bb;
    var world = inst.rect.xy + local;
    var w = 1.0;
    if ((inst.misc.w & PROJECTED) != 0u) {
        var order = array<u32, 6>(0u, 1u, 3u, 1u, 2u, 3u);
        let ci = order[vi];
        var screen = array<vec2<f32>, 4>(inst.quad_a.xy, inst.quad_a.zw, inst.quad_b.xy, inst.quad_b.zw);
        var depth = array<f32, 4>(inst.quad_w.x, inst.quad_w.y, inst.quad_w.z, inst.quad_w.w);
        var along = array<f32, 4>(inst.quad_l.x, inst.quad_l.y, inst.quad_l.z, inst.quad_l.w);
        world = screen[ci];
        w = depth[ci];
        if ((inst.misc.w & RIBBON_COLUMNS) != 0u) {
            local = vec2(along[ci], c.y * half_bb.y);
        } else {
            local = vec2(c.x * (half_bb.x + inst.shape.y), along[ci]);
        }
    }
    let ndc = vec2(
        world.x / globals.resolution.x * 2.0 - 1.0,
        1.0 - world.y / globals.resolution.y * 2.0,
    );

    var out: VsOut;
    out.pos = vec4(ndc * w, 0.0, w);
    out.local = local;
    out.half_ext = vec2(inst.rect.z, inst.rect.w);
    out.radii = inst.radii;
    out.fill = inst.fill;
    out.tint = inst.tint;
    out.border = inst.border;
    out.params = inst.params;
    out.uv = inst.uv;
    out.crop = inst.crop;
    out.misc = inst.misc;
    out.world = world;
    out.flip = inst.flip;
    out.shape = inst.shape;
    out.quad_l = inst.quad_l;
    return out;
}

fn ribbon_cut(in: VsOut) -> f32 {
    let he = max(in.half_ext, vec2(1.0));
    if ((in.misc.w & RIBBON_COLUMNS) != 0u) {
        let f = clamp(in.local.y / he.y * 0.5 + 0.5, -0.5, 1.5);
        let left = mix(in.quad_l.x, in.quad_l.w, f);
        let right = mix(in.quad_l.y, in.quad_l.z, f);
        return max(left - in.local.x, in.local.x - right);
    }
    let f = clamp(in.local.x / he.x * 0.5 + 0.5, -0.5, 1.5);
    let top = mix(in.quad_l.x, in.quad_l.y, f);
    let bottom = mix(in.quad_l.w, in.quad_l.z, f);
    return max(top - in.local.y, in.local.y - bottom);
}

fn backdrop_blur(in: VsOut, norm: vec2<f32>, radius: f32) -> vec3<f32> {
    var acc = vec3(0.0);
    for (var i = 0; i < 16; i++) {
        let fi = f32(i) + 0.5;
        let ang = fi * 2.39996323;
        let r = sqrt(fi / 16.0) * radius;
        acc += near_lod(in, norm + vec2(cos(ang), sin(ang)) * r, 4.0);
    }
    return acc / 16.0;
}

fn ghost_blur(in: VsOut, norm: vec2<f32>, radius: f32) -> vec3<f32> {
    let aspect = max(in.half_ext.x, 1.0) / max(in.half_ext.y, 1.0);
    var acc = vec3(0.0);
    for (var i = 0; i < 12; i++) {
        let fi = f32(i) + 0.5;
        let ang = fi * 2.39996323;
        let r = sqrt(fi / 12.0) * radius;
        let p = norm + vec2(cos(ang), sin(ang) * aspect) * r;
        if (in.misc.x == 1u) {
            acc += near_lod(in, p, 1.0);
        } else {
            acc += sample_card(in, p);
        }
    }
    return acc / 12.0;
}

fn backdrop_grade(in: VsOut, rgb_in: vec3<f32>) -> vec3<f32> {
    var rgb = (rgb_in - vec3(0.5)) * 1.25 + vec3(0.5);
    rgb = max(rgb, vec3(0.0)) * 0.38;
    let norm = in.local / (2.0 * max(in.half_ext, vec2(1.0))) + vec2(0.5);
    let p = length((norm - vec2(0.5, 0.44)) / vec2(0.7, 0.6));
    var veil = 0.72 * clamp(p / 0.7, 0.0, 1.0);
    veil = veil + 0.24 * clamp((p - 0.7) / 0.3, 0.0, 1.0);
    rgb = mix(rgb, vec3(4.0, 4.0, 6.0) / 255.0, clamp(veil, 0.0, 0.96));
    let line = step(fract(in.world.y / 3.0), 0.34) - 0.34;
    return rgb + vec3(line * 0.008);
}

struct Sheared {
    q: vec2<f32>,
    b: vec2<f32>,
    lean: vec2<f32>,
}

fn sheared(p: vec2<f32>, b_in: vec2<f32>, skew: f32, edge_tilt: f32) -> Sheared {
    let sx = skew * 0.5;
    let ty = edge_tilt * 0.5;
    let b = max(b_in - abs(vec2(sx, ty)), vec2(1.0));
    let det = b.x * b.y - sx * ty;
    let safe_det = select(det, select(-1.0, 1.0, det >= 0.0), abs(det) < 1.0);
    let q = vec2(
        (b.y * p.x + sx * p.y) / safe_det * b.x,
        (ty * p.x + b.x * p.y) / safe_det * b.y,
    );
    return Sheared(q, b, vec2(sx, ty));
}

fn corner_size(q: vec2<f32>, sizes: vec4<f32>) -> f32 {
    if (q.y < 0.0) {
        return select(sizes.y, sizes.x, q.x < 0.0);
    }
    return select(sizes.z, sizes.w, q.x < 0.0);
}

fn sd_sheared_rounded_box(
    p: vec2<f32>,
    b_in: vec2<f32>,
    radii: vec4<f32>,
    skew: f32,
    edge_tilt: f32,
) -> f32 {
    let s = sheared(p, b_in, skew, edge_tilt);
    let r = corner_size(s.q, radii);
    let d = abs(s.q) - s.b + vec2(r);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2(0.0))) - r;
}

fn sd_sheared_chamfered_box(
    p: vec2<f32>,
    b_in: vec2<f32>,
    cuts: vec4<f32>,
    skew: f32,
    edge_tilt: f32,
) -> f32 {
    let s = sheared(p, b_in, skew, edge_tilt);
    let a = abs(s.q);
    let e = a - s.b;
    let boxed = min(max(e.x, e.y), 0.0) + length(max(e, vec2(0.0)));
    let c = corner_size(s.q, cuts);
    if (c <= 0.0) {
        return boxed;
    }
    let edges = vec2(length(vec2(s.b.x, s.lean.y)), length(vec2(s.lean.x, s.b.y)));
    let cut = c * s.b / edges;
    let n = normalize(vec2(cut.y, cut.x));
    return max(boxed, dot(a - vec2(s.b.x - cut.x, s.b.y), n));
}

fn sd_card(
    p: vec2<f32>,
    b_in: vec2<f32>,
    sizes: vec4<f32>,
    skew: f32,
    edge_tilt: f32,
    flags: u32,
) -> f32 {
    if ((flags & CHAMFER) != 0u) {
        return sd_sheared_chamfered_box(p, b_in, sizes, skew, edge_tilt);
    }
    return sd_sheared_rounded_box(p, b_in, sizes, skew, edge_tilt);
}

fn sd_hexagon_flat(p: vec2<f32>, circumradius: f32) -> f32 {
    let a = circumradius * 0.866025;
    let q = abs(p);
    return max(dot(q, vec2(0.866025, 0.5)), q.y) - a;
}

fn sd_triangle(p: vec2<f32>, half_ext: vec2<f32>, direction: u32) -> f32 {
    var q = p / max(half_ext, vec2(1.0));
    if (direction == 1u) {
        q.y = -q.y;
    } else if (direction == 2u) {
        q = vec2(q.y, q.x);
    } else if (direction == 3u) {
        q = vec2(q.y, -q.x);
    }
    let side = (2.0 * abs(q.x) - q.y - 1.0) / 2.2360679775;
    return max(side, q.y - 1.0) * min(half_ext.x, half_ext.y);
}

fn sd_diamond(p: vec2<f32>, half_ext: vec2<f32>) -> f32 {
    let he = max(half_ext, vec2(1.0));
    let normal_len = length(vec2(he.y, he.x));
    let edge = he.x * he.y;
    let top_right = dot(p, vec2(he.y, he.x)) - edge;
    let bottom_right = dot(p, vec2(he.y, -he.x)) - edge;
    let top_left = dot(p, vec2(-he.y, he.x)) - edge;
    let bottom_left = dot(p, vec2(-he.y, -he.x)) - edge;
    return max(max(top_right, bottom_right), max(top_left, bottom_left)) / normal_len;
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

fn vnoise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i), hash21(i + vec2(1.0, 0.0)), u.x),
        mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), u.x),
        u.y,
    );
}

fn sample_card(in: VsOut, norm: vec2<f32>) -> vec3<f32> {
    let n = clamp(norm, vec2(0.0), vec2(1.0));
    let cropped = in.crop.xy + n * in.crop.zw;
    let uv = in.uv.xy + cropped * in.uv.zw;
    if (in.misc.x == 1u) {
        return textureSampleLevel(near_tex, samp, uv, in.misc.y, 0.0).rgb;
    }
    if (in.misc.x == 3u) {
        return textureSampleLevel(preview_tex, samp, uv, 0.0).rgb;
    }
    return textureSampleLevel(far_tex, samp, uv, in.misc.y, 0.0).rgb;
}

fn near_lod(in: VsOut, norm: vec2<f32>, lod: f32) -> vec3<f32> {
    let n = clamp(norm, vec2(0.0), vec2(1.0));
    let cropped = in.crop.xy + n * in.crop.zw;
    let uv = in.uv.xy + cropped * in.uv.zw;
    return textureSampleLevel(near_tex, samp, uv, in.misc.y, lod).rgb;
}

fn card_dof(in: VsOut, norm: vec2<f32>, blur: f32, ca: f32) -> vec3<f32> {
    let dirn = in.local / max(in.half_ext, vec2(1.0));
    let ca_off = dirn * ca * 0.016;
    if (in.misc.x == 1u) {
        let lod = blur * 3.4;
        if (ca > 0.01) {
            let r = near_lod(in, norm + ca_off, lod).r;
            let g = near_lod(in, norm, lod).g;
            let b = near_lod(in, norm - ca_off, lod).b;
            return vec3(r, g, b);
        }
        return near_lod(in, norm, lod);
    }
    var offs = array<vec2<f32>, 5>(
        vec2(0.0, 0.0),
        vec2(0.85, 0.3),
        vec2(-0.3, 0.85),
        vec2(-0.85, -0.3),
        vec2(0.3, -0.85),
    );
    let rad = blur * 0.05;
    var acc = vec3(0.0);
    for (var t: i32 = 0; t < 5; t++) {
        acc += sample_card(in, norm + offs[t] * rad);
    }
    var rgb = acc / 5.0;
    if (ca > 0.01) {
        let r = sample_card(in, norm + ca_off).r;
        let b = sample_card(in, norm - ca_off).b;
        rgb = vec3(mix(rgb.r, r, 0.7), rgb.g, mix(rgb.b, b, 0.7));
    }
    return rgb;
}

fn cnoise(p: vec2<f32>) -> f32 {
    return vnoise2(p) * 0.65 + vnoise2(p * 2.13 + vec2(11.5, 3.7)) * 0.35;
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.299, 0.587, 0.114));
}

fn flip_color(in: VsOut, p: f32) -> vec3<f32> {
    let eff = i32(in.flip.z + 0.5);
    let norm = clamp(in.local / (2.0 * in.half_ext) + vec2(0.5), vec2(0.0), vec2(1.0));
    let t = globals.time;
    let seed = vec2(in.flip.y * 1.7, in.flip.y * 0.9 + 4.0);
    let sharp = sample_card(in, norm);
    var bacc = vec3(0.0);
    bacc = bacc + sample_card(in, norm + vec2(0.012, 0.0));
    bacc = bacc + sample_card(in, norm + vec2(-0.012, 0.0));
    bacc = bacc + sample_card(in, norm + vec2(0.0, 0.012));
    bacc = bacc + sample_card(in, norm + vec2(0.0, -0.012));
    let back = (bacc / 4.0) * 0.2;
    var col = sharp;

    if (eff == 0) {
        let lum = luma(sharp);
        let n = cnoise(norm * 7.0 + seed) * 0.12;
        let field = (1.0 - lum) * 0.8 + n;
        let burn = p * 1.25 - 0.12;
        let e = field - burn;
        let intact = smoothstep(0.0, 0.05, e);
        let g1 = e / 0.045;
        let glow = exp(-g1 * g1);
        let g2 = e / 0.015;
        let core = exp(-g2 * g2);
        var c = mix(back, sharp, intact);
        c = mix(c, c * vec3(0.25, 0.18, 0.15), smoothstep(0.05, -0.02, e) * intact);
        c = c + vec3(1.0, 0.5, 0.12) * glow * 1.3 + vec3(1.0, 0.9, 0.5) * core * 0.8;
        let spk = hash21(floor((norm - vec2(0.0, p * 0.1)) * 200.0) + seed);
        c = c + vec3(1.0, 0.7, 0.3) * step(0.992, spk) * glow * 1.5;
        col = c;
    } else if (eff == 1) {
        let o = 0.004;
        let lx1 = luma(sample_card(in, norm + vec2(o, 0.0)));
        let lx0 = luma(sample_card(in, norm - vec2(o, 0.0)));
        let ly1 = luma(sample_card(in, norm + vec2(0.0, o)));
        let ly0 = luma(sample_card(in, norm - vec2(0.0, o)));
        let grad = vec2(lx1 - lx0, ly1 - ly0);
        let edge = clamp(length(grad) * 7.0, 0.0, 1.0);
        let field = (1.0 - edge) * 0.7 + cnoise(norm * 9.0 + seed) * 0.3;
        let front = p * 1.2 - 0.1;
        let m = smoothstep(front, front + 0.08, field);
        let src = sample_card(in, norm + normalize(grad + vec2(1e-4, 0.0)) * (1.0 - m) * 0.06);
        col = mix(back, src, m) + vec3(0.5, 0.75, 1.0) * edge * (1.0 - m) * m * 3.5;
    } else if (eff == 2) {
        let lum = luma(sharp);
        let field = mix((norm.x + norm.y) * 0.5, 1.0 - lum, 0.75);
        let front = p * 1.2 - 0.1;
        let m = smoothstep(front - 0.04, front + 0.04, field);
        let g = (field - front) / 0.04;
        let glow = exp(-g * g);
        col = mix(back, sharp, m) + vec3(0.7, 0.85, 1.0) * glow * 0.5;
    } else if (eff == 4) {
        let depth = luma(sharp);
        let push = p * 0.18;
        let disp = (norm - vec2(0.5)) * (depth * push * 1.5 + push * 0.2);
        let s = sample_card(in, norm + disp);
        let shade = mix(0.6, 1.1, depth);
        let dim = mix(1.0, 0.42, smoothstep(0.3, 1.0, p));
        col = s * shade * dim;
    } else if (eff == 5) {
        let r = 0.012 + p * 0.07;
        var acc = vec3(0.0);
        var wsum = 0.0;
        for (var i = 0; i < 40; i = i + 1) {
            let fi = f32(i) + 0.5;
            let ang = fi * 2.39996323;
            let rad = sqrt(fi / 40.0) * r;
            let s = sample_card(in, norm + vec2(cos(ang), sin(ang)) * rad);
            let l = luma(s);
            let w = pow(l + 0.02, 4.0);
            acc = acc + s * w;
            wsum = wsum + w;
        }
        let bok = acc / max(wsum, 0.0001);
        let mixed = mix(sharp, bok, smoothstep(0.0, 0.45, p));
        let glow = mixed * (1.0 + smoothstep(0.55, 1.0, luma(bok)) * 0.9);
        col = glow * mix(1.0, 0.55, smoothstep(0.3, 1.0, p));
    } else if (eff == 6) {
        let dir = normalize(vec2(0.55, -0.83));
        let len = p * 0.55;
        var acc = vec3(0.0);
        var w = 0.0;
        for (var i = 0; i < 16; i = i + 1) {
            let f = f32(i) / 16.0;
            let s = sample_card(in, norm - dir * len * f);
            let l = luma(s);
            let wt = l * l * (1.0 - f) + 0.001;
            acc = acc + s * wt;
            w = w + wt;
        }
        let streak = acc / w;
        let base = sharp + streak * smoothstep(0.0, 0.55, p) * 1.3;
        col = base * mix(1.0, 0.5, smoothstep(0.3, 1.0, p));
    } else if (eff == 7) {
        let n = mix(70.0, 22.0, p);
        let cell = floor(norm * n);
        let tile = (cell + vec2(0.5)) / n;
        let tl = luma(sample_card(in, tile));
        let local = fract(norm * n) - vec2(0.5);
        let bevel = clamp(1.0 - (abs(local.x) + abs(local.y)) * 0.8, 0.45, 1.25);
        let s = sample_card(in, tile) * bevel * (0.7 + tl * 0.7);
        col = s * mix(1.0, 0.5, smoothstep(0.3, 1.0, p));
    } else if (eff == 8) {
        let amt = p * 0.6;
        var acc = vec3(0.0);
        var w = 0.0;
        for (var i = 0; i < 16; i = i + 1) {
            let f = f32(i) / 16.0;
            let s = sample_card(in, vec2(norm.x, clamp(norm.y - amt * f, 0.0, 1.0)));
            let l = luma(s);
            let wt = smoothstep(0.45, 1.0, l) + 0.03;
            acc = acc + s * wt;
            w = w + wt;
        }
        let sorted = acc / w;
        let base = mix(sharp, sorted, smoothstep(0.0, 0.45, p));
        col = base * mix(1.0, 0.5, smoothstep(0.3, 1.0, p));
    } else if (eff == 9) {
        let r = p * 0.05;
        var acc = vec3(0.0);
        for (var i = 0; i < 24; i = i + 1) {
            let fi = f32(i) + 0.5;
            let ang = fi * 2.39996323;
            let rad = sqrt(fi / 24.0) * r;
            acc = acc + sample_card(in, norm + vec2(cos(ang), sin(ang)) * rad);
        }
        col = (acc / 24.0) * mix(1.0, 0.5, smoothstep(0.3, 1.0, p));
    } else if (eff == 10) {
        let l = luma(sharp);
        let d = abs(fract(l * 15.0) - 0.5) * 2.0;
        let contour = 1.0 - smoothstep(0.0, 0.14, d);
        let glowcol = mix(sharp * 0.22, vec3(0.45, 0.85, 1.0), contour);
        let topo = mix(sharp, glowcol, smoothstep(0.1, 0.55, p));
        col = topo * mix(1.0, 0.5, smoothstep(0.3, 1.0, p));
    } else if (eff == 11) {
        let lum = luma(sharp);
        let field = 1.0 - lum;
        let front = p * 1.2 - 0.1;
        let m = smoothstep(front - 0.04, front + 0.04, field);
        let g = (field - front) / 0.04;
        let glow = exp(-g * g);
        col = mix(back, sharp, m) + vec3(0.7, 0.85, 1.0) * glow * 0.5;
    } else {
        let lum = luma(sharp);
        let order = (1.0 - lum) * 0.85 + cnoise(norm * 8.0 + seed) * 0.15;
        let burn = p * 1.2 - 0.1;
        let age = clamp((burn - order) * 2.2, 0.0, 1.0);
        if (age <= 0.0) {
            col = sharp;
        } else {
            let drift = vec2((cnoise(norm * 5.0 + seed) - 0.5) * 0.6, -0.5) * age * 0.13;
            let src = sample_card(in, norm - drift);
            let grain = hash21(floor((norm - drift) * 150.0) + seed);
            let a = step(age, grain) * (1.0 - age);
            let e = (age - 0.1) / 0.12;
            let glow = exp(-e * e);
            col = mix(back, src + vec3(0.8, 0.4, 0.15) * glow, a);
            col = col + vec3(1.0, 0.85, 0.6) * step(0.99, grain) * glow * 0.7;
        }
    }
    return col;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let skew = in.params.x;
    let border_w = in.params.y;
    let opacity = in.params.z;
    let fade = in.params.w;
    if ((in.misc.w & 64u) == 64u) {
        let blur = max(border_w, 1.0);
        let he = max(in.half_ext - vec2(blur * 0.8), vec2(1.0));
        let ds = sd_card(in.local, he, in.radii, 0.0, 0.0, in.misc.w);
        let sa = in.fill.a * pow(clamp(1.0 - (ds + blur * 0.2) / blur, 0.0, 1.0), 1.7) * opacity;
        let vfs = clamp(globals.vis, 0.0, 1.0);
        return vec4(in.fill.rgb * sa * vfs, sa * vfs);
    }
    var card_p = in.local;
    if ((in.misc.w & BACKFACE) != 0u) {
        if ((in.misc.w & RIBBON_COLUMNS) != 0u) {
            card_p.x = 2.0 * in.shape.z - card_p.x;
        } else {
            card_p.y = 2.0 * in.shape.z - card_p.y;
        }
    }
    var d: f32;
    if ((in.misc.w & 1u) == 1u) {
        let shape = (in.misc.w >> 8u) & 15u;
        if (shape >= 1u && shape <= 4u) {
            d = sd_triangle(in.local, in.half_ext, shape - 1u);
        } else if (shape == 5u) {
            d = sd_diamond(in.local, in.half_ext);
        } else if (shape == 6u) {
            d = sd_diamond(in.local, in.half_ext);
        } else {
            d = sd_hexagon_flat(in.local, in.half_ext.x);
        }
    } else {
        d = sd_card(card_p, in.half_ext, in.radii, skew, in.shape.x, in.misc.w);
    }
    if ((in.misc.w & PROJECTED) != 0u) {
        d = max(d, ribbon_cut(in));
    }
    let grad = max(length(vec2(dpdx(d), dpdy(d))), 0.0001);
    let shape_a = select(
        clamp(0.5 - d / grad, 0.0, 1.0),
        step(d, 0.0),
        (in.misc.w & UNFRAMED_RECT) != 0u,
    );

    if ((in.misc.w & 32u) == 32u && in.misc.x > 0u) {
        let bend = clamp(in.flip.w, -2.7, 2.7);
        let par = in.flip.y;
        let pad = max(in.flip.z, 1.0);
        let hw = in.half_ext.x / pad;
        let hh = in.half_ext.y / pad;
        var loc = in.local;
        let yn = clamp(loc.y / max(hh, 1.0), -1.35, 1.35);
        loc.x -= bend * hw * 0.5 * yn * yn;
        loc.y *= 1.0 + abs(bend) * 0.12 * (1.0 - clamp(abs(loc.x / max(hw, 1.0)), 0.0, 1.0));
        let d2 = sd_card(loc, vec2(hw, hh), in.radii, skew, in.shape.x, in.misc.w);
        let grad2 = max(length(vec2(dpdx(d2), dpdy(d2))), 0.0001);
        let sa = clamp(0.5 - d2 / grad2, 0.0, 1.0);
        var norm = clamp(loc / (2.0 * vec2(hw, hh)) + vec2(0.5), vec2(0.0), vec2(1.0));
        norm.x = clamp(norm.x + par * 0.05, 0.0, 1.0);
        let ca = abs(bend) * 0.02;
        let cr = sample_card(in, vec2(clamp(norm.x + ca, 0.0, 1.0), norm.y));
        let cg = sample_card(in, norm);
        let cb = sample_card(in, vec2(clamp(norm.x - ca, 0.0, 1.0), norm.y));
        var col = vec3(cr.r, cg.g, cb.b);
        col = mix(col, in.tint.rgb, in.tint.a);
        let op = in.params.z;
        let vf2 = clamp(globals.vis, 0.0, 1.0);
        let fade2 = in.params.w;
        let base2 = mix(in.fill.rgb, col, fade2);
        let bw = in.params.y;
        let stroke_d2 = abs(d2 + bw * 0.5) - bw * 0.5;
        let sa2 = clamp(0.5 - stroke_d2 / grad2, 0.0, 1.0) * in.border.a * step(0.001, bw);
        var rgbo = base2 * sa;
        rgbo = rgbo * (1.0 - sa2) + in.border.rgb * sa2;
        let ao = max(sa, sa2);
        return vec4(rgbo * op * vf2, ao * op * vf2);
    }
    var base = in.fill;
    if (in.misc.x > 0u) {
        let norm = clamp(
            card_p / (2.0 * in.half_ext) + vec2(0.5),
            vec2(0.0),
            vec2(1.0),
        );
        let blur = in.flip.w;
        let ca = select(0.0, in.flip.y, in.flip.x < 0.001);
        var rgb: vec3<f32>;
        if ((in.misc.w & GHOST) != 0u) {
            rgb = ghost_blur(in, norm, 0.06 * in.flip.w);
        } else if ((in.misc.w & BACKDROP) != 0u && in.misc.x == 1u) {
            rgb = backdrop_grade(in, backdrop_blur(in, norm, 0.05 * in.flip.w));
        } else if (blur > 0.01 || ca > 0.01) {
            rgb = card_dof(in, norm, blur, ca);
        } else {
            let cropped = in.crop.xy + norm * in.crop.zw;
            let uv = in.uv.xy + cropped * in.uv.zw;
            var tex: vec4<f32>;
            if (in.misc.x == 1u) {
                tex = textureSample(near_tex, samp, uv, in.misc.y);
            } else if (in.misc.x == 3u) {
                tex = textureSample(preview_tex, samp, uv);
            } else {
                tex = textureSample(far_tex, samp, uv, in.misc.y);
            }
            rgb = tex.rgb;
        }
        if ((in.misc.w & GHOST) != 0u) {
            rgb = vec3(luma(rgb) * 1.7);
        }
        if ((in.misc.w & (BACKFACE | MUTED)) != 0u) {
            rgb = vec3(clamp((luma(rgb) - 0.5) * 1.15 + 0.5, 0.0, 1.0) * 0.5);
        }
        base = mix(in.fill, vec4(rgb, 1.0), fade);
    }
    if ((in.misc.w & 2u) != 0u) {
        let sweep = fract(globals.time / 1.2);
        let nx = clamp(in.local.x / (2.0 * max(in.half_ext.x, 1.0)) + 0.5, 0.0, 1.0);
        let band = 1.0 - abs(nx - sweep * 1.5 + 0.25) / 0.25;
        base = vec4(base.rgb + vec3(0.35) * clamp(band, 0.0, 1.0) * 0.35, base.a);
    }
    let flip_p = in.flip.x;
    if (flip_p > 0.001 && in.misc.x > 0u) {
        base = vec4(flip_color(in, clamp(flip_p, 0.0, 1.0)), base.a);
    }
    base = vec4(mix(base.rgb, in.tint.rgb, in.tint.a), base.a);

    let border_rgb = in.border.rgb;
    let fill_a = shape_a * base.a;
    let stroke_d = abs(d + border_w * 0.5) - border_w * 0.5;
    let stroke_a = clamp(0.5 - stroke_d / grad, 0.0, 1.0) * in.border.a * step(0.001, border_w);
    var rgb = base.rgb * fill_a;
    rgb = rgb * (1.0 - stroke_a) + border_rgb * stroke_a;
    var a = max(fill_a, stroke_a) * opacity;
    if (in.misc.z == 1u) {
        let inside = step(globals.clip.x, in.world.x)
            * step(globals.clip.y, in.world.y)
            * step(in.world.x, globals.clip.z)
            * step(in.world.y, globals.clip.w);
        a *= inside;
        rgb *= inside;
    }
    let vf = clamp(globals.vis, 0.0, 1.0);
    return vec4(rgb * opacity * vf, a * vf);
}
