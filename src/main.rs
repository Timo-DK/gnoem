use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};

use gnoem::config::Config;
use gnoem::hooks;
use gnoem::office::Office;
use gnoem::persistence::GnoemRegistry;
use gnoem::renderer::Renderer;
use gnoem::tui::RatatuiRenderer;
use gnoem::watcher::EventWatcher;

#[derive(Parser)]
#[command(name = "gnoem", about = "Visualize Claude Code sessions as ASCII gnomes")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install hook scripts and print Claude settings snippet
    InstallHooks,
    /// Remove all Gnoem data (~/.config/gnoem/)
    Uninstall,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::InstallHooks) => {
            hooks::install_hooks()?;
            Ok(())
        }
        Some(Commands::Uninstall) => {
            hooks::uninstall()?;
            Ok(())
        }
        None => run_tui(),
    }
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(&Config::config_path())?;

    let registry_path = GnoemRegistry::registry_path();
    let registry = GnoemRegistry::load(registry_path)?;

    let mut office = Office::new(registry, config.clone());

    let mut watcher = EventWatcher::new(config.paths.events_dir.clone())?;
    watcher.start()?;

    let mut renderer = RatatuiRenderer::new()?;

    let ui_tick = Duration::from_millis(1000 / config.animation.ui_fps as u64);
    let sim_tick = Duration::from_millis(500);
    let mut last_sim = Instant::now();
    let mut paused = config.animation.paused;

    loop {
        // Process all pending events
        while let Some(event) = watcher.try_recv() {
            office.process_event(event);
        }

        // Simulation tick
        if !paused && last_sim.elapsed() >= sim_tick {
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            office.tick(now_secs);
            last_sim = Instant::now();
        }

        // Render
        renderer.render(&office)?;

        // Handle input
        if let Some(action) = renderer.handle_input() {
            match action {
                gnoem::renderer::AppAction::Quit => break,
                gnoem::renderer::AppAction::TogglePause => paused = !paused,
            }
        }

        // Sleep until next UI tick
        std::thread::sleep(ui_tick);
    }

    watcher.stop();

    Ok(())
}
