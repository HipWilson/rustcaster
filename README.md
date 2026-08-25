# RUSTCASTER 🧱🔦

Ray caster simple (estilo Wolfenstein 3D) hecho en **Rust** con [macroquad](https://github.com/not-fl3/macroquad), como proyecto del curso de gráficas por computadora.

Renderiza un nivel completo y jugable en primera persona usando el algoritmo clásico de **DDA (Digital Differential Analysis)**, con colisiones, minimapa, sonido, animación de sprite, disparos, pantalla de bienvenida con selección de nivel, y pantalla de éxito.

## ▶️ Cómo correrlo

Necesitas tener [Rust](https://www.rust-lang.org/tools/install) instalado (via `rustup`, o el paquete de tu distro con `rustc >= 1.80`).

```bash
git clone <tu-repo>
cd rustcaster
cargo run --release
```

> **Importante:** corre el comando desde la raíz del proyecto (donde está la carpeta `assets/`), porque el juego carga los sonidos con rutas relativas (`assets/*.wav`).

En Linux puede que necesites las librerías de desarrollo de X11/OpenGL/ALSA si no las tienes:
```bash
sudo apt-get install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

## 🎮 Controles

| Tecla / acción      | Función                          |
|---------------------|-----------------------------------|
| `W A S D`            | Moverse                          |
| Mouse (horizontal)   | Rotar la cámara                  |
| `Flechas ←/→`        | Rotar (alternativa sin mouse)    |
| Click izquierdo / `ESPACIO` | Disparar                  |
| `1` / `2`            | Elegir nivel en el menú          |
| `ENTER`               | Confirmar / continuar            |
| `R`                   | Reintentar nivel (pantalla de éxito) |
| `ESC`                 | Volver al menú / salir           |

**Objetivo:** llegar a la celda de salida (verde en el minimapa) sin atravesar las paredes. En el camino puedes dispararle al orbe brillante que flota y pulsa — cada acierto suma un punto y el orbe reaparece en otro lugar del mapa.

## 🧩 Estructura del proyecto

```
rustcaster/
├── Cargo.toml
├── assets/          <- sonidos .wav generados proceduralmente (música y efectos originales)
│   ├── music.wav
│   ├── shoot.wav
│   ├── step.wav
│   ├── pop.wav
│   └── win.wav
└── src/
    └── main.rs      <- todo el juego (raycasting, input, UI, audio, niveles)
```

## 🛠️ Detalles técnicos

- **Raycasting:** algoritmo DDA por columnas (uno de los más usados en tutoriales de raycasting clásico), con corrección de "fish-eye" mediante distancia perpendicular.
- **Colisión de paredes:** el jugador se mueve por ejes (X e Y por separado) y se prueban las 4 esquinas de su "hitbox" circular contra el mapa antes de aplicar el movimiento, así nunca atraviesa paredes ni puede quedar fuera del mapa (sin crashear).
- **Paredes distintas:** 4 tipos de pared (ladrillo rojo, piedra azul, musgo verde, metal dorado), cada una con su color propio más un patrón tipo "textura" de ladrillos generado matemáticamente (sin necesitar imágenes externas) y sombreado por lado/distancia (niebla).
- **Sprite animado:** el orbe coleccionable usa proyección de billboard (con oclusión contra las paredes usando un buffer de profundidad) y tiene animación de rebote + pulso de tamaño.
- **Audio:** los 5 sonidos (música de fondo, disparo, pasos, "pop" del orbe y fanfarria de victoria) son **sintetizados proceduralmente** con ondas seno/cuadradas (ver el pequeño script usado para generarlos), por lo que son 100% originales — no hay riesgo de usar música con derechos de autor.
- **Minimapa:** dibujado en la esquina superior derecha, superpuesto sobre la vista 3D (no al lado del mapa principal), muestra paredes, salida, orbe y la posición/dirección del jugador.
- **Niveles:** 2 mapas distintos definidos como arte ASCII, parseados a una grilla; la pantalla de bienvenida permite elegir entre ambos antes de empezar.

## ✅ Objetivos cubiertos (según la rúbrica)

- Raycaster jugable en Rust, sin atravesar paredes, sin crashear.
- Color/textura distinta por tipo de pared.
- Estética cuidada (paleta de colores, degradados de piso/techo, HUD, menús).
- Rotación horizontal con el mouse.
- Disparo.
- Minimapa en una esquina (no al lado del mapa).
- Música de fondo (original, no es de Taylor Swift).
- Efectos de sonido (disparo, pasos, recolección del orbe).
- Animación de sprite (el orbe flotante).
- Pantalla de bienvenida con selección entre múltiples niveles.
- Pantalla de éxito al completar el nivel.

¡Que lo disfrutes! 🎉
