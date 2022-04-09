#[derive(Debug, Copy, Clone)]
pub struct TurtleDirection {
    pub rotation: i32,
    pub movement: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct MoveEvent {
    pub rotation: i32,
    pub movement: i32,
}
