use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use ratatui::widgets::Widget;

fn main() {
    let mut picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
    let img = image::open("sample.jpg").unwrap();
    let mut protocol: StatefulProtocol = picker.new_resize_protocol(img);
    let image_widget = StatefulImage::default();
}
