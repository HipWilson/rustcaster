// =============================================================================
// RUSTCASTER - Un Ray Caster simple estilo Wolfenstein 3D, hecho en Rust
// usando macroquad. Proyecto educativo (curso de gráficas por computadora).
// =============================================================================

use macroquad::audio::{load_sound, play_sound, play_sound_once, PlaySoundParams, Sound};
use macroquad::prelude::*;

// -----------------------------------------------------------------------------
// Constantes generales
// -----------------------------------------------------------------------------
const SCREEN_W: f32 = 1024.0;
const SCREEN_H: f32 = 640.0;
const NUM_RAYS: usize = 480; // resolución horizontal del render 3D (columnas)
const FOV_DEG: f32 = 66.0;
const MOVE_SPEED: f32 = 3.2; // celdas por segundo
const ROT_MOUSE_SENS: f32 = 2.6;
const ROT_KEY_SPEED: f32 = 2.4; // rad/s con flechas (respaldo sin mouse)
const PLAYER_RADIUS: f32 = 0.22;
const SHOOT_RANGE: f32 = 12.0;
const SHOOT_ANGLE_TOL: f32 = 0.09; // radianes de tolerancia para "apuntar"

// -----------------------------------------------------------------------------
// Pequeño generador de números pseudoaleatorios (para no depender de crates
// externos de rand y mantener el proyecto simple)
// -----------------------------------------------------------------------------
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E3779B97F4A7C15)
    }
    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn gen_range(&mut self, lo: f32, hi: f32) -> f32 {
        let v = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + (hi - lo) * v as f32
    }
    fn gen_index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        (self.next_u64() as usize) % len
    }
}

// -----------------------------------------------------------------------------
// Mapa / Nivel
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Level {
    name: &'static str,
    grid: Vec<Vec<u8>>, // 0 = piso, 1..=4 = tipos de pared distintos
    width: i32,
    height: i32,
    start: Vec2,
    exit: (i32, i32),
    floor_cells: Vec<(i32, i32)>, // celdas caminables (para reubicar el orbe)
}

fn parse_level(name: &'static str, rows: &[&str]) -> Level {
    let height = rows.len() as i32;
    let width = rows[0].len() as i32;
    let mut grid = vec![vec![0u8; width as usize]; height as usize];
    let mut start = vec2(1.5, 1.5);
    let mut exit = (width - 2, height - 2);
    let mut floor_cells = Vec::new();
    let mut orb_spawn = None;

    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let cell = match ch {
                '1' => 1u8,
                '2' => 2u8,
                '3' => 3u8,
                '4' => 4u8,
                'S' => {
                    start = vec2(x as f32 + 0.5, y as f32 + 0.5);
                    0
                }
                '9' => {
                    exit = (x as i32, y as i32);
                    0
                }
                'O' => {
                    orb_spawn = Some((x as i32, y as i32));
                    0
                }
                _ => 0u8,
            };
            grid[y][x] = cell;
            if cell == 0 {
                floor_cells.push((x as i32, y as i32));
            }
        }
    }

    let mut lvl = Level {
        name,
        grid,
        width,
        height,
        start,
        exit,
        floor_cells,
    };
    if let Some(p) = orb_spawn {
        lvl.floor_cells.push(p);
        lvl.floor_cells.dedup();
    }
    lvl
}

impl Level {
    #[inline]
    fn is_wall(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return true;
        }
        self.grid[y as usize][x as usize] != 0
    }

    #[inline]
    fn wall_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return 1;
        }
        self.grid[y as usize][x as usize]
    }

    fn orb_default_pos(&self) -> Vec2 {
        for (x, y) in &self.floor_cells {
            let p = vec2(*x as f32 + 0.5, *y as f32 + 0.5);
            if (p - self.start).length() > 3.0 {
                return p;
            }
        }
        vec2(self.start.x + 1.0, self.start.y)
    }
}

