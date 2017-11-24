extern crate gtk;

use gtk::prelude::*;
use gtk::{Entry, Window, WindowType};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("New Task");
    window.set_default_size(350, 70);
    let entry = Entry::new();
    window.add(&entry);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    entry.connect_activate(|entry: &Entry| {
        if let Some(text) = entry.get_text() {
            println!("{}", text);
        }
        gtk::main_quit();
    });

    gtk::main();
}
