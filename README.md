# RUSTCASTER 

Ray caster hecho en **Rust** con [macroquad](https://github.com/not-fl3/macroquad), como proyecto 1 de gráficas por computadora.

Renderiza un nivel completo y jugable en primera persona usando el algoritmo clásico de **DDA** Uncluye colisiones, minimapa, sonido, animación de sprite, disparos, pantalla de bienvenida con selección de nivel, y pantalla de éxito.

## Cómo correrlo

Necesitas tener [Rust](https://www.rust-lang.org/tools/install) instalado.

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

## Controles

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

**Objetivo:** llegar a la celda de salida esto sin atravesar las paredes lo bonito es que en el camino puedes dispararle al orbe brillante que flota y pulsa, cada acierto suma un punto y el orbe reaparece en otro lugar del mapa.

## 🧩 Estructura del proyecto

```
rustcaster/
├── Cargo.toml
├── assets/          <- sonidos .wav generados 
│   ├── music.wav
│   ├── shoot.wav
│   ├── step.wav
│   ├── pop.wav
│   └── win.wav
└── src/
    └── main.rs      <- todo el juego
```

## 🛠️ Detalles técnicos

- **Raycasting:** algoritmo DDA por columnas con corrección de "fish-eye" mediante distancia perpendicular.
- **Colisión de paredes:** el jugador se mueve por ejes y se prueban las 4 esquinas de su "hitbox" circular contra el mapa antes de aplicar el movimiento, así nunca atraviesa paredes ni puede quedar fuera del mapa.
- **Paredes distintas:** 4 tipos de paredes y cada una con su color propio más un patrón que se le podria decir textura de ladrillos generado matemáticamente y sombreado por lado/distancia.
- **Sprite animado:** el orbe coleccionable usa proyección de billboard y tiene animación de rebote + pulso de tamaño.
- **Audio:** los 5 sonidos  son **sintetizados proceduralmente** con ondas seno/cuadradas, por lo que son 100% originales, por lo que no hay riesgo de usar música con derechos.
- **Minimapa:** dibujado en la esquina superior derecha, superpuesto sobre la vista 3D muestra paredes, salida, orbe y la posición/dirección del jugador.
- **Niveles:** 2 mapas distintos definidos como arte ASCII, la pantalla de bienvenida permite elegir entre ambos antes de empezar.

## Objetivos cubiertos 

- Raycaster jugable en Rust, sin atravesar paredes, sin crashear.
- Color/textura distinta por tipo de pared.
- Estética cuidada.
- Rotación horizontal con el mouse.
- Disparo.
- Minimapa en una esquina.
- Música de fondo.
- Efectos de sonido.
- Animación de sprite.
- Pantalla de bienvenida con selección entre múltiples niveles.
- Pantalla de éxito al completar el nivel.

