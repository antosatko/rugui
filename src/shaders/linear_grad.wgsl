struct VertexInput {
    @builtin(vertex_index) index: u32
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) clip_position: vec2<f32>,
    
}

@group(0)@binding(0) var<uniform> screen_size: vec2<f32>;

@group(1)@binding(0) var<uniform> center: vec2<f32>;
@group(1)@binding(1) var<uniform> size: vec2<f32>;
@group(1)@binding(2) var<uniform> rotation: f32;
@group(1)@binding(3) var<uniform> alpha: f32;

@group(2)@binding(0) var<uniform> start_color: vec4<f32>;
@group(2)@binding(1) var<uniform> end_color: vec4<f32>;
@group(2)@binding(2) var<uniform> start: vec2<f32>;
@group(2)@binding(3) var<uniform> end: vec2<f32>;


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate vertex position
    var position = vertex_position(in.index);
    out.clip_position = position + 0.5;

    // Scale and rotate the position
    var scale = vec2(size.x * position.x, size.y * position.y);
    var cos_angle = cos(rotation);
    var sin_angle = sin(rotation);
    var rotated_position = vec2(
        scale.x * cos_angle - scale.y * sin_angle,
        scale.x * sin_angle + scale.y * cos_angle
    );
    
    // Translate to the new position
    var new_position = center + rotated_position;
    
    // Convert to screen space
    var screen_space = new_position / screen_size * 2.0 - 1.0;
    var invert_y = vec2(screen_space.x, -screen_space.y);

    out.position = vec4<f32>(invert_y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    var gradient_factor = dot(in.clip_position - start, end - start) / dot(end - start, end - start);
    var color = mix(start_color, end_color, clamp(gradient_factor, 0.0, 1.0));
    return vec4<f32>(color.rgb, color.a*alpha);
}



fn vertex_position(vertex_index: u32) -> vec2<f32> {
    // x: + + - - - +
    // y: + - - - + +
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u))-0.5;
}