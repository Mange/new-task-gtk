extern crate gtk;
extern crate xdg;

mod command;

use std::rc::Rc;
use gtk::prelude::*;
use gtk::{Entry, Window, TextView, Builder, Revealer, CssProvider, StyleContext, TextBuffer};
use xdg::BaseDirectories;
use command::{TaskWarrior, CommandStream};

struct App {
    window: Window,
    entry: Entry,
    revealer: Revealer,
    output_view: TextView,
}

impl App {
    fn build(glade: &str) -> App {
        let builder = Builder::new_from_string(glade);
        let window: Window = builder.get_object("window").unwrap();
        let entry = builder.get_object("entry").unwrap();
        let revealer = builder.get_object("revealer").unwrap();
        let output_view = builder.get_object("output_view").unwrap();

        window.set_default_size(800, -1);

        App {
            window: window,
            entry: entry,
            revealer: revealer,
            output_view: output_view,
        }
    }

    fn add_task(&self, text: String) {
        self.revealer.set_reveal_child(true);
        self.entry.set_editable(false);

        if text.len() > 0 {
            let stream = TaskWarrior::add(&text);

            match stream {
                Ok(stream) => self.run_task_command(stream),
                Err(error) => self.show_task_error(error),
            }
        } else {
            App::quit();
        }
    }

    fn run_task_command(&self, mut stream: CommandStream) {
        let output_buffer = self.output_view.get_buffer().unwrap().clone();

        fn insert_into_buffer(buffer: &TextBuffer, text: &str) {
            let (_, mut end) = buffer.get_bounds();
            buffer.insert(&mut end, text);
        }

        gtk::timeout_add(100, move || {
            use command::StreamStatus::*;

            loop {
                match stream.try_next_line() {
                    Line(line) => {
                        insert_into_buffer(&output_buffer, &line);
                    }
                    Wait => return Continue(true),
                    Complete => {
                        App::quit_after_seconds(1);
                        return Continue(false);
                    }
                    Failed(code) => {
                        let message = format!("\nProgram exited with {} exit code\n", code);
                        insert_into_buffer(&output_buffer, &message);
                        return Continue(false);
                    }
                    Error(message) => {
                        let message = format!("\nERROR: {}\n", message);
                        insert_into_buffer(&output_buffer, &message);
                        return Continue(false);
                    }
                }
            }
        });

    }

    fn show_task_error(&self, error: String) {
        self.output_view.get_buffer().unwrap().set_text(&error);
    }

    fn quit() {
        gtk::main_quit();
    }

    fn quit_after_seconds(seconds: u32) {
        gtk::timeout_add_seconds(seconds, || {
            App::quit();
            Continue(false)
        });
    }
}

fn apply_custom_stylesheets(app: &App) -> bool {
    let css_provider = CssProvider::new();

    css_provider.connect_parsing_error(|_, _section, error| {
        eprintln!("Could not load stylesheet: {}", error);
    });

    let base_dir =
        BaseDirectories::with_prefix("new-task-gtk").expect("Could not access your HOME directory");

    if let Some(style_file) = base_dir.find_config_file("style.css") {
        if let Some(utf8_path) = style_file.to_str() {
            if css_provider.load_from_path(utf8_path).is_err() {
                return false;
            }
        } else {
            eprintln!(
                "Could not load stylesheet; file path not valid UTF-8: {}",
                style_file.to_string_lossy(),
            );
        }
    }

    let screen = app.window.get_screen().unwrap();
    StyleContext::add_provider_for_screen(&screen, &css_provider, 2000);

    true
}

fn apply_default_stylesheets(app: &App) {
    let css_provider = CssProvider::new();

    css_provider.connect_parsing_error(|_, _section, error| {
        eprintln!("Could not load stylesheet: {}", error);
    });

    CssProviderExt::load_from_data(&css_provider, include_bytes!("default.css"))
        .expect("Default styles are invalid!");

    let screen = app.window.get_screen().unwrap();
    StyleContext::add_provider_for_screen(&screen, &css_provider, 1000);
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let app = Rc::new(App::build(include_str!("window.glade")));

    apply_default_stylesheets(&app);
    if !apply_custom_stylesheets(&app) {
        std::process::exit(1);
    }

    app.window.show_all();

    app.window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    {
        let app2 = app.clone();
        app.entry.connect_activate(move |entry: &Entry| {
            app2.add_task(entry.get_text().unwrap_or_else(|| String::from("")));
        });
    }

    gtk::main();
}