fn level_one() -> Level {
    let rows = [
        "1111111111111",
        "1S..1...2...1",
        "1.1.1.1.111.1",
        "1.1...1.....1",
        "1.111.1.111.1",
        "1.....1.1.1.1",
        "1.333.1.1.1.1",
        "1.....1...O.1",
        "1.11111.111.1",
        "1...........1",
        "1.111.1.1.1.1",
        "1.....1.1.9.1",
        "1111111111111",
    ];
    parse_level("Nivel 1 - Catacumbas", &rows)
}

fn level_two() -> Level {
    let rows = [
        "21111111411112111",
        "1S1...1.........3",
        "1.111.1.1.111.1.2",
        "1...1...1.......1",
        "1.1.1.111.3.11..1",
        "1.1.3.1...1.....1",
        "1.3.141.1.3.111.1",
        "1.1.....1...1...1",
        "1.111111.1114.1.1",
        "1.....O.......1.1",
        "3.11.31.1..1..1.1",
        "1.......1......91",
        "13111111131111111",
    ];
    parse_level("Nivel 2 - El Laberinto", &rows)
}

// -----------------------------------------------------------------------------
// Colores / "texturas" de pared
// -----------------------------------------------------------------------------
fn wall_base_color(wall_type: u8) -> Color {
    match wall_type {
        1 => Color::new(0.75, 0.24, 0.20, 1.0), // ladrillo rojo
        2 => Color::new(0.20, 0.42, 0.78, 1.0), // piedra azul
        3 => Color::new(0.25, 0.65, 0.30, 1.0), // musgo verde
        4 => Color::new(0.65, 0.55, 0.15, 1.0), // metal dorado
        _ => Color::new(0.5, 0.5, 0.5, 1.0),
    }
}

fn shade_wall(base: Color, wall_x: f32, dark_side: bool, dist: f32) -> Color {
    let brick_line = ((wall_x * 8.0) as i32) % 2 == 0;
    let mut r = base.r;
    let mut g = base.g;
    let mut b = base.b;
    if brick_line {
        r *= 0.86;
        g *= 0.86;
        b *= 0.86;
    }
    if dark_side {
        r *= 0.65;
        g *= 0.65;
        b *= 0.65;
    }
    let fog = (1.0 - (dist / 16.0)).clamp(0.25, 1.0);
    Color::new(r * fog, g * fog, b * fog, 1.0)
}

// -----------------------------------------------------------------------------
// Raycasting (algoritmo DDA clásico)
// -----------------------------------------------------------------------------
struct RayHit {
    dist: f32,
    wall_type: u8,
    dark_side: bool,
    wall_x: f32,
}

fn cast_ray(level: &Level, ox: f32, oy: f32, dir_x: f32, dir_y: f32) -> RayHit {
    let mut map_x = ox.floor() as i32;
    let mut map_y = oy.floor() as i32;

    let delta_dist_x = if dir_x.abs() < 1e-6 { 1e30 } else { (1.0 / dir_x).abs() };
    let delta_dist_y = if dir_y.abs() < 1e-6 { 1e30 } else { (1.0 / dir_y).abs() };

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1, (ox - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - ox) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1, (oy - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - oy) * delta_dist_y)
    };

    #[allow(unused_assignments)]
    let mut side_is_y = false;
    let mut safety = 0;
    loop {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side_is_y = false;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side_is_y = true;
        }
        safety += 1;
        if level.wall_at(map_x, map_y) != 0 || safety > 4000 {
            break;
        }
    }

    let wall_type = level.wall_at(map_x, map_y).max(1);

    let perp_dist = if !side_is_y {
        ((map_x as f32 - ox + (1 - step_x) as f32 / 2.0) / dir_x).abs()
    } else {
        ((map_y as f32 - oy + (1 - step_y) as f32 / 2.0) / dir_y).abs()
    };
    let perp_dist = perp_dist.max(0.0001);

    let wall_x = if !side_is_y { oy + perp_dist * dir_y } else { ox + perp_dist * dir_x };
    let wall_x = wall_x - wall_x.floor();

    RayHit { dist: perp_dist, wall_type, dark_side: side_is_y, wall_x }
}

