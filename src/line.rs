// src/line.rs
use raylib::prelude::*;
use crate::fragment::Fragment;

/// Rasteriza una línea (Bresenham) entre dos puntos en espacio de pantalla (x,y) con profundidad (z).
/// Devuelve los Fragment generados; el pintado queda a cargo del caller.
pub fn line(a: &Vector3, b: &Vector3, color: Vector3) -> Vec<Fragment> {
    let mut x0 = a.x.round() as i32;
    let mut y0 = a.y.round() as i32;
    let x1 = b.x.round() as i32;
    let y1 = b.y.round() as i32;

    // Profundidades para interpolación
    let z0 = a.z;
    let z1 = b.z;

    // Diferencias para Bresenham
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    // Cantidad total de pasos para interpolar profundidad
    let total_steps = dx.max(-dy) as f32;
    let mut step_idx: i32 = 0;

    let mut out = Vec::new();

    loop {
        // t en [0,1] para interpolar z
        let t = if total_steps > 0.0 {
            (step_idx as f32 / total_steps).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let depth = z0 + (z1 - z0) * t;

        out.push(Fragment::new(x0 as f32, y0 as f32, color, depth));

        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
        step_idx += 1;
    }

    out
}