use ratatui::{
    Frame,
    widgets::Widget,
};

pub fn render<T: Widget>(frame: &mut Frame, widget: T) {
    frame.render_widget(widget, frame.area());
}
