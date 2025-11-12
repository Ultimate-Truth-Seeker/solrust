use raylib::prelude::*;

use crate::VertexShader;
pub struct Entity {
    pub name: &'static str,
    pub translation: Vector3,
    pub rotation: Vector3,
    pub scale: f32,
    pub vertices: Vec<Vector3>,
    pub vshader: VertexShader,
    pub face_tangent: bool,       // if true, add tangent-facing yaw from orbital motion
}