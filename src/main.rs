mod game;
mod generator;

use game::{Cell, Game};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets, Skin};
use std::f32::consts::PI;

const LEVELS: [(&str, usize, usize, usize); 3] =
    [("Fácil", 9, 9, 10), ("Médio", 16, 16, 40), ("Difícil", 30, 16, 99)];
const DIGITS: [Color; 8] = [BLUE, DARKGREEN, RED, DARKBLUE, MAROON, DARKPURPLE, BLACK, GRAY];

#[derive(Clone, Copy)]
enum Face {
    Smile,
    Wow,
    Cool,
    Dizzy,
}

/// The "Personalizado" box: raw text fields plus the last validation error.
struct Form {
    w: String,
    h: String,
    m: String,
    err: String,
}

fn conf() -> Conf {
    Conf {
        window_title: "Campo Minado".into(),
        window_width: 960,
        window_height: 640,
        ..Default::default()
    }
}

/// Mines' counter format: "%03d", or "-%02d" below zero, clamped to -99..=999.
fn counter(n: isize) -> String {
    let n = n.clamp(-99, 999);
    if n < 0 { format!("-{:02}", -n) } else { format!("{n:03}") }
}

/// Same checks and messages as Mines' custom difficulty sheet.
fn validate(w: &str, h: &str, m: &str) -> Result<(usize, usize, usize), String> {
    let (Ok(w), Ok(h), Ok(m)) = (w.trim().parse::<i64>(), h.trim().parse::<i64>(), m.trim().parse::<i64>()) else {
        return Err("Insira números válidos".into());
    };
    if !(9..=50).contains(&w) {
        return Err("Largura deve ser 9-50".into());
    }
    if !(9..=50).contains(&h) {
        return Err("Altura deve ser 9-50".into());
    }
    if m < 1 {
        return Err("Pelo menos 1 mina".into());
    }
    let max = (w * h - 9).min(999);
    if m > max {
        return Err(format!("Muitas minas, máx {max}"));
    }
    Ok((w as usize, h as usize, m as usize))
}

