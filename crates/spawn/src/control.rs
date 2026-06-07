use crate::config::Signal;

#[derive(Debug)]
pub enum ControlMessage {
    Input(Vec<u8>),
    CloseStdin,
    Resize { rows: u16, cols: u16 },
    Cancel,
    Signal(Signal),
    ForceKill,
}