// -----------------------------------------------------------------------------
// Movimiento con colisión (sin atravesar paredes)
// -----------------------------------------------------------------------------
fn collides(level: &Level, pos: Vec2, radius: f32) -> bool {
    let offsets = [(-radius, -radius), (radius, -radius), (-radius, radius), (radius, radius)];
    for (dx, dy) in offsets.iter() {
        let cx = (pos.x + dx).floor() as i32;
        let cy = (pos.y + dy).floor() as i32;
        if level.is_wall(cx, cy) {
            return true;
        }
    }
    false
}

fn try_move(level: &Level, pos: Vec2, delta: Vec2, radius: f32) -> Vec2 {
    let mut new_pos = pos;
    let step_x = vec2(pos.x + delta.x, pos.y);
    if !collides(level, step_x, radius) {
        new_pos.x = step_x.x;
    }
    let step_y = vec2(new_pos.x, pos.y + delta.y);
    if !collides(level, step_y, radius) {
        new_pos.y = step_y.y;
    }
    new_pos
}

// -----------------------------------------------------------------------------
// Partículas (efecto visual al "romper" el orbe)
// -----------------------------------------------------------------------------
struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    color: Color,
}

// -----------------------------------------------------------------------------
// Estados del juego
// -----------------------------------------------------------------------------
#[derive(PartialEq, Clone, Copy)]
enum GameState {
    Welcome,
    Playing,
    Success,
}

#[derive(PartialEq, Clone, Copy)]
enum LevelId {
    One,
    Two,
}

