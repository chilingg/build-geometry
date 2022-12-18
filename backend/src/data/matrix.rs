use super::prelude::*;

pub trait ProjectWorldToScreen {
    fn look_to(view_data: &ViewData) -> Self;
    fn look_to_range(view_data: &mut ViewData, size: WorldSize) -> Self;
}

impl ProjectWorldToScreen for ProjMatrix {
    fn look_to(view_data: &ViewData) -> Self {
        let c0r0 = 2.0 / view_data.size.width / view_data.pixel_size;
        let c1r1 = 2.0 / view_data.size.height / view_data.pixel_size;
        let c3r0 = -view_data.center.x * c0r0;
        let c3r1 = -view_data.center.y * c1r1;

        Self::new(
            c0r0, 0.0, 0.0, 0.0,
            0.0, c1r1, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            c3r0, c3r1, 0.0, 1.0,
        )
    }

    fn look_to_range(view_data: &mut ViewData, size: WorldSize) -> Self {
        let c0r0 = 2.0 / view_data.size.width / view_data.pixel_size;
        let c1r1 = 2.0 / view_data.size.height / view_data.pixel_size;
        let c3r0 = -view_data.center.x * c0r0;
        let c3r1 = -view_data.center.y * c1r1;

        Self::new(
            c0r0, 0.0, 0.0, 0.0,
            0.0, c1r1, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            c3r0, c3r1, 0.0, 1.0,
        )
    }
}

#[cfg(test)]
mod test_matrix {
    use super::*;

    #[test]
    fn test_wpoint_to_spoint() {
        let mut view_data = ViewData {
            size: ScreenSize::new(1024.0, 720.0),
            center: WorldPoint::new(-10.0, -10.0),
            pixel_size: 2.0,
        };
        let point = WorldPoint::new(12.0, 20.0);
        let mut proj_mat;

        proj_mat = ProjMatrix::look_to(&view_data);
        assert_eq!(proj_mat.transform_point2d(point).unwrap(), ScreenPoint::new(532.0, 330.0));

        proj_mat = ProjMatrix::look_to_range(
            &mut view_data,
            WorldSize::new(800.0, 800.0),
        );
        assert_eq!(proj_mat.transform_point2d(point).unwrap(), ScreenPoint::new(521.8, 323.0));
        assert_eq!(view_data.pixel_size, 0.9);
    }
}