fn draw_face(x: f32, y: f32, r: f32, face: Face) {
    let t = (r / 10.0).max(1.5);
    draw_circle(x, y, r, GOLD);
    draw_circle_lines(x, y, r, t, BLACK);
    let (ex, ey, er) = (r * 0.35, y - r * 0.25, r * 0.12);
    match face {
        Face::Cool => {
            for cx in [x - ex, x + ex] {
                draw_rectangle(cx - r * 0.25, ey - er * 1.5, r * 0.5, er * 3.0, BLACK);
            }
            draw_line(x - r * 0.8, ey - er, x + r * 0.8, ey - er, t, BLACK);
        }
        Face::Dizzy => {
            for cx in [x - ex, x + ex] {
                draw_line(cx - er, ey - er, cx + er, ey + er, t, BLACK);
                draw_line(cx - er, ey + er, cx + er, ey - er, t, BLACK);
            }
        }
        _ => {
            for cx in [x - ex, x + ex] {
                draw_circle(cx, ey, er, BLACK);
            }
        }
    }
    match face {
        Face::Wow => draw_circle_lines(x, y + r * 0.35, r * 0.18, t, BLACK),
        Face::Dizzy => draw_circle_lines(x, y + r * 0.4, r * 0.15, t, BLACK),
        _ => {
            let mr = r * 0.5;
            for k in 0..8 {
                let (a0, a1) = (PI * (0.2 + 0.075 * k as f32), PI * (0.2 + 0.075 * (k + 1) as f32));
                draw_line(x + mr * a0.cos(), y + mr * a0.sin(), x + mr * a1.cos(), y + mr * a1.sin(), t, BLACK);
            }
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);

    // ponytail: scale picked once from the start-up screen size; recompute per frame
    // (or use screen_dpi_scale) if rotating a phone makes the HUD look off.
    let u = (screen_width().min(screen_height()) / 600.0).max(1.0);
    let skin = {
        let ui = root_ui();
        let fs = (16.0 * u) as u16;
        Skin {
            label_style: ui.style_builder().font_size(fs).build(),
            button_style: ui
                .style_builder()
                .font_size(fs)
                .color(Color::from_rgba(204, 204, 204, 235))
                .color_hovered(Color::from_rgba(170, 170, 170, 235))
                .color_clicked(Color::from_rgba(187, 187, 187, 255))
                .color_selected(Color::from_rgba(120, 170, 230, 255))
                .color_selected_hovered(Color::from_rgba(100, 150, 210, 255))
                .build(),
            editbox_style: ui.style_builder().font_size(fs).color_selected(Color::from_rgba(200, 200, 200, 255)).build(),
            window_titlebar_style: ui
                .style_builder()
                .font_size(fs)
                .color(Color::from_rgba(68, 68, 68, 255))
                .text_color(WHITE)
                .build(),
            title_height: 28.0 * u,
            ..ui.default_skin()
        }
    };
    root_ui().push_skin(&skin);

    let (pad, btn_h, bar_h) = (8.0 * u, 32.0 * u, 44.0 * u);
    let hud = 3.0 * pad + btn_h + bar_h;

    let (w, h, m) = (LEVELS[0].1, LEVELS[0].2, LEVELS[0].3);
    let mut game = Game::new(w, h, m);
    let (mut t0, mut t1): (Option<f64>, Option<f64>) = (None, None);
    let mut flag_mode = false;
    let mut form: Option<Form> = None;

    loop {
        let mut next: Option<Game> = None;
        let (mx, my) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        // Row 1: difficulty buttons.
        let bar_w = (screen_width() - 2.0 * pad).min(640.0 * u);
        let bx = (screen_width() - bar_w) / 2.0;
        let bw = (bar_w - 3.0 * pad) / 4.0;
        let selected = LEVELS
            .iter()
            .position(|&(_, w, h, m)| (w, h, m) == (game.w, game.h, game.mines))
            .unwrap_or(LEVELS.len());
        for (k, label) in LEVELS.iter().map(|l| l.0).chain(["Personalizado"]).enumerate() {
            let pressed = widgets::Button::new(label)
                .position(vec2(bx + k as f32 * (bw + pad), pad))
                .size(vec2(bw, btn_h))
                .selected(k == selected)
                .ui(&mut root_ui());
            if pressed && form.is_none() {
                match LEVELS.get(k) {
                    Some(&(_, w, h, m)) => next = Some(Game::new(w, h, m)),
                    None => {
                        let (w, h, m) = (game.w.to_string(), game.h.to_string(), game.mines.to_string());
                        form = Some(Form { w, h, m, err: String::new() });
                    }
                }
            }
        }

        // Row 2: mines counter, smiley, flag mode, timer.
        let row_y = 2.0 * pad + btn_h;
        let (sx, sy, sr) = (screen_width() / 2.0, row_y + bar_h / 2.0, bar_h * 0.45);
        let hovered = vec2(mx, my).distance(vec2(sx, sy)) <= sr;
        if clicked && hovered && form.is_none() {
            next = Some(Game::new(game.w, game.h, game.mines));
        }
        let flag_btn = widgets::Button::new("Bandeira")
            .position(vec2(sx + sr + pad, row_y + (bar_h - btn_h) / 2.0))
            .size(vec2(120.0 * u, btn_h))
            .selected(flag_mode)
            .ui(&mut root_ui());
        if flag_btn {
            flag_mode = !flag_mode;
        }

        // Board.
        let size = (screen_width() / game.w as f32).min((screen_height() - hud) / game.h as f32).floor().max(1.0);
        let ox = ((screen_width() - size * game.w as f32) / 2.0).floor();
        let (cx, cy) = (((mx - ox) / size).floor(), ((my - hud) / size).floor());
        if form.is_none() && cx >= 0.0 && cy >= 0.0 && (cx as usize) < game.w && (cy as usize) < game.h {
            let i = cy as usize * game.w + cx as usize;
            if clicked {
                if game.state[i] == Cell::Open {
                    game.explore_all(i);
                } else if flag_mode {
                    game.toggle_flag(i);
                } else {
                    game.explore(i);
                }
            } else if is_mouse_button_pressed(MouseButton::Right) {
                game.toggle_flag(i);
            }
        }

        // Custom difficulty box.
        let mut close = false;
        if let Some(f) = form.as_mut() {
            let (ww, wh) = (360.0 * u, 230.0 * u);
            let (mut cancel, mut start) = (false, false);
            widgets::Window::new(hash!(), vec2((screen_width() - ww) / 2.0, (screen_height() - wh) / 2.0), vec2(ww, wh))
                .label("Dificuldade personalizada")
                .titlebar(true)
                .movable(false)
                .ui(&mut root_ui(), |ui| {
                    for (k, (label, data)) in [("Largura:", &mut f.w), ("Altura:", &mut f.h), ("Minas:", &mut f.m)]
                        .into_iter()
                        .enumerate()
                    {
                        widgets::InputText::new(hash!(k))
                            .label(label)
                            .ratio(0.6)
                            .position(vec2(pad, pad + k as f32 * (btn_h + pad / 2.0)))
                            .size(vec2(ww - 3.0 * pad, btn_h))
                            .ui(ui, data);
                    }
                    ui.label(vec2(pad, pad + 3.0 * (btn_h + pad / 2.0)), &f.err);
                    let by = wh - skin.title_height - btn_h - 2.0 * pad;
                    let half = (ww - 4.0 * pad) / 2.0;
                    cancel = widgets::Button::new("Cancelar")
                        .position(vec2(pad, by))
                        .size(vec2(half, btn_h))
                        .ui(ui);
                    start = widgets::Button::new("Iniciar")
                        .position(vec2(2.0 * pad + half, by))
                        .size(vec2(half, btn_h))
                        .ui(ui);
                });
            if start {
                match validate(&f.w, &f.h, &f.m) {
                    Ok((w, h, m)) => {
                        next = Some(Game::new(w, h, m));
                        close = true;
                    }
                    Err(e) => f.err = e,
                }
            }
            close |= cancel;
        }
        if close {
            form = None;
        }

        if let Some(g) = next {
            game = g;
            (t0, t1) = (None, None);
        }
        if game.started && t0.is_none() {
            t0 = Some(get_time());
        }
        if game.over() && t1.is_none() {
            t1 = Some(get_time());
        }
        let secs = t0.map_or(0.0, |t| t1.unwrap_or(get_time()) - t);

        // Draw.
        clear_background(Color::from_rgba(200, 200, 200, 255));
        let fs = 32.0 * u;
        let box_w = measure_text("-99", None, fs as u16, 1.0).width + 2.0 * pad;
        for (x, n) in [(bx, game.remaining_mines()), (bx + bar_w - box_w, secs as isize)] {
            draw_rectangle(x, row_y, box_w, bar_h, BLACK);
            let s = counter(n);
            let d = measure_text(&s, None, fs as u16, 1.0);
            draw_text(&s, x + (box_w - d.width) / 2.0, row_y + (bar_h + d.height) / 2.0, fs, RED);
        }
        let face = if game.won() {
            Face::Cool
        } else if game.lost {
            Face::Dizzy
        } else if hovered {
            Face::Wow
        } else {
            Face::Smile
        };
        draw_face(sx, sy, sr, face);

        for i in 0..game.state.len() {
            let (x, y) = (ox + (i % game.w) as f32 * size, hud + (i / game.w) as f32 * size);
            let open = game.state[i] == Cell::Open;
            let show_mine = game.lost && game.mine[i] && game.state[i] != Cell::Flag;
            let bg = if game.exploded == Some(i) {
                RED
            } else if open || show_mine {
                Color::from_rgba(235, 235, 235, 255)
            } else {
                Color::from_rgba(160, 160, 160, 255)
            };
            draw_rectangle(x, y, size, size, bg);
            draw_rectangle_lines(x, y, size, size, 1.0, Color::from_rgba(120, 120, 120, 255));

            let label = |s: &str, c: Color| {
                let d = measure_text(s, None, size as u16, 1.0);
                draw_text(s, x + (size - d.width) / 2.0, y + (size + d.height) / 2.0, size, c);
            };
            if show_mine {
                draw_circle(x + size / 2.0, y + size / 2.0, size * 0.28, BLACK);
            } else if open && game.adj[i] > 0 {
                label(&game.adj[i].to_string(), DIGITS[game.adj[i] as usize - 1]);
            } else if game.state[i] == Cell::Flag {
                let (fx, fy) = (x + size * 0.35, y + size * 0.2);
                draw_line(fx, fy, fx, y + size * 0.8, 2.0, BLACK);
                draw_triangle(vec2(fx, fy), vec2(fx, fy + size * 0.3), vec2(fx + size * 0.4, fy + size * 0.15), RED);
            } else if game.state[i] == Cell::Question {
                label("?", BLACK);
            }
        }

        next_frame().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_matches_mines_format() {
        assert_eq!(counter(5), "005");
        assert_eq!(counter(-3), "-03");
        assert_eq!(counter(-150), "-99");
        assert_eq!(counter(1500), "999");
    }

    #[test]
    fn custom_box_rules() {
        assert_eq!(validate("abc", "9", "10"), Err("Insira números válidos".into()));
        assert_eq!(validate("8", "9", "10"), Err("Largura deve ser 9-50".into()));
        assert_eq!(validate("9", "51", "10"), Err("Altura deve ser 9-50".into()));
        assert_eq!(validate("9", "9", "0"), Err("Pelo menos 1 mina".into()));
        assert_eq!(validate("9", "9", "72"), Ok((9, 9, 72)));
        assert_eq!(validate("9", "9", "73"), Err("Muitas minas, máx 72".into()));
        assert_eq!(validate("50", "50", "1000"), Err("Muitas minas, máx 999".into()));
        assert_eq!(validate(" 30 ", "16", "99"), Ok((30, 16, 99)));
    }
}