struct Sounds {
    music: Sound,
    shoot: Sound,
    step: Sound,
    pop: Sound,
    win: Sound,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "RUSTCASTER".to_owned(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let sounds = Sounds {
        music: load_sound("assets/music.wav").await.expect("music.wav"),
        shoot: load_sound("assets/shoot.wav").await.expect("shoot.wav"),
        step: load_sound("assets/step.wav").await.expect("step.wav"),
        pop: load_sound("assets/pop.wav").await.expect("pop.wav"),
        win: load_sound("assets/win.wav").await.expect("win.wav"),
    };

    let mut rng = Rng::new(0xC0FFEE ^ (macroquad::miniquad::date::now() as u64));

    let levels = [level_one(), level_two()];
    let mut selected_level = LevelId::One;

    let mut state = GameState::Welcome;
    let mut music_started = false;

    let mut level: Level = levels[0].clone();
    let mut player_pos: Vec2 = level.start;
    let mut player_angle: f32 = 0.0;
    let mut orb_pos: Vec2 = level.orb_default_pos();
    let mut orb_alive = true;
    let mut score: u32 = 0;
    let mut play_time: f32 = 0.0;

    let mut last_mouse: Vec2 = mouse_position().into();
    let mut step_timer: f32 = 0.0;
    let mut muzzle_flash: f32 = 0.0;
    let mut particles: Vec<Particle> = Vec::new();

    let mut depth_buf = vec![0.0f32; NUM_RAYS];

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        match state {
            // =========================================================
            // PANTALLA DE BIENVENIDA (con selección de nivel)
            // =========================================================
            GameState::Welcome => {
                show_mouse(true);
                set_cursor_grab(false);

                if is_key_pressed(KeyCode::Key1) {
                    selected_level = LevelId::One;
                }
                if is_key_pressed(KeyCode::Key2) {
                    selected_level = LevelId::Two;
                }
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                    selected_level = if selected_level == LevelId::One { LevelId::Two } else { LevelId::One };
                }

                draw_menu_background();

                let title = "RUSTCASTER";
                let td = measure_text(title, None, 64, 1.0);
                draw_text(title, SCREEN_W / 2.0 - td.width / 2.0, 120.0, 64.0, Color::new(0.95, 0.75, 0.2, 1.0));
                draw_centered("Un Ray Caster simple escrito en Rust", 170.0, 26.0, LIGHTGRAY);

                draw_centered("Selecciona tu nivel:", 260.0, 28.0, WHITE);
                let lvl1_txt = format!("{} 1) {}", if selected_level == LevelId::One { ">>" } else { "  " }, levels[0].name);
                let lvl2_txt = format!("{} 2) {}", if selected_level == LevelId::Two { ">>" } else { "  " }, levels[1].name);
                let c1 = if selected_level == LevelId::One { YELLOW } else { GRAY };
                let c2 = if selected_level == LevelId::Two { YELLOW } else { GRAY };
                draw_centered(&lvl1_txt, 300.0, 26.0, c1);
                draw_centered(&lvl2_txt, 335.0, 26.0, c2);

                draw_centered("[ENTER] Comenzar   [1 / 2 / <- ->] Elegir nivel", 410.0, 22.0, LIGHTGRAY);
                draw_centered("WASD mover | Mouse mirar | Click / ESPACIO disparar", 445.0, 20.0, GRAY);
                draw_centered("Objetivo: encuentra la salida (verde) del laberinto", 470.0, 20.0, GRAY);
                draw_centered("ESC para salir en cualquier momento", 495.0, 18.0, DARKGRAY);

                if !music_started {
                    play_sound(&sounds.music, PlaySoundParams { looped: true, volume: 0.35 });
                    music_started = true;
                }

                if is_key_pressed(KeyCode::Enter) {
                    level = if selected_level == LevelId::One { levels[0].clone() } else { levels[1].clone() };
                    player_pos = level.start;
                    player_angle = 0.0;
                    orb_pos = level.orb_default_pos();
                    orb_alive = true;
                    score = 0;
                    play_time = 0.0;
                    particles.clear();
                    state = GameState::Playing;
                    set_cursor_grab(true);
                    show_mouse(false);
                    last_mouse = mouse_position().into();
                }

                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
            }

            // =========================================================
            // JUGANDO
            // =========================================================
            GameState::Playing => {
                play_time += dt;

                let mouse_now: Vec2 = mouse_position().into();
                let delta_mouse = mouse_now - last_mouse;
                last_mouse = mouse_now;
                player_angle += (delta_mouse.x / SCREEN_W) * ROT_MOUSE_SENS * 6.0;

                if is_key_down(KeyCode::Right) {
                    player_angle += ROT_KEY_SPEED * dt;
                }
                if is_key_down(KeyCode::Left) {
                    player_angle -= ROT_KEY_SPEED * dt;
                }

                let dir_x = player_angle.cos();
                let dir_y = player_angle.sin();
                let right_x = -dir_y;
                let right_y = dir_x;

                let mut move_vec = vec2(0.0, 0.0);
                let mut moving = false;
                if is_key_down(KeyCode::W) {
                    move_vec += vec2(dir_x, dir_y);
                    moving = true;
                }
                if is_key_down(KeyCode::S) {
                    move_vec -= vec2(dir_x, dir_y);
                    moving = true;
                }
                if is_key_down(KeyCode::D) {
                    move_vec += vec2(right_x, right_y);
                    moving = true;
                }
                if is_key_down(KeyCode::A) {
                    move_vec -= vec2(right_x, right_y);
                    moving = true;
                }
                if move_vec.length_squared() > 0.0 {
                    move_vec = move_vec.normalize() * MOVE_SPEED * dt;
                }
                player_pos = try_move(&level, player_pos, move_vec, PLAYER_RADIUS);

                if moving {
                    step_timer -= dt;
                    if step_timer <= 0.0 {
                        play_sound_once(&sounds.step);
                        step_timer = 0.33;
                    }
                } else {
                    step_timer = 0.0;
                }

                if muzzle_flash > 0.0 {
                    muzzle_flash -= dt;
                }
                if (is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space)) && orb_alive {
                    play_sound_once(&sounds.shoot);
                    muzzle_flash = 0.12;
                    let to_orb = orb_pos - player_pos;
                    let dist_to_orb = to_orb.length();
                    let angle_to_orb = to_orb.y.atan2(to_orb.x);
                    let mut angle_diff = angle_to_orb - player_angle;
                    while angle_diff > std::f32::consts::PI {
                        angle_diff -= std::f32::consts::TAU;
                    }
                    while angle_diff < -std::f32::consts::PI {
                        angle_diff += std::f32::consts::TAU;
                    }
                    if angle_diff.abs() < SHOOT_ANGLE_TOL && dist_to_orb < SHOOT_RANGE {
                        let wall_hit = cast_ray(&level, player_pos.x, player_pos.y, dir_x, dir_y);
                        if wall_hit.dist > dist_to_orb {
                            play_sound_once(&sounds.pop);
                            score += 1;
                            orb_alive = false;
                            spawn_burst(&mut particles, &mut rng, SCREEN_W / 2.0, SCREEN_H / 2.0);
                        }
                    }
                }
                if !orb_alive && particles.is_empty() {
                    let idx = rng.gen_index(level.floor_cells.len());
                    let (cx, cy) = level.floor_cells[idx];
                    orb_pos = vec2(cx as f32 + 0.5, cy as f32 + 0.5);
                    orb_alive = true;
                }

                for p in particles.iter_mut() {
                    p.pos += p.vel * dt;
                    p.vel *= 0.94;
                    p.life -= dt;
                }
                particles.retain(|p| p.life > 0.0);

                let pcx = player_pos.x.floor() as i32;
                let pcy = player_pos.y.floor() as i32;
                if (pcx, pcy) == level.exit {
                    play_sound_once(&sounds.win);
                    state = GameState::Success;
                    show_mouse(true);
                    set_cursor_grab(false);
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Welcome;
                    show_mouse(true);
                    set_cursor_grab(false);
                }

                render_scene(&level, player_pos, dir_x, dir_y, right_x, right_y, orb_pos, orb_alive, &mut depth_buf);

                if muzzle_flash > 0.0 {
                    let a = (muzzle_flash / 0.12).clamp(0.0, 1.0);
                    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color::new(1.0, 0.9, 0.5, 0.15 * a));
                }

