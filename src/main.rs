#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(
    let_chains,
    result_flattening,
    thread_local,
    trait_alias,
    try_blocks,
    variant_count
)]

mod backend;
mod frontend;

use crate::frontend::{App, AppInit, RunBackendResult, GLOBAL_CSS};
use bytesize::ByteSize;
use clap::Parser;
use duct::{cmd, Expression};
use file_rotate::compression::Compression;
use file_rotate::suffix::AppendCount;
use file_rotate::{ContentLimit, FileRotate};
use futures::channel::mpsc;
use gtk::glib;
use relm4::prelude::*;
use relm4::RELM_THREADS;
use std::borrow::Cow;
use std::cell::Cell;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Termination};
use std::rc::Rc;
use std::thread::available_parallelism;
use std::time::{Duration, Instant};
use std::{env, fs, io, process};
use subspace_farmer::utils::run_future_in_dedicated_thread;
use subspace_proof_of_space::chia::ChiaTable;
use tracing::{debug, error, info, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Number of log files to keep
const LOG_FILE_LIMIT_COUNT: usize = 5;
/// Size of one log file
const LOG_FILE_LIMIT_SIZE: usize = ByteSize::mib(10).as_u64() as usize;
const LOG_READ_BUFFER: usize = ByteSize::mib(1).as_u64() as usize;
/// If `true`, this means supervisor will not be able to capture logs from child application and logger needs to be in
/// the child process itself, while supervisor will not attempt to read stdout/stderr at all
const WINDOWS_SUBSYSTEM_WINDOWS: bool = cfg!(all(windows, not(debug_assertions)));
const MIN_RUNTIME_DURATION_FOR_AUTORESTART: Duration = Duration::from_secs(30);

type PosTable = ChiaTable;

fn raise_fd_limit() {
    match fdlimit::raise_fd_limit() {
        Ok(fdlimit::Outcome::LimitRaised { from, to }) => {
            debug!(
                "Increased file descriptor limit from previous (most likely soft) limit {} to \
                new (most likely hard) limit {}",
                from, to
            );
        }
        Ok(fdlimit::Outcome::Unsupported) => {
            // Unsupported platform (non-Linux)
        }
        Err(error) => {
            warn!(
                "Failed to increase file descriptor limit for the process due to an error: {}.",
                error
            );
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum AppStatusCode {
    Exit,
    Restart,
    Unknown(i32),
}

impl AppStatusCode {
    fn from_status_code(status_code: i32) -> Self {
        match status_code {
            0 => Self::Exit,
            100 => Self::Restart,
            code => Self::Unknown(code),
        }
    }

    fn into_status_code(self) -> i32 {
        match self {
            AppStatusCode::Exit => 0,
            AppStatusCode::Restart => 100,
            AppStatusCode::Unknown(code) => code,
        }
    }
}

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Cli {
    /// Used for startup to minimize the window
    #[arg(long)]
    startup: bool,
    /// Used to indicate that application was restarted after crash
    #[arg(long)]
    after_crash: bool,
    /// Used by child process such that supervisor parent process can control it
    #[arg(long)]
    child_process: bool,
    /// Show uninstall dialog to delete configuration and logs, typically called from installer
    /// during package uninstallation
    #[arg(long)]
    #[doc(hidden)]
    uninstall: bool,
    /// The rest of the arguments that will be sent to GTK4 as is
    #[arg(raw = true)]
    gtk_arguments: Vec<String>,
}

impl Cli {
    fn run(self) -> ExitCode {
        if self.uninstall {
            #[cfg(windows)]
            {
                let dirs_to_remove = env::var_os("SystemDrive")
                    .and_then(|system_drive| {
                        let system_drive = system_drive.into_string().ok()?;
                        Some(
                            fs::read_dir(format!("{system_drive}\\Users"))
                                .ok()?
                                .flatten()
                                .map(|user_dir| {
                                    user_dir
                                        .path()
                                        .join("AppData")
                                        .join("Local")
                                        .join(env!("CARGO_PKG_NAME"))
                                })
                                .filter(|path| path.exists())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .unwrap_or_default();
                if dirs_to_remove.is_empty() {
                    return ExitCode::SUCCESS;
                }

                if native_dialog::MessageDialog::new()
                    .set_type(native_dialog::MessageType::Info)
                    .set_title("Space Acres Uninstallation")
                    .set_text("Delete Space Acres configuration and logs for all users?")
                    .show_confirm()
                    .unwrap_or_default()
                {
                    for dir in dirs_to_remove {
                        let _ = fs::remove_dir_all(dir);
                    }
                }
            }

            ExitCode::SUCCESS
        } else if self.child_process {
            ExitCode::from(self.app().into_status_code() as u8)
        } else {
            self.supervisor().report()
        }
    }

    fn app(self) -> AppStatusCode {
        let maybe_app_data_dir = Self::app_data_dir();

        {
            let layer = tracing_subscriber::fmt::layer()
                // TODO: Workaround for https://github.com/tokio-rs/tracing/issues/2214, also on
                //  Windows terminal doesn't support the same colors as bash does
                .with_ansi(if cfg!(windows) {
                    false
                } else {
                    supports_color::on(supports_color::Stream::Stderr).is_some()
                });
            let filter = EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy();
            if WINDOWS_SUBSYSTEM_WINDOWS {
                if let Some(app_data_dir) = &maybe_app_data_dir {
                    let logger = std::sync::Mutex::new(Self::new_logger(app_data_dir));
                    let layer = layer.with_writer(logger);

                    tracing_subscriber::registry()
                        .with(layer.with_filter(filter))
                        .init();
                } else {
                    tracing_subscriber::registry()
                        .with(layer.with_filter(filter))
                        .init();
                }
                #[cfg(windows)]
                std::panic::set_hook(Box::new(tracing_panic::panic_hook));
            } else {
                tracing_subscriber::registry()
                    .with(layer.with_filter(filter))
                    .init();
            }
        }

        info!(
            "Starting {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        glib::log_set_writer_func(|log_level, log_fields| {
            let log_fields = log_fields
                .iter()
                .map(|log_field| {
                    let key = log_field.key();
                    if let Some(string) = log_field.value_str() {
                        (key, Cow::Borrowed(string))
                    } else if let Some(bytes) = log_field.value_bytes() {
                        (key, Cow::Owned(hex::encode(bytes)))
                    } else if let Some(user_data) = log_field.user_data() {
                        (key, Cow::Owned(user_data.to_string()))
                    } else {
                        (key, Cow::Borrowed(""))
                    }
                })
                .collect::<Vec<_>>();

            match log_level {
                glib::LogLevel::Error => {
                    tracing::event!(tracing::Level::ERROR, ?log_fields, "Glib log");
                }
                glib::LogLevel::Critical => {
                    tracing::event!(tracing::Level::ERROR, ?log_fields, "Glib log");
                }
                glib::LogLevel::Warning => {
                    tracing::event!(tracing::Level::WARN, ?log_fields, "Glib log");
                }
                glib::LogLevel::Message => {
                    tracing::event!(tracing::Level::INFO, ?log_fields, "Glib log");
                }
                glib::LogLevel::Info => {
                    tracing::event!(tracing::Level::INFO, ?log_fields, "Glib log");
                }
                glib::LogLevel::Debug => {
                    tracing::event!(tracing::Level::DEBUG, ?log_fields, "Glib log");
                }
            }

            glib::LogWriterOutput::Handled
        });

        // The default in `relm4` is `1`, set this back to Tokio's default
        RELM_THREADS
            .set(
                available_parallelism()
                    .map(|cores| cores.get())
                    .unwrap_or(1),
            )
            .expect("The first thing in the app, is not set; qed");

        let app = RelmApp::new("xyz.autonomys.space_acres");
        let app = app.with_args({
            let mut args = self.gtk_arguments;
            // Application itself is expected as the first argument
            args.insert(0, env::args().next().expect("Guaranteed to exist; qed"));
            args
        });

        app.set_global_css(GLOBAL_CSS);
        relm4_icons::initialize_icons();

        // Prefer dark theme in cross-platform way if environment is configured that way
        if let Some(settings) = gtk::Settings::default() {
            if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                settings.set_gtk_application_prefer_dark_theme(true);
            }
        }

        let exit_status_code = Rc::new(Cell::new(AppStatusCode::Exit));

        app.run_async::<App>(AppInit {
            app_data_dir: maybe_app_data_dir,
            exit_status_code: Rc::clone(&exit_status_code),
            minimize_on_start: self.startup,
            crash_notification: self.after_crash,
            run_backend: || {
                let (backend_action_sender, backend_action_receiver) = mpsc::channel(1);
                let (backend_notification_sender, backend_notification_receiver) =
                    mpsc::channel(100);

                // Create and run backend in dedicated thread
                let backend_fut = run_future_in_dedicated_thread(
                    move || backend::create(backend_action_receiver, backend_notification_sender),
                    "backend".to_string(),
                )
                .expect("Must be able to spawn a thread");

                RunBackendResult {
                    backend_fut: Box::new(async move {
                        match backend_fut.await {
                            Ok(()) => {
                                info!("Backend exited");
                            }
                            Err(_) => {
                                error!("Backend spawning failed");
                            }
                        }
                    }),
                    backend_action_sender,
                    backend_notification_receiver,
                }
            },
        });

        let exit_status_code = exit_status_code.get();
        info!(
            ?exit_status_code,
            "Exiting {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        exit_status_code
    }

    fn supervisor(mut self) -> io::Result<()> {
        let maybe_app_data_dir = Self::app_data_dir();

        let mut last_start;
        let program = Self::child_program()?;

        loop {
            let mut args = vec!["--child-process".to_string()];
            if self.after_crash {
                self.after_crash = false;

                args.push("--after-crash".to_string());
            } else if self.startup {
                // In case of restart we no longer want to minimize the app
                self.startup = false;

                args.push("--startup".to_string());
            }
            args.push("--".to_string());
            args.extend_from_slice(&self.gtk_arguments);

            last_start = Instant::now();
            let exit_status = if let Some(app_data_dir) = (!WINDOWS_SUBSYSTEM_WINDOWS)
                .then_some(maybe_app_data_dir.as_ref())
                .flatten()
            {
                let mut expression = Self::maybe_force_renderer(cmd(&program, args))
                    .stderr_to_stdout()
                    // We use non-zero status codes, and they don't mean error necessarily
                    .unchecked()
                    .reader()?;

                let mut logger = Self::new_logger(app_data_dir);

                let mut log_read_buffer = vec![0u8; LOG_READ_BUFFER];

                let mut stdout = io::stdout();
                loop {
                    match expression.read(&mut log_read_buffer) {
                        Ok(bytes_count) => {
                            if bytes_count == 0 {
                                break;
                            }

                            let write_result: io::Result<()> = try {
                                stdout.write_all(&log_read_buffer[..bytes_count])?;
                                logger.write_all(&log_read_buffer[..bytes_count])?;
                            };

                            if let Err(error) = write_result {
                                eprintln!("Error while writing output of child process: {error}");
                                break;
                            }
                        }
                        Err(error) => {
                            if error.kind() == io::ErrorKind::Interrupted {
                                // Try again
                                continue;
                            }
                            eprintln!("Error while reading output of child process: {error}");
                            break;
                        }
                    }
                }

                stdout.flush()?;
                if let Err(error) = logger.flush() {
                    eprintln!("Error while flushing logs: {error}");
                }

                match expression.try_wait()? {
                    Some(output) => output.status,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Logs writing ended before child process did, exiting",
                        ));
                    }
                }
            } else if WINDOWS_SUBSYSTEM_WINDOWS {
                Self::maybe_force_renderer(cmd(&program, args))
                    .stdin_null()
                    .stdout_null()
                    .stderr_null()
                    // We use non-zero status codes and they don't mean error necessarily
                    .unchecked()
                    .run()?
                    .status
            } else {
                eprintln!("App data directory doesn't exist, not creating log file");
                Self::maybe_force_renderer(cmd(&program, args))
                    // We use non-zero status codes and they don't mean error necessarily
                    .unchecked()
                    .run()?
                    .status
            };

            match exit_status.code() {
                Some(status_code) => match AppStatusCode::from_status_code(status_code) {
                    AppStatusCode::Exit => {
                        eprintln!("Application exited gracefully");
                        break;
                    }
                    AppStatusCode::Restart => {
                        eprintln!("Restarting application");
                        continue;
                    }
                    AppStatusCode::Unknown(status_code) => {
                        eprintln!("Application exited with unexpected status code {status_code}");
                        process::exit(status_code);
                    }
                },
                None => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;

                        eprintln!(
                            "Application terminated by signal {:?}",
                            exit_status.signal()
                        );
                    }
                    #[cfg(not(unix))]
                    {
                        eprintln!("Application terminated by signal");
                    }
                    if last_start.elapsed() >= MIN_RUNTIME_DURATION_FOR_AUTORESTART {
                        self.after_crash = true;
                        continue;
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    fn app_data_dir() -> Option<PathBuf> {
        dirs::data_local_dir()
            .map(|data_local_dir| data_local_dir.join(env!("CARGO_PKG_NAME")))
            .and_then(|app_data_dir| {
                if !app_data_dir.exists() {
                    if let Err(error) = fs::create_dir_all(&app_data_dir) {
                        eprintln!(
                            "App data directory \"{}\" doesn't exist and can't be created: {}",
                            app_data_dir.display(),
                            error
                        );
                        return None;
                    }
                }

                Some(app_data_dir)
            })
    }

    fn new_logger(app_data_dir: &Path) -> FileRotate<AppendCount> {
        FileRotate::new(
            app_data_dir.join("space-acres.log"),
            AppendCount::new(LOG_FILE_LIMIT_COUNT),
            ContentLimit::Bytes(LOG_FILE_LIMIT_SIZE),
            Compression::OnRotate(0),
            #[cfg(unix)]
            Some(0o600),
        )
    }

    #[cfg(target_arch = "x86_64")]
    fn child_program() -> io::Result<PathBuf> {
        let program = env::current_exe()?;

        if !std::arch::is_x86_feature_detected!("xsavec") {
            return Ok(program);
        }

        let mut maybe_extension = program.extension();
        let Some(file_name) = program.file_stem() else {
            return Ok(program);
        };

        let mut file_name = file_name.to_os_string();

        if let Some(extension) = maybe_extension
            && extension != "exe"
        {
            file_name = program
                .file_name()
                .expect("Checked above; qed")
                .to_os_string();
            maybe_extension = None;
        }

        file_name.push("-modern");
        if let Some(extension) = maybe_extension {
            file_name.push(".");
            file_name.push(extension);
        }
        let mut modern_program = program.clone();
        modern_program.set_file_name(file_name);
        if modern_program.exists() {
            Ok(modern_program)
        } else {
            Ok(program)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn child_program() -> io::Result<PathBuf> {
        env::current_exe()
    }

    #[cfg(target_arch = "x86_64")]
    fn maybe_force_renderer(expression: Expression) -> Expression {
        if cfg!(windows) && !std::arch::is_x86_feature_detected!("xsavec") {
            // Force old GL renderer on Windows with old CPUs
            // TODO: This is a workaround for https://gitlab.gnome.org/GNOME/gtk/-/issues/6721
            expression.env("GSK_RENDERER", "gl")
        } else {
            expression
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn maybe_force_renderer(expression: Expression) -> Expression {
        expression
    }
}

fn main() -> ExitCode {
    raise_fd_limit();

    Cli::parse().run()
}
