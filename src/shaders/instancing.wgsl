@group(0)@binding(0) var<uniform> screen_size: vec2<f32>;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(5) position: vec2<f32>,
    @location(6) size: vec2<f32>,
    @location(7) rotation: f32,
    @location(8) color: vec4<f32>,
    @location(9) alpha: f32,
    @location(10) edges: vec2<f32>,
    @location(11) _remove_test_size_: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) clip_position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) edges: vec2<f32>,
    @location(3) size: vec2<f32>,
    @location(4) alpha: f32,
}


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = in.color;
    out.edges = in.edges;
    out.size = in.size;
    out.alpha = in.alpha;

    // Calculate vertex position
    var position = vertex_position(in.index);
    out.clip_position = (in.size * position*2.0);

    // Scale and rotate the position
    var scale = in.size * position;
    var cos_angle = cos(in.rotation);
    var sin_angle = sin(in.rotation);
    var rotated_position = vec2(
        scale.x * cos_angle - scale.y * sin_angle,
        scale.x * sin_angle + scale.y * cos_angle
    );
    
    // Translate to the new position
    var new_position = in.position + rotated_position;
    
    // Convert to screen space
    var screen_space = new_position / screen_size * 2.0 - 1.0;
    var invert_y = vec2(screen_space.x, -screen_space.y);

    out.position = vec4<f32>(invert_y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    var p = abs(in.clip_position);
    var s = in.size - in.edges.x;
    if p.x < s.x || p.y < s.y {
        return vec4<f32>(in.color.rgb, in.color.a*in.alpha);
    }
    var dist = distance(p, s);
    if dist < in.edges.x {
        return vec4<f32>(in.color.rgb, in.color.a*in.alpha);
    }
    var glow = 1.0 - ((dist - in.edges.x) / in.edges.y);
    return vec4<f32>(in.color.rgb, in.color.a*in.alpha*glow);
    //return in.color;
}

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     var p = abs(in.clip_position);
//     var s = in.size - in.edges.x;
//     
//     // Calculate the condition value as a float where 1.0 means true and 0.0 means false
//     var cond1 = step(s.x, p.x) * step(s.y, p.y); // This is the negation of "p.x < s.x || p.y < s.y"
//     
//     var dist = distance(p, s);
//     var cond2 = step(in.edges.x, dist); // This is the negation of "dist < in.edges.x"
//     
//     // Alpha calculation based on conditions
//     var alpha1 = in.color.a * in.alpha * (1.0 - cond1); // Case for "if p.x < s.x || p.y < s.y"
//     var alpha2 = in.color.a * in.alpha * (1.0 - cond2); // Case for "if dist < in.edges.x"
//     
//     // Glow calculation
//     var glow = max(0.0, 1.0 - ((dist - in.edges.x) / in.edges.y));
//     var alpha3 = in.color.a * in.alpha * glow * cond2; // Case for glow when dist >= in.edges.x
//     
//     // Combine the results, leveraging the conditions to blend the results correctly
//     var final_alpha = alpha1 + alpha2 * cond1 + alpha3 * cond1 * cond2;
//     
//     return vec4<f32>(in.color.rgb, final_alpha);
// }

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    // x: + + - - - +
    // y: + - - - + +
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u))-0.5;
}