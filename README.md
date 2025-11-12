

# 🌞 Proyecto: Simulación Solar con Shaders Dinámicos

Este proyecto implementa una animación cíclica del **Sol** mediante shaders personalizados, utilizando funciones de ruido para simular **turbulencias, erupciones y pulsaciones solares**. La animación es completamente procedural y controlada en tiempo real.

---

## 🎮 Controles

| Acción | Tecla |
|--------|-------|
| Mover cámara adelante / atrás | **W / S** |
| Mover cámara izquierda / derecha | **A / D** |
| Mover cámara arriba / abajo | **R / F** |
| Aumentar temperatura | **→ (flecha derecha)** |
| Disminuir temperatura | **← (flecha izquierda)** |
| Aumentar intensidad | **↑ (flecha arriba)** |
| Disminuir intensidad | **↓ (flecha abajo)** |

---

## 🔥 Descripción técnica

El shader combina **ruido fractal FBM** (*Fractal Brownian Motion*) con un **ruido de valor 3D** (*Value Noise*) para crear una superficie animada y orgánica.

### Funciones principales de ruido

- `value_noise3(p: Vector3)`: genera ruido pseudoaleatorio basado en coordenadas espaciales 3D. Utiliza funciones de hash y suavizado (*fade*) para interpolar los valores entre celdas.
- `fbm(p: Vector3, octaves: i32, lacunarity: f32, gain: f32)`: combina múltiples octavas de ruido `value_noise3` con distinta frecuencia y amplitud, generando un patrón más complejo y natural.

### Uniformes implementados

| Uniform | Tipo | Descripción |
|----------|------|-------------|
| `time` | `float` | Controla la animación del ruido y las pulsaciones solares. |
| `resolution` | `vec2` | Tamaño de la ventana en píxeles, usado para normalizar coordenadas. |
| `temp` | `float` | Controla la **temperatura** del color, variando del rojo anaranjado al blanco azulado. |
| `intensity` | `float` | Controla la **emisión de luz**, simulando la luminosidad o energía del Sol. |

### Vertex Shader – `SolarFlare`

Aplica un desplazamiento dinámico en los vértices hacia afuera de la superficie esférica mediante ruido FBM, generando una **corona solar** y efecto de flare.

### Fragment Shader

Usa la posición en espacio de objeto (`obj_pos`) para calcular el color y la emisión del Sol:
- Las zonas más cercanas al eje central son más brillantes (núcleo).  
- El **ruido FBM** modula el color local y la intensidad de la emisión.  
- El parámetro `temp` ajusta el gradiente de color desde rojo a blanco/azul.  
- `intensity` amplifica la luminosidad general, simulando picos de energía o erupciones.

---

## 🌈 Resultado visual

![Animación Solar](./demo.gif)

---

## 🧠 Créditos y referencias

- Basado en conceptos de *Procedural Texturing* y *Noise Functions* descritos por Ken Perlin.  
- Implementación inspirada en técnicas de *GPU procedural animation* y shading en espacio de objeto.  

---

**Autor:** Roberto — Universidad del Valle de Guatemala (UVG)  
**Fecha:** 2025