                for p in &particles {
                    let a = (p.life / p.max_life).clamp(0.0, 1.0);
                    draw_circle(p.pos.x, p.pos.y, 4.0 * a + 1.0, Color::new(p.color.r, p.color.g, p.color.b, a));
                }

                draw_line(SCREEN_W / 2.0 - 8.0, SCREEN_H / 2.0, SCREEN_W / 2.0 + 8.0, SCREEN_H / 2.0, 2.0, WHITE);
                draw_line(SCREEN_W / 2.0, SCREEN_H / 2.0 - 8.0, SCREEN_W / 2.0, SCREEN_H / 2.0 + 8.0, 2.0, WHITE);

                draw_minimap(&level, player_pos, player_angle, orb_pos, orb_alive);

                draw_text(&format!("Puntos: {}", score), 14.0, 26.0, 26.0, WHITE);
                draw_text(&format!("Nivel: {}", level.name), 14.0, 52.0, 20.0, LIGHTGRAY);
                draw_text(&format!("Tiempo: {:.1}s", play_time), 14.0, 76.0, 20.0, LIGHTGRAY);
            }

            // =========================================================
            // PANTALLA DE ÉXITO
            // =========================================================
            GameState::Success => {
                draw_menu_background();
                draw_centered("¡NIVEL COMPLETADO!", 220.0, 52.0, Color::new(0.3, 0.95, 0.4, 1.0));
                draw_centered(&format!("Nivel: {}", level.name), 290.0, 26.0, WHITE);
                draw_centered(&format!("Tiempo: {:.1} segundos", play_time), 325.0, 24.0, LIGHTGRAY);
                draw_centered(&format!("Orbes recolectados: {}", score), 355.0, 24.0, LIGHTGRAY);

                draw_centered("[ENTER] Volver al menú     [R] Reintentar nivel", 430.0, 24.0, YELLOW);
                draw_centered("ESC para salir", 465.0, 18.0, DARKGRAY);

                if is_key_pressed(KeyCode::Enter) {
                    state = GameState::Welcome;
                }
                if is_key_pressed(KeyCode::R) {
                    player_pos = level.start;
                    player_angle = 0.0;
                    orb_pos = level.orb_default_pos();
                    orb_alive = true;
                    score = 0;
                    play_time = 0.0;
                    particles.clear();
                    state = GameState::Playing;
                    set_cursor_grab(true);
                    show_mouse(false);
                    last_mouse = mouse_position().into();
                }
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
            }
        }

        next_frame().await;
    }
}

