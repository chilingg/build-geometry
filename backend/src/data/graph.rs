use super::prelude::*;

pub enum GraphType {
    Line { from: WorldPoint, to: WorldPoint },
    Circle { center: WorldPoint, radius: f32 },
    Point(WorldPoint),
}