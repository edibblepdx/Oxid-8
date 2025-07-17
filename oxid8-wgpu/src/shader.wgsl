// Vertex Shader

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Position: as is
    out.position = vec4<f32>(in.position, 1.0);

    // Texture: Convert clip space to [0, 1] and flip y
    out.tex_coords = (in.position.xy / 2.0) + 0.5;
    out.tex_coords.y = 1.0 - out.tex_coords.y;

    return out;
}

// Fragment Shader

// TODO: Add a light blur which should simulate a glow effect

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

fn box_blur(uv: vec2<f32>) -> vec3<f32> {
    var boxBlurColor: vec3<f32> = vec3<f32>(0.0);
    let kernelSize: i32 = 4;
    let texelSize: vec2<f32> = 0.05 / vec2<f32>(64.0, 32.0);
    let boxBlurDivisor: f32 = pow(f32(2 * kernelSize + 1), 2.0);
    for (var i: i32 = -kernelSize; i <= kernelSize; i++) {
        for (var j: i32 = -kernelSize; j <= kernelSize; j++) {
            let tx: vec3<f32> = textureSample(
                t_diffuse,
                s_diffuse,
                uv + vec2<f32>(f32(i), f32(j)) * texelSize
            ).rgb;
            boxBlurColor += tx;
        }
    }
    boxBlurColor /= boxBlurDivisor;
    return boxBlurColor;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv: vec2<f32> = in.tex_coords;
    let dist: vec2<f32> = abs(uv - 0.5);

    // Stretch uv (somewhat) radially
    uv -= 0.5;
    let r: vec2<f32> = vec2<f32>(0.0) - uv;
    uv = uv - length(dist) * 0.2 * r;
    uv += 0.5;

    // Sample where in [0,1] range
    var color: vec3<f32>;
    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        color = vec3(0.0);
    } else {
        color = box_blur(uv);
        color = color * 0.9; // Darken white pixels
        color = mix(color, vec3<f32>(0.1, 0.2, 0.8), 0.3);
    }

    return vec4<f32>(color, 1.0);
}