// -----------------------------------------------------------------------------
// Helpers de dibujo
// -----------------------------------------------------------------------------
fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let td = measure_text(text, None, size as u16, 1.0);
    draw_text(text, SCREEN_W / 2.0 - td.width / 2.0, y, size, color);
}

fn draw_menu_background() {
    let bands = 24;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let c = Color::new(0.05 + 0.05 * t, 0.02 + 0.03 * t, 0.10 + 0.08 * t, 1.0);
        draw_rectangle(0.0, SCREEN_H * t, SCREEN_W, SCREEN_H / bands as f32 + 1.0, c);
    }
}

fn spawn_burst(particles: &mut Vec<Particle>, rng: &mut Rng, x: f32, y: f32) {
    for _ in 0..18 {
        let ang = rng.gen_range(0.0, std::f32::consts::TAU);
        let speed = rng.gen_range(60.0, 220.0);
        particles.push(Particle {
            pos: vec2(x, y),
            vel: vec2(ang.cos() * speed, ang.sin() * speed),
            life: 0.5,
            max_life: 0.5,
            color: Color::new(1.0, 0.85, 0.2, 1.0),
        });
    }
}

fn draw_minimap(level: &Level, player_pos: Vec2, player_angle: f32, orb_pos: Vec2, orb_alive: bool) {
    let cell = 6.0;
    let map_w = level.width as f32 * cell;
    let map_h = level.height as f32 * cell;
    let margin = 14.0;
    let ox = SCREEN_W - map_w - margin;
    let oy = margin;

    draw_rectangle(ox - 4.0, oy - 4.0, map_w + 8.0, map_h + 8.0, Color::new(0.0, 0.0, 0.0, 0.55));

    for y in 0..level.height {
        for x in 0..level.width {
            let w = level.wall_at(x, y);
            if w != 0 {
                let c = wall_base_color(w);
                draw_rectangle(ox + x as f32 * cell, oy + y as f32 * cell, cell, cell, c);
            }
        }
    }
    draw_rectangle(
        ox + level.exit.0 as f32 * cell,
        oy + level.exit.1 as f32 * cell,
        cell,
        cell,
        Color::new(0.2, 0.9, 0.3, 1.0),
    );
    if orb_alive {
        draw_circle(ox + orb_pos.x * cell, oy + orb_pos.y * cell, cell * 0.4, YELLOW);
    }
    let px = ox + player_pos.x * cell;
    let py = oy + player_pos.y * cell;
    draw_circle(px, py, cell * 0.5, Color::new(1.0, 0.2, 0.2, 1.0));
    draw_line(
        px,
        py,
        px + player_angle.cos() * cell * 1.6,
        py + player_angle.sin() * cell * 1.6,
        2.0,
        WHITE,
    );

    draw_rectangle_lines(ox - 4.0, oy - 4.0, map_w + 8.0, map_h + 8.0, 2.0, WHITE);
}

