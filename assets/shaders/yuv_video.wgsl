#import bevy_ui::ui_vertex_output::UiVertexOutput
#import bevy_render::color_operations::srgb_to_linear

struct YuvParams {
    mode: u32,
    matrix: u32,
    range: u32,
    _pad: u32,
}

@group(1) @binding(0)
var y_texture: texture_2d<f32>;
@group(1) @binding(1)
var y_sampler: sampler;
@group(1) @binding(2)
var u_texture: texture_2d<f32>;
@group(1) @binding(3)
var u_sampler: sampler;
@group(1) @binding(4)
var v_texture: texture_2d<f32>;
@group(1) @binding(5)
var v_sampler: sampler;
@group(1) @binding(6)
var<uniform> params: YuvParams;

fn sample_yuv420(uv: vec2<f32>) -> vec3<f32> {
    let y = textureSample(y_texture, y_sampler, uv).r;
    let u = textureSample(u_texture, u_sampler, uv).r;
    let v = textureSample(v_texture, v_sampler, uv).r;
    return vec3<f32>(y, u, v);
}

fn sample_nv12(uv: vec2<f32>) -> vec3<f32> {
    let y = textureSample(y_texture, y_sampler, uv).r;
    let uv2 = textureSample(u_texture, u_sampler, uv).rg;
    return vec3<f32>(y, uv2.r, uv2.g);
}

fn normalize_yuv(yuv: vec3<f32>) -> vec3<f32> {
    if params.range == 0u {
        let y = (yuv.x - 16.0 / 255.0) * (255.0 / 219.0);
        let u = (yuv.y - 128.0 / 255.0) * (255.0 / 224.0);
        let v = (yuv.z - 128.0 / 255.0) * (255.0 / 224.0);
        return vec3<f32>(y, u, v);
    }

    return vec3<f32>(yuv.x, yuv.y - 0.5, yuv.z - 0.5);
}

fn yuv_to_rgb_bt601(yuv: vec3<f32>) -> vec3<f32> {
    let y = yuv.x;
    let u = yuv.y;
    let v = yuv.z;
    return vec3<f32>(
        y + 1.4020 * v,
        y - 0.3441 * u - 0.7141 * v,
        y + 1.7720 * u,
    );
}

fn yuv_to_rgb_bt709(yuv: vec3<f32>) -> vec3<f32> {
    let y = yuv.x;
    let u = yuv.y;
    let v = yuv.z;
    return vec3<f32>(
        y + 1.5748 * v,
        y - 0.1873 * u - 0.4681 * v,
        y + 1.8556 * u,
    );
}

fn yuv_to_rgb_bt2020(yuv: vec3<f32>) -> vec3<f32> {
    let y = yuv.x;
    let u = yuv.y;
    let v = yuv.z;
    return vec3<f32>(
        y + 1.4746 * v,
        y - 0.1646 * u - 0.5714 * v,
        y + 1.8814 * u,
    );
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    var raw: vec3<f32>;
    if params.mode == 1u {
        raw = sample_nv12(in.uv);
    } else {
        raw = sample_yuv420(in.uv);
    }

    let yuv = normalize_yuv(raw);
    var rgb: vec3<f32>;
    if params.matrix == 0u {
        rgb = yuv_to_rgb_bt601(yuv);
    } else if params.matrix == 2u {
        rgb = yuv_to_rgb_bt2020(yuv);
    } else {
        rgb = yuv_to_rgb_bt709(yuv);
    }

    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(srgb_to_linear(rgb), 1.0);
}
