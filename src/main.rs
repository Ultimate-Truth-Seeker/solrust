// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]
#[inline]
fn rotate_y(v: Vector3, ang: f32) -> Vector3 {
    let (s, c) = ang.sin_cos();
    Vector3::new(c*v.x + 0.0*v.y + -s*v.z, v.y, s*v.x + 0.0*v.y + c*v.z)
}

use raylib::prelude::*;
use std::f32::consts::PI;
use std::time::Instant;

mod framebuffer;
mod camera;
mod matrix;
mod triangle;
mod fragment;
mod light;
mod entity;

mod uniforms;
mod procedural;
use camera::Camera;
use entity::Entity;
use framebuffer::Framebuffer;
use light::Light;
use uniforms::Uniforms;
use fragment::Fragment;
use triangle::triangle;
use crate::{matrix::*, procedural::*, uniforms::*};

enum VertexShader {
    Identity,
    SolarFlare,
}

#[inline]
fn dot3(a: Vector3, b: Vector3) -> f32 { a.x*b.x + a.y*b.y + a.z*b.z }

#[inline]
fn fract(x: f32) -> f32 { x - x.floor() }

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + t * (b - a) }

#[inline]
fn fade(t: f32) -> f32 { t*t*t*(t*(t*6.0 - 15.0) + 10.0) }

#[inline]
fn hash3(p: Vector3) -> f32 {
    let n = dot3(p, Vector3::new(127.1, 311.7, 74.7));
    fract((n.sin() * 43758.5453).sin() * 143758.5453)
}

fn value_noise3(mut p: Vector3) -> f32 {
    let i = Vector3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = Vector3::new(p.x - i.x, p.y - i.y, p.z - i.z);

    let n000 = hash3(i + Vector3::new(0.0,0.0,0.0));
    let n100 = hash3(i + Vector3::new(1.0,0.0,0.0));
    let n010 = hash3(i + Vector3::new(0.0,1.0,0.0));
    let n110 = hash3(i + Vector3::new(1.0,1.0,0.0));
    let n001 = hash3(i + Vector3::new(0.0,0.0,1.0));
    let n101 = hash3(i + Vector3::new(1.0,0.0,1.0));
    let n011 = hash3(i + Vector3::new(0.0,1.0,1.0));
    let n111 = hash3(i + Vector3::new(1.0,1.0,1.0));

    let u = Vector3::new(fade(f.x), fade(f.y), fade(f.z));

    let nx00 = lerp(n000, n100, u.x);
    let nx10 = lerp(n010, n110, u.x);
    let nx01 = lerp(n001, n101, u.x);
    let nx11 = lerp(n011, n111, u.x);

    let nxy0 = lerp(nx00, nx10, u.y);
    let nxy1 = lerp(nx01, nx11, u.y);

    lerp(nxy0, nxy1, u.z)
}

fn fbm(mut p: Vector3, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    for _ in 0..octaves {
        sum += amp * value_noise3(Vector3::new(p.x*freq, p.y*freq, p.z*freq));
        freq *= lacunarity;
        amp *= gain;
    }
    sum
}

fn temperature_to_rgb(t: f32) -> Vector3 {
    // t in [0,1]: 0 = red/orange, 1 = white/blue
    // simple 3-point gradient: red -> yellow -> white
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        // red(1,0.2,0) to yellow(1,1,0)
        let k = t / 0.5;
        Vector3::new(1.0, lerp(0.2, 1.0, k), 0.0)
    } else {
        // yellow(1,1,0) to white(1,1,1) with slight blue tint
        let k = (t-0.5)/0.5;
        Vector3::new(1.0, 1.0, lerp(0.0, 0.3, k))
    }
}

fn apply_vertex_shader(v: Vector3, shader: &VertexShader, time: f32) -> Vector3 {
    match shader {
        VertexShader::Identity => v,
        VertexShader::SolarFlare => {
            // Displace along pseudo-normal (normalized position) with animated FBM
            let dir = if v.length() > 0.0 { v.normalized() } else { Vector3::new(0.0,0.0,1.0) };
            let p = Vector3::new(v.x*0.25, v.y*0.25, v.z*0.25 + time*0.2);
            let n = fbm(p, 4, 2.0, 0.5);
            let flare = (n*2.0 - 1.0) * 0.35; // amplitude in object units
            v + dir * flare
        }
    }
}

fn fragment_shader(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    // Use object-space direction for stable texturing on the sphere surface
    let mut dir = fragment.obj_position;
    let len = (dir.x*dir.x + dir.y*dir.y + dir.z*dir.z).sqrt();
    if len > 0.0 { dir = Vector3::new(dir.x/len, dir.y/len, dir.z/len); }

    // FBM turbulence driven by object-space, time-cycled
    let tloop = (u.time % 8.0) / 8.0;
    let p3 = Vector3::new(dir.x*3.0, dir.y*3.0, tloop*8.0);
    let turb = fbm(p3, 5, 2.0, 0.55);

    // Core intensity based on how close to the disc center it projects (approx with dir.z)
    // dir.z ~ facing viewer if camera looks down -Z; use abs to be camera-agnostic
    let facing = dir.z.abs();
    let base_core = facing.clamp(0.0, 1.0);

    // User controls: temp in [0,1], intensity scaler ~ [0,2]
    let intensity = ((base_core * 0.7 + turb * 0.6) * u.intensity).clamp(0.0, 1.0);

    // Temperature affects gradient selection
    let color_base = temperature_to_rgb(((intensity + u.temp*0.8)*0.7).clamp(0.0,1.0));

    // Emission spikes add energetic flicker
    let spikes = (value_noise3(Vector3::new(dir.x*10.0 + u.time*1.7, dir.y*10.0 - u.time*1.3, u.time*0.5))*2.0-1.0).abs();
    let emission = (0.6*intensity + 0.8*spikes).clamp(0.0, 1.5);

    Vector3::new(
        (color_base.x * emission).clamp(0.0, 1.0),
        (color_base.y * emission).clamp(0.0, 1.0),
        (color_base.z * emission).clamp(0.0, 1.0),
    )
}