#[allow(clippy::too_many_arguments)]
fn render_scene(
    level: &Level,
    player_pos: Vec2,
    dir_x: f32,
    dir_y: f32,
    right_x: f32,
    right_y: f32,
    orb_pos: Vec2,
    orb_alive: bool,
    depth_buf: &mut Vec<f32>,
) {
    // techo (degradado)
    let bands = 16;
    for i in 0..bands {
        let t0 = i as f32 / bands as f32;
        let t1 = (i + 1) as f32 / bands as f32;
        let y0 = (SCREEN_H / 2.0) * t0;
        let h = (SCREEN_H / 2.0) * (t1 - t0);
        let shade = 0.10 + 0.10 * t0;
        draw_rectangle(0.0, y0, SCREEN_W, h + 1.0, Color::new(shade, shade, shade + 0.08, 1.0));
    }
    // piso (degradado)
    for i in 0..bands {
        let t0 = i as f32 / bands as f32;
        let t1 = (i + 1) as f32 / bands as f32;
        let y0 = SCREEN_H / 2.0 + (SCREEN_H / 2.0) * t0;
        let h = (SCREEN_H / 2.0) * (t1 - t0);
        let shade = 0.28 - 0.14 * t0;
        draw_rectangle(0.0, y0, SCREEN_W, h + 1.0, Color::new(shade * 0.6, shade * 0.45, shade * 0.30, 1.0));
    }

    let fov_rad = FOV_DEG.to_radians();
    let plane_scale = (fov_rad / 2.0).tan();
    let plane_x = right_x * plane_scale;
    let plane_y = right_y * plane_scale;

    let col_w = SCREEN_W / NUM_RAYS as f32;

    for i in 0..NUM_RAYS {
        let camera_x = 2.0 * (i as f32) / (NUM_RAYS as f32) - 1.0;
        let ray_dir_x = dir_x + plane_x * camera_x;
        let ray_dir_y = dir_y + plane_y * camera_x;

        let hit = cast_ray(level, player_pos.x, player_pos.y, ray_dir_x, ray_dir_y);
        depth_buf[i] = hit.dist;

        let line_h = (SCREEN_H / hit.dist).min(4000.0);
        let draw_start = (SCREEN_H / 2.0 - line_h / 2.0).max(-2000.0);
        let color = shade_wall(wall_base_color(hit.wall_type), hit.wall_x, hit.dark_side, hit.dist);

        draw_rectangle(i as f32 * col_w, draw_start, col_w + 1.0, line_h, color);
    }

    // sprite del orbe (billboard con animación de rebote/pulso)
    if orb_alive {
        let t = get_time() as f32;
        let bob = (t * 4.0).sin() * 0.12;
        let pulse = 1.0 + 0.18 * (t * 6.0).sin();

        let sx = orb_pos.x - player_pos.x;
        let sy = orb_pos.y - player_pos.y;

        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        let transform_x = inv_det * (dir_y * sx - dir_x * sy);
        let transform_y = inv_det * (-plane_y * sx + plane_x * sy);

        if transform_y > 0.15 {
            let sprite_screen_x = (SCREEN_W / 2.0) * (1.0 + transform_x / transform_y);
            let sprite_size = ((SCREEN_H / transform_y).abs() * 0.35 * pulse).clamp(2.0, 2000.0);

            let col = (sprite_screen_x / col_w) as i32;
            let visible = if col >= 0 && (col as usize) < NUM_RAYS {
                depth_buf[col as usize] > transform_y
            } else {
                false
            };

            if visible {
                let cy = SCREEN_H / 2.0 - sprite_size * bob;
                let glow = Color::new(1.0, 0.85, 0.25, 0.9);
                draw_circle(sprite_screen_x, cy, sprite_size * 0.5, glow);
                draw_circle(sprite_screen_x, cy, sprite_size * 0.28, Color::new(1.0, 1.0, 0.8, 1.0));
            }
        }
    }
}
