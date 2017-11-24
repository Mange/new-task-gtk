extern crate gtk;
extern crate xdg;

use std::rc::Rc;
use gtk::prelude::*;
use gtk::{Entry, Window, WindowType, TextBuffer, TextView, Orientation, Builder, Revealer,
          CssProvider, StyleContext};
use xdg::BaseDirectories;

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
        self.output_view.get_buffer().unwrap().set_text(&format!(
            "task add {}",
            text
        ));
        gtk::timeout_add_seconds(2, || {
            gtk::main_quit();
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
    StyleContext::add_provider_for_screen(&screen, &css_provider, 100);

    true
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let app = Rc::new(App::build(include_str!("window.glade")));

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
