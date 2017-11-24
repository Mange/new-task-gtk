extern crate gtk;

use std::rc::Rc;
use gtk::prelude::*;
use gtk::{Entry, Window, WindowType, TextBuffer, TextView, Orientation};
use gtk::Box as GtkBox;

struct App {
    window: Window,

    entry: Entry,

    vertical_box: GtkBox,
    command_output: TextBuffer,
    command_output_view: TextView,
}

impl App {
    fn new(title: &str) -> App {
        let window = Window::new(WindowType::Toplevel);
        window.set_title(title);
        window.set_default_size(350, 70);

        let vertical_box = GtkBox::new(Orientation::Vertical, 5);
        let entry = Entry::new();
        let command_output = TextBuffer::new(None);
        let command_output_view = TextView::new_with_buffer(&command_output);

        vertical_box.pack_start(&entry, false, false, 0);

        window.add(&vertical_box);
        window.show_all();

        App {
            window: window,
            entry: entry,
            vertical_box: vertical_box,
            command_output: command_output,
            command_output_view: command_output_view,
        }
    }

    fn add_task(&self, text: String) {
        self.vertical_box.pack_end(
            &self.command_output_view,
            true,
            true,
            5,
        );
        self.command_output_view.show();
        self.entry.hide();

        self.command_output.set_text(&format!("task add {}", text));
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

    let app = Rc::new(App::new("New Task"));

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
