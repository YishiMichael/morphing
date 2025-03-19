const PI = radians(180.0);

// struct Camera3DUniform {
//     view_motor: mat2x4<f32>,
//     projection_matrix: mat4x4<f32>,
// }

// struct Transform3DUniform {
//     motor: mat2x4<f32>,
//     scale: f32,
// }

struct CameraTransform2DUniform {
    view_motor: vec4<f32>,
    projection_matrix: mat3x3<f32>,
}

struct Transform2DUniform {
    motor: vec4<f32>,
    scale: f32,
}

struct ColorUniform {
    color: vec4<f32>,
}

struct GradientStorage {
    from_position: vec2<f32>,
    to_position: vec2<f32>,
    radius_slope: f32,
    radius_quotient: f32,
    radial_stops_range: vec2<u32>,
    angular_stops_range: vec2<u32>,
}

struct GradientStopStorage {
    alpha: f32,
    color: vec4<f32>,
}

struct Vertex {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
}

@group(0) @binding(0) var<uniform> u_transform_2d: Transform2DUniform;
@group(1) @binding(0) var<uniform> u_color: ColorUniform;
@group(1) @binding(1) var<storage> s_gradients: array<GradientStorage>;
@group(1) @binding(2) var<storage> s_radial_stops: array<GradientStopStorage>;
@group(1) @binding(3) var<storage> s_angular_stops: array<GradientStopStorage>;
@group(2) @binding(0) var<uniform> u_camera_transform_2d: CameraTransform2DUniform;

// @vertex
// fn vs_main(
//     in: Vertex,
// ) -> VertexOutput {
//     return VertexOutput(
//         vec4(apply_projection_matrix(
//             u_camera_transform_3d.projection_matrix, apply_motor(
//                 u_camera_transform_3d.motor, apply_motor(
//                     u_transform_3d.motor, in.position
//                 )
//             )
//         ), 1.0),
//         in.position,
//     );
// }

// fn apply_projection_matrix(
//     projection_matrix: mat4x4<f32>,
//     position: vec3<f32>,
// ) -> vec3<f32> {
//     let homogeneous_position = projection_matrix * vec4(position, 1.0);
//     return homogeneous_position.xyz / homogeneous_position.w;
// }

// // https://github.com/enkimute/LookMaNoMatrices/blob/main/src/miniPGA.glsl
// // Port from function `sw_mp`
// Motor should be normalized.
// fn apply_motor(
//     motor: mat2x4<f32>,
//     position: vec3<f32>,
// ) -> vec3<f32> {
//     let direction = cross(position, motor[0].yzw) - motor[1].yzw;
//     let half_shift = motor[0].x * direction + cross(direction, motor[0].yzw) - motor[0].yzw * motor[1].x;
//     return half_shift * 2.0 + position;
// }

@vertex
fn vs_main(
    in: Vertex,
) -> VertexOutput {
    return VertexOutput(
        vec4(apply_projection_matrix(
            u_camera_transform_2d.projection_matrix, apply_motor(
                u_camera_transform_2d.motor, apply_motor(
                    u_transform_2d.motor, in.position
                )
            )
        ), 0.0, 1.0),
        in.position,
    );
}

fn apply_projection_matrix(
    projection_matrix: mat3x3<f32>,
    position: vec2<f32>,
) -> vec3<f32> {
    let homogeneous_position = projection_matrix * vec3(position, 1.0);
    return homogeneous_position.xy / homogeneous_position.z;
}

// Motor should be normalized.
fn apply_motor(
    motor: vec4<f32>,
    position: vec2<f32>,
) -> vec2<f32> {
    let half_shift = (motor.w - motor.x * position.x) * vec2(motor.y, motor.x) + (motor.z - motor.y * position.y) * vec2(-motor.x, motor.y);
    return half_shift * 2.0 + position;
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    var color = u_color.color;
    for (var i = 0u; i < arrayLength(&s_gradients); i++) {
        color *= eval_gradient_color(s_gradients[i], in.position);
    }
    return color;
}

fn eval_gradient_color(
    gradient: GradientStorage,
    //radial_stops: ptr<storage, array<GradientStopStorage>>,
    //angular_stops: ptr<storage, array<GradientStopStorage>>,
    position: vec2<f32>,
) -> vec4<f32> {
    let from_position = gradient.from_position - position;
    let to_position = gradient.to_position - position;
    let p = gradient.radius_slope;
    let q = gradient.radius_quotient;

    let mid = q * to_position - from_position;
    let offset = to_position - from_position;
    let mid_dot_mid = dot(mid, mid);
    let mid_dot_offset = dot(mid, offset);
    let offset_dot_offset = dot(offset, offset);
    let from_cross_to = from_position[0] * to_position[1] - from_position[1] * to_position[0];
    let eta = dot(mid, from_position) / mid_dot_offset;
    let kappa = (from_cross_to * from_cross_to) / (mid_dot_mid * offset_dot_offset);
    let sigma = (1.0 - q) * (p - 2.0) / (1.0 - (p - 1.0) * (1.0 - q) * (1.0 - q) * kappa);
    let nu = sigma * kappa / (1.0 + sqrt(1.0 - sigma * sigma * kappa));
    let alpha = eta * (1.0 + (1.0 - q) * nu) + q * nu;
    let theta = atan2(mid_dot_offset, (1.0 - q) * from_cross_to);

    return interpolate_radial_color(gradient.radial_stops_range, fract(alpha))
        * interpolate_angular_color(gradient.angular_stops_range, (1.0 + theta / PI) / 2.0);
}

// TODO: refactor when https://github.com/gfx-rs/wgpu/pull/6913 is closed,
// with `requires pointer_composite_access;`
fn interpolate_radial_color(
    //stops: ptr<storage, array<GradientStopStorage>>,
    stops_range: vec2<u32>,
    alpha: f32,
) -> vec4<f32> {
    var start = stops_range[0];
    var end = stops_range[1];

    if (start == end) {
        return vec4(1.0);
    }
    if (alpha < s_radial_stops[start].alpha) {
        return s_radial_stops[start].color;
    }
    if (alpha >= s_radial_stops[end].alpha) {
        return s_radial_stops[end].color;
    }
    while (start + 1 < end) {
        let mid = start + (end - start) / 2;
        if (alpha < s_radial_stops[mid].alpha) {
            end = mid;
        } else {
            start = mid;
        }
    }
    return s_radial_stops[start].color
        + (s_radial_stops[end].color - s_radial_stops[start].color)
        * ((alpha - s_radial_stops[start].alpha) / (s_radial_stops[end].alpha - s_radial_stops[start].alpha));
}

fn interpolate_angular_color(
    //stops: ptr<storage, array<GradientStopStorage>>,
    stops_range: vec2<u32>,
    alpha: f32,
) -> vec4<f32> {
    var start = stops_range[0];
    var end = stops_range[1];

    if (start == end) {
        return vec4(1.0);
    }
    if (alpha < s_angular_stops[start].alpha) {
        return s_angular_stops[start].color;
    }
    if (alpha >= s_angular_stops[end].alpha) {
        return s_angular_stops[end].color;
    }
    while (start + 1 < end) {
        let mid = start + (end - start) / 2;
        if (alpha < s_angular_stops[mid].alpha) {
            end = mid;
        } else {
            start = mid;
        }
    }
    return s_angular_stops[start].color
        + (s_angular_stops[end].color - s_angular_stops[start].color)
        * ((alpha - s_angular_stops[start].alpha) / (s_angular_stops[end].alpha - s_angular_stops[start].alpha));
}
