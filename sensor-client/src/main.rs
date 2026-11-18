use gtk4::{ CssProvider, gdk::Display, prelude::*, Label };
use libadwaita::Application as AdwApplication;
use gtk4::ApplicationWindow as AdwWindow;

const APP_ID: &str = "com.cardoor.system";


fn main() {
    // Create a new application
    let app = AdwApplication::builder() // Use AdwApplication
        .application_id(APP_ID)
        .build();

    let _ = gtk4::init();
    // load and inject theme.css file
    let provider = CssProvider::new();
    provider.load_from_path("src/theme.css"); 
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    app.connect_activate(main_window);
    app.run();

}


fn main_window(app: &AdwApplication) {
    // Create a new window
    let window = AdwWindow::builder() // Use AdwWindow
        .application(app)
        .title("Card Door")
        .default_width(900)
        .default_height(800)
        .build();
    let header_bar = libadwaita::HeaderBar::new();
    window.set_titlebar(Some(&header_bar));
    
    let test_label = Label::builder()
        .label("Testing Text")
        .build();

    window.set_child(Some(&test_label));
}