#[derive(Debug, Copy, Clone)]
pub struct MoveEvent {
    pub rotation: i32,
    pub movement: i32,
    pub teleop: bool,
}
