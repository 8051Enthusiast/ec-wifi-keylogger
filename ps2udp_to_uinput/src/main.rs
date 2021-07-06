use pc_keyboard::{KeyCode as PcKey, KeyState, ScancodeSet2, layouts::Us104Key};
use std::net::UdpSocket;
use uinput::event::keyboard::Key as UinputKey;
fn event_ps2_to_uinput(from: PcKey) -> Option<UinputKey> {
    Some(match from {
        PcKey::AltLeft => UinputKey::LeftAlt,
        PcKey::AltRight => UinputKey::RightAlt,
        PcKey::ArrowDown => UinputKey::Down,
        PcKey::ArrowLeft => UinputKey::Left,
        PcKey::ArrowRight => UinputKey::Right,
        PcKey::ArrowUp => UinputKey::Up,
        PcKey::BackSlash => UinputKey::BackSlash,
        PcKey::Backspace => UinputKey::BackSpace,
        PcKey::BackTick => UinputKey::Grave,
        PcKey::BracketSquareLeft => UinputKey::LeftBrace,
        PcKey::BracketSquareRight => UinputKey::RightBrace,
        PcKey::CapsLock => UinputKey::CapsLock,
        PcKey::Comma => UinputKey::Comma,
        PcKey::ControlLeft => UinputKey::LeftControl,
        PcKey::ControlRight => UinputKey::RightControl,
        PcKey::Delete => UinputKey::Delete,
        PcKey::End => UinputKey::End,
        PcKey::Enter => UinputKey::Enter,
        PcKey::Escape => UinputKey::Esc,
        PcKey::Equals => UinputKey::Equal,
        PcKey::F1 => UinputKey::F1,
        PcKey::F2 => UinputKey::F2,
        PcKey::F3 => UinputKey::F3,
        PcKey::F4 => UinputKey::F4,
        PcKey::F5 => UinputKey::F5,
        PcKey::F6 => UinputKey::F6,
        PcKey::F7 => UinputKey::F7,
        PcKey::F8 => UinputKey::F8,
        PcKey::F9 => UinputKey::F9,
        PcKey::F10 => UinputKey::F10,
        PcKey::F11 => UinputKey::F11,
        PcKey::F12 => UinputKey::F12,
        PcKey::Fullstop => UinputKey::Dot,
        PcKey::Home => UinputKey::Home,
        PcKey::Insert => UinputKey::Insert,
        PcKey::Key1 => UinputKey::_1,
        PcKey::Key2 => UinputKey::_2,
        PcKey::Key3 => UinputKey::_3,
        PcKey::Key4 => UinputKey::_4,
        PcKey::Key5 => UinputKey::_5,
        PcKey::Key6 => UinputKey::_6,
        PcKey::Key7 => UinputKey::_7,
        PcKey::Key8 => UinputKey::_8,
        PcKey::Key9 => UinputKey::_9,
        PcKey::Key0 => UinputKey::_0,
        PcKey::Minus => UinputKey::Minus,
        PcKey::Numpad0 => UinputKey::_0,
        PcKey::Numpad1 => UinputKey::_1,
        PcKey::Numpad2 => UinputKey::_2,
        PcKey::Numpad3 => UinputKey::_3,
        PcKey::Numpad4 => UinputKey::_4,
        PcKey::Numpad5 => UinputKey::_5,
        PcKey::Numpad6 => UinputKey::_6,
        PcKey::Numpad7 => UinputKey::_7,
        PcKey::Numpad8 => UinputKey::_8,
        PcKey::Numpad9 => UinputKey::_9,
        PcKey::NumpadEnter => UinputKey::Enter,
        PcKey::NumpadLock => UinputKey::NumLock,
        PcKey::NumpadSlash => UinputKey::Slash,
        PcKey::NumpadMinus => UinputKey::Minus,
        PcKey::NumpadPeriod => UinputKey::Dot,
        PcKey::PageDown => UinputKey::PageDown,
        PcKey::PageUp => UinputKey::PageUp,
        PcKey::PrintScreen => UinputKey::SysRq,
        PcKey::ScrollLock => UinputKey::ScrollLock,
        PcKey::SemiColon => UinputKey::SemiColon,
        PcKey::ShiftLeft => UinputKey::LeftShift,
        PcKey::ShiftRight => UinputKey::RightShift,
        PcKey::Slash => UinputKey::Slash,
        PcKey::Spacebar => UinputKey::Space,
        PcKey::Tab => UinputKey::Tab,
        PcKey::Quote => UinputKey::Apostrophe,
        PcKey::WindowsLeft => UinputKey::LeftMeta,
        PcKey::WindowsRight => UinputKey::RightMeta,
        PcKey::A => UinputKey::A,
        PcKey::B => UinputKey::B,
        PcKey::C => UinputKey::C,
        PcKey::D => UinputKey::D,
        PcKey::E => UinputKey::E,
        PcKey::F => UinputKey::F,
        PcKey::G => UinputKey::G,
        PcKey::H => UinputKey::H,
        PcKey::I => UinputKey::I,
        PcKey::J => UinputKey::J,
        PcKey::K => UinputKey::K,
        PcKey::L => UinputKey::L,
        PcKey::M => UinputKey::M,
        PcKey::N => UinputKey::N,
        PcKey::O => UinputKey::O,
        PcKey::P => UinputKey::P,
        PcKey::Q => UinputKey::Q,
        PcKey::R => UinputKey::R,
        PcKey::S => UinputKey::S,
        PcKey::T => UinputKey::T,
        PcKey::U => UinputKey::U,
        PcKey::V => UinputKey::V,
        PcKey::W => UinputKey::W,
        PcKey::X => UinputKey::X,
        PcKey::Y => UinputKey::Y,
        PcKey::Z => UinputKey::Z,
        PcKey::Menus
        | PcKey::NumpadStar
        | PcKey::NumpadPlus
        | PcKey::PauseBreak
        | PcKey::HashTilde
        | PcKey::PrevTrack
        | PcKey::NextTrack
        | PcKey::Mute
        | PcKey::Calculator
        | PcKey::Play
        | PcKey::Stop
        | PcKey::VolumeDown
        | PcKey::VolumeUp
        | PcKey::WWWHome
        | PcKey::PowerOnTestOk => return None,
    })
}

fn main() -> eyre::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:10002")?;
    let mut decode =
        pc_keyboard::Keyboard::new(Us104Key, ScancodeSet2, pc_keyboard::HandleControl::Ignore);
    if let Err(e) = uinput::default() {
        eprintln!("{:?}", e);
    }
    let mut device = uinput::default()?
        .name("ps2udp")?
        .event(uinput::event::Keyboard::All)?
        .create()?;
    let mut buf = [0u8; 3];
    loop {
        let (amt, _) = socket.recv_from(&mut buf)?;
        for i in 0..amt {
            let ev = match decode.add_byte(buf[i]) {
                Ok(Some(ev)) => ev,
                Ok(None) | Err(_) => continue,
            };
            let code = match event_ps2_to_uinput(ev.code) {
                Some(code) => code,
                None => continue,
            };
            eprintln!("{:?}", code, ev.state);
            match ev.state {
                KeyState::Down => device.press(&code)?,
                KeyState::Up => device.release(&code)?,
            }
            device.synchronize()?;
        }
    }
}