fn transform(
    vertex: Vector3,
    translation: Vector3,
    scale: f32,
    rotation: Vector3,
    view: &Matrix,
    projection: &Matrix,
    viewport: &Matrix,
) -> Vector3 {
    let model : Matrix = create_model_matrix(translation, scale, rotation);
    let vertex4 = Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);

    let world_transform = multiply_matrix_vector4(&model, &vertex4);
    let view_transform = multiply_matrix_vector4(view, &world_transform);
    let projection_transform = multiply_matrix_vector4(projection, &view_transform);

    // División por w (NDC)
    let ndc = Vector4::new(
        projection_transform.x / projection_transform.w,
        projection_transform.y / projection_transform.w,
        projection_transform.z / projection_transform.w,
        1.0
    );

    // Viewport una sola vez (x,y), pero mantenemos depth en NDC [-1,1] para el Z-buffer
    let screen = multiply_matrix_vector4(viewport, &ndc);
    Vector3::new(screen.x, screen.y, ndc.z)
}

pub fn render(
    framebuffer: &mut Framebuffer,
    translation: Vector3,
    scale: f32,
    rotation: Vector3,
    vertex_array: &[Vector3],
    vshader: &VertexShader,
    view: &Matrix,
    projection: &Matrix,
    viewport: &Matrix,
    time: f32,
    resolution: Vector2,
    temp: f32,
    intensity: f32,
) {
    let light = Light::new(Vector3::new(0.0, 10.0, 0.0));
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    let mut obj_vertices_after_vs = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let v_obj = apply_vertex_shader(*vertex, vshader, time);
        obj_vertices_after_vs.push(v_obj);
        let transformed = transform(v_obj, translation, scale, rotation, view, projection, viewport);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    let mut obj_tris = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
            obj_tris.push([
                obj_vertices_after_vs[i].clone(),
                obj_vertices_after_vs[i + 1].clone(),
                obj_vertices_after_vs[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for (tri, obj_tri) in triangles.iter().zip(obj_tris.iter()) {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], &obj_tri[0], &obj_tri[1], &obj_tri[2], &light));
    }
    
    let uniforms = Uniforms {
        time,
        resolution,
        temp,
        intensity,
    };

    // Fragment Processing Stage
    for fragment in fragments {
        let final_rgb = fragment_shader(&fragment, &uniforms);
        let out = vec3_to_color(final_rgb);
        framebuffer.set_current_color(out);
        framebuffer.set_pixel(
            fragment.position.x as u32,
            fragment.position.y as u32,
            fragment.depth
        );
    }

}

fn main() {
    let window_width = 1300;
    let window_height = 600;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Wireframe")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let projection = create_projection_matrix(PI/3.0, window_width as f32 / window_height as f32, 0.5, 100.0);
    let viewport = create_viewport_matrix(0.0, 0.0, window_width as f32, window_height as f32);

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32, Color::BLACK);
    framebuffer.set_background_color(Color::new(4, 12, 36, 255));

    let mut temp_control: f32 = 0.5;      // 0 (rojo) … 1 (blanco/azulado)
    let mut intensity_control: f32 = 1.0; // 1 = normal, >1 más brillante

    // --- Scene entities ---
    let mut entities: Vec<Entity> = vec![
        // The ship we will follow
        Entity {
            name: "sun",
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: 1.0,
            vertices: generate_uv_sphere(3.0, 24, 32),
            vshader: VertexShader::SolarFlare,
            face_tangent: false,
        },
    ];


    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 17.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    let start_time = Instant::now();

    while !window.window_should_close() {
        framebuffer.clear();
        camera.process_input(&window);

        if window.is_key_down(KeyboardKey::KEY_RIGHT) { temp_control += 0.3 * window.get_frame_time(); }
        if window.is_key_down(KeyboardKey::KEY_LEFT)  { temp_control -= 0.3 * window.get_frame_time(); }
        if window.is_key_down(KeyboardKey::KEY_UP)    { intensity_control += 0.5 * window.get_frame_time(); }
        if window.is_key_down(KeyboardKey::KEY_DOWN)  { intensity_control -= 0.5 * window.get_frame_time(); }
        temp_control = temp_control.clamp(0.0, 1.0);
        intensity_control = intensity_control.clamp(0.2, 2.0);

        // Global time and resolution
        let time = start_time.elapsed().as_secs_f32();
        let resolution = Vector2::new(window_width as f32, window_height as f32);

        // --- Update entity motions ---
        use std::collections::HashMap;

        

        // --- Follow camera: lock target to sun position ---
        if let Some(sun) = entities.iter().find(|ent| ent.name == "sun") {
            camera.set_target(sun.translation);
        }

        let view = camera.get_view_matrix();

        // --- Render all entities ---
        for e in &entities {

            render(
                &mut framebuffer,
                e.translation,
                e.scale,
                e.rotation,
                &e.vertices,
                &e.vshader,
                &view,
                &projection,
                &viewport,
                time,
                resolution,
                temp_control,
                intensity_control,

            );
        }

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
