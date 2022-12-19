pub mod matrix;
pub mod graph;
pub mod dirty_flag;
pub mod color;

use lyon::geom::euclid;

pub mod prelude {
    pub use super::matrix::ProjectWorldToViewport;
    pub use super::color;

    pub use lyon::path::Path;

    pub struct ScreenSpace;
    pub type ScreenPoint = super::euclid::Point2D<f32, ScreenSpace>;
    pub type ScreenSize = super::euclid::Size2D<f32, ScreenSpace>;
    
    pub struct WorldSpace;
    pub type WorldPoint = super::euclid::Point2D<f32, WorldSpace>;
    pub type WorldSize = super::euclid::Size2D<f32, WorldSpace>;
    pub type WorldVector = super::euclid::Vector2D<f32, WorldSpace>;
    
    pub struct ViewportSpace;
    pub type ViewportPoint = super::euclid::Point2D<f32, ViewportSpace>;
    pub type ProjMatrix = super::euclid::Transform3D<f32, WorldSpace, ViewportSpace>;

    pub use super::graph::GraphType;

    #[derive(Default)]
    pub struct ViewData {
        pub center: WorldPoint,
        pub size: ScreenSize,
        pub pixel_size: f32,
    }
    
    pub use super::dirty_flag::DirtyFlag;
}
