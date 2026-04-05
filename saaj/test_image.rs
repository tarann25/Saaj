use ratatui_image::{picker::Picker, protocol::Protocol, StatefulImage};
use image::io::Reader;

fn main() {
    let img = Reader::open("sample.jpg").unwrap().decode().unwrap();
    let mut picker = Picker::from_termios().unwrap_or_else(|_| Picker::new((8, 16)));
    let mut protocol = picker.new_resize_protocol(img);
}
