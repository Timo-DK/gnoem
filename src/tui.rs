/// Ratatui-based terminal renderer.
///
/// Implements the [`Renderer`] trait using crossterm as the backend.  The
/// renderer owns the terminal lifecycle (raw-mode, alternate screen) and tears
/// it down cleanly via [`Drop`].
use std::io::{self, Stdout};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::office::Office;
use crate::renderer::{AppAction, CubicleLayout, Renderer};
use crate::sprites::{cubicle_border, desk_art, sprite_frames};

// ---------------------------------------------------------------------------
// RatatuiRenderer
// ---------------------------------------------------------------------------

/// A terminal renderer backed by ratatui + crossterm.
pub struct RatatuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
}

impl RatatuiRenderer {
    /// Initialise the terminal: enable raw mode, enter the alternate screen,
    /// and create the ratatui [`Terminal`].
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            should_quit: false,
        })
    }
}

// ---------------------------------------------------------------------------
// Drop — restore the terminal on exit
// ---------------------------------------------------------------------------

impl Drop for RatatuiRenderer {
    fn drop(&mut self) {
        // Best-effort cleanup; ignore errors since we are already exiting.
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

// ---------------------------------------------------------------------------
// Renderer impl
// ---------------------------------------------------------------------------

impl Renderer for RatatuiRenderer {
    fn render(&mut self, office: &Office) -> Result<(), Box<dyn std::error::Error>> {
        let gnoems = office.gnoems();
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        self.terminal.draw(|frame| {
            let area = frame.area();

            if gnoems.is_empty() {
                // ── empty office ──────────────────────────────────────────
                let msg = Paragraph::new("The office is empty...")
                    .style(Style::default().fg(Color::DarkGray))
                    .centered();
                frame.render_widget(msg, area);
                return;
            }

            // ── cubicle grid ─────────────────────────────────────────────
            let layout = CubicleLayout::calculate(area.width, area.height);

            for (idx, gnoem) in gnoems.iter().enumerate() {
                let (cx, cy, cw, ch) = layout.cubicle_rect(idx);

                // Skip cubicles that fall entirely outside the visible area.
                if cx >= area.width || cy >= area.height {
                    continue;
                }

                let cubicle_area = Rect::new(
                    area.x + cx,
                    area.y + cy,
                    cw.min(area.width.saturating_sub(cx)),
                    ch.min(area.height.saturating_sub(cy)),
                );

                // ── header / border ──────────────────────────────────────
                let state_name = format!("{:?}", gnoem.state_machine.state);
                let duration_secs = gnoem.duration_secs(now_secs);
                let duration_str =
                    format!("{}m {}s", duration_secs / 60, duration_secs % 60);
                let border_lines = cubicle_border(
                    &gnoem.name,
                    &state_name,
                    &duration_str,
                    &gnoem.last_activity,
                    cw,
                );

                // Render the border lines at the top of the cubicle area.
                let header_height = border_lines.len() as u16;
                let header_area = Rect::new(
                    cubicle_area.x,
                    cubicle_area.y,
                    cubicle_area.width,
                    header_height.min(cubicle_area.height),
                );

                // Build styled lines: colour the name on the second line.
                let gnoem_color = ratatui_color(gnoem.color);
                let styled_lines: Vec<Line<'static>> = border_lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        if i == 1 {
                            // Name line — highlight the gnoem name.
                            colorise_name_line(line, &gnoem.name, gnoem_color)
                        } else {
                            Line::from(line.clone())
                        }
                    })
                    .collect();

                let header_widget = Paragraph::new(styled_lines);
                frame.render_widget(header_widget, header_area);

                // ── sprite ───────────────────────────────────────────────
                let frames = sprite_frames(&gnoem.state_machine.state);
                let frame_idx =
                    gnoem.state_machine.animation_frame % frames.len();
                let sprite_text = frames[frame_idx];

                let sprite_area_y = cubicle_area.y + header_height;
                if sprite_area_y < cubicle_area.y + cubicle_area.height {
                    let sprite_lines: Vec<Line> = sprite_text
                        .split('\n')
                        .map(|l| Line::from(Span::styled(l, Style::default().fg(gnoem_color))))
                        .collect();
                    let sprite_height = sprite_lines.len() as u16;
                    let sprite_area = Rect::new(
                        cubicle_area.x + 2,
                        sprite_area_y,
                        cubicle_area.width.saturating_sub(2),
                        sprite_height
                            .min(cubicle_area.height.saturating_sub(header_height)),
                    );
                    let sprite_widget = Paragraph::new(sprite_lines);
                    frame.render_widget(sprite_widget, sprite_area);

                    // ── desk art ─────────────────────────────────────────
                    let desk_area_y = sprite_area_y + sprite_height;
                    if desk_area_y < cubicle_area.y + cubicle_area.height {
                        let d = &gnoem.desk;
                        let art = desk_art(
                            d.coffee_level,
                            d.paper_stack,
                            d.plant_health,
                            d.monitor_error,
                            d.monitor_typing,
                            d.has_envelope,
                        );
                        let art_lines: Vec<Line> =
                            art.split('\n').map(Line::from).collect();
                        let art_height = art_lines.len() as u16;
                        let desk_area = Rect::new(
                            cubicle_area.x + 2,
                            desk_area_y,
                            cubicle_area.width.saturating_sub(2),
                            art_height.min(
                                cubicle_area
                                    .height
                                    .saturating_sub(header_height + sprite_height),
                            ),
                        );
                        let desk_widget = Paragraph::new(art_lines);
                        frame.render_widget(desk_widget, desk_area);

                        // ── mini-gnoems count ─────────────────────────────
                        let mini_count = gnoem.mini_gnoems.len();
                        if mini_count > 0 {
                            let mini_y = desk_area_y + art_height;
                            if mini_y < cubicle_area.y + cubicle_area.height {
                                let mini_text = format!(
                                    " ^{} mini-gnoem{}",
                                    mini_count,
                                    if mini_count == 1 { "" } else { "s" }
                                );
                                let mini_area = Rect::new(
                                    cubicle_area.x,
                                    mini_y,
                                    cubicle_area.width,
                                    1,
                                );
                                let mini_widget = Paragraph::new(mini_text)
                                    .style(Style::default().fg(Color::Yellow));
                                frame.render_widget(mini_widget, mini_area);
                            }
                        }
                    }
                }

                // ── outer border block (decorative) ──────────────────────
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray));
                frame.render_widget(block, cubicle_area);
            }
        })?;

        Ok(())
    }

    fn handle_input(&mut self) -> Option<AppAction> {
        // Non-blocking poll: return immediately if no event is waiting.
        if event::poll(Duration::ZERO).ok()? {
            if let CEvent::Key(key) = event::read().ok()? {
                // Ignore key-release events on platforms that emit them.
                if key.kind == KeyEventKind::Release {
                    return None;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                        return Some(AppAction::Quit);
                    }
                    KeyCode::Char('p') | KeyCode::Char(' ') => {
                        return Some(AppAction::TogglePause);
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Convert a ratatui [`Color`] stored on a `Gnoem` (which already uses
/// `ratatui::style::Color`) into one we can pass to [`Style`].
///
/// The `Color` in the office model is `ratatui::style::Color`, so this is
/// effectively a no-op copy, but the explicit function makes the intent clear.
fn ratatui_color(color: Color) -> Color {
    color
}

/// Build a styled [`Line`] for the cubicle name row, colouring the gnome's
/// name while leaving the surrounding box-drawing characters unstyled.
///
/// The returned `Line` uses `'static`-compatible owned strings so that the
/// borrow of `name` does not escape into the line's lifetime.
fn colorise_name_line(line: &str, name: &str, color: Color) -> Line<'static> {
    // The name row looks like: "║ Grumbold           ║"
    // We want to colour just the name part.
    if let Some(pos) = line.find(name) {
        let before = line[..pos].to_owned();
        let styled = name.to_owned();
        let after = line[pos + name.len()..].to_owned();
        Line::from(vec![
            Span::raw(before),
            Span::styled(styled, Style::default().fg(color)),
            Span::raw(after),
        ])
    } else {
        Line::from(line.to_owned())
    }
}
