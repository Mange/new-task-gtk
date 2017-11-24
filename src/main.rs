extern crate gtk;

use std::rc::Rc;
use gtk::prelude::*;
use gtk::{Entry, Window, WindowType, TextBuffer, TextView, Orientation, Builder, Revealer};

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

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let app = Rc::new(App::build(include_str!("window.glade")));

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
