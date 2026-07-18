use ratatui::widgets::Widget;

pub trait Block<T: Handle> {
    fn dispatch(self) -> T;
}

pub trait Handle {
    fn get_widget(&self) -> impl Widget;
}
