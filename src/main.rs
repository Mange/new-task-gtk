extern crate gtk;
extern crate gdk;
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
        self.output_view.grab_focus();

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

    fn delete_word(&self) {
        if !self.entry.get_editable() {
            return;
        }

        let cursor_position = self.entry.get_property_cursor_position() as usize;
        let text = self.entry.get_text().unwrap_or_else(|| String::from(""));
        assert!(text.len() >= cursor_position);

        let (from, length) = delete_word_backwards(&text, cursor_position);
        self.entry.delete_text(from as i32, length as i32);
    }

    fn handle_key(&self, event: &gdk::EventKey) {
        use gdk::enums::key;

        let keyval = event.get_keyval();
        let modifiers = event.get_state();

        if keyval == key::Escape {
            App::quit();
        } else if keyval == key::w && modifiers.contains(gdk::CONTROL_MASK) {
            self.delete_word();
        }
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
        app.entry.connect_activate(
            move |entry: &Entry| if entry.get_editable() {
                app2.add_task(entry.get_text().unwrap_or_else(|| String::from("")));
            },
        );
    }

    {
        let app2 = app.clone();
        app.window.connect_key_release_event(move |_, key| {
            app2.handle_key(key);
            Inhibit(false)
        });
    }

    gtk::main();
}

fn delete_word_backwards(text: &str, cursor_position: usize) -> (usize, usize) {
    let text_before_cursor = &text[0..cursor_position as usize];

    if let Some(position) = text_before_cursor.rfind(char::is_whitespace) {
        // Try to delete all consecutive whitespace by searching for the next non-whitespace
        // character and delete back to there.
        // Note that position must be advanced once again or else that first non-whitespace
        // character will be included in the deletion range.
        // As we are searching from the right and we got a position, the new position must be at
        // least 1 lower than the previous position, so it should be safe to +1 it again.
        // At worst we'll end up on the same numbers as before this branch.
        if let Some(position) = (&text[0..position]).rfind(|c: char| !c.is_whitespace()) {
            (position + 1, cursor_position - position - 1)
        } else {
            (position, cursor_position - position)
        }
    } else {
        // Delete to beginning of entry
        (0, cursor_position)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod delete_word {
        use super::*;

        fn simulate(input: &str, position: usize) -> String {
            let (from, length) = delete_word_backwards(input, position);
            if length == 0 {
                input.to_owned()
            } else {
                let string = String::from(&input[0..from]);
                string + &input[(from + length)..input.len()]
            }
        }

        #[test]
        fn it_deletes_nothing_on_empty_input() {
            let input = "";
            assert_eq!(delete_word_backwards(input, 0), (0, 0));
        }

        #[test]
        fn it_deletes_all_on_no_whitespace() {
            let input = "123";
            assert_eq!(delete_word_backwards(input, 3), (0, 3));
            assert_eq!(&simulate(input, 3), "");
        }

        #[test]
        fn it_deletes_to_cursor_if_middle_of_word() {
            let input = "12345";
            assert_eq!(delete_word_backwards(input, 3), (0, 3));
            assert_eq!(&simulate(input, 3), "45");
        }

        #[test]
        fn it_deletes_last_word_and_space() {
            let input = "AAA BBB";
            assert_eq!(delete_word_backwards(input, 7), (3, 4));
            assert_eq!(&simulate(input, 7), "AAA");
        }

        #[test]
        fn it_deletes_previous_word_and_space() {
            let input = "AAA BBB CCC";
            assert_eq!(delete_word_backwards(input, 7), (3, 4));
            assert_eq!(&simulate(input, 7), "AAA CCC");
        }

        #[test]
        fn it_deletes_consecutive_whitespace() {
            let input = "AAA   BBB";
            // Three spaces
            assert_eq!(delete_word_backwards(input, 3 + 3 + 3), (3, 6));
            assert_eq!(&simulate(input, 3 + 3 + 3), "AAA");
        }
    }
}
