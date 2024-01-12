use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::{Bundle, Component, GlobalTransform, Transform, Vec2},
    render::{
        camera::{Camera, CameraProjection, CameraRenderGraph},
        extract_component::ExtractComponent,
        primitives::Frustum,
        view::VisibleEntities,
    },
};

use crate::{context::KayakRootContext, event_dispatcher::EventDispatcher};

use super::ortho::UIOrthographicProjection;

/// Kayak UI's default UI camera.
#[derive(Component, ExtractComponent, Clone, Default)]
pub struct CameraUIKayak {
    pub clear_color: ClearColorConfig,
}

/// Kayak UI's default UI camera bundle.
#[derive(Bundle)]
pub struct UICameraBundle {
    pub camera: Camera,
    // pub camera_2d: Camera2d,
    pub camera_render_graph: CameraRenderGraph,
    pub orthographic_projection: UIOrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_ui: CameraUIKayak,
    pub context: KayakRootContext,
    pub event_disaptcher: EventDispatcher,
}

impl Default for UICameraBundle {
    fn default() -> Self {
        Self::new(KayakRootContext::default())
    }
}

impl UICameraBundle {
    pub const UI_CAMERA: &'static str = "KAYAK_UI_CAMERA";
    pub fn new(kayak_root_context: KayakRootContext) -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let far = 1000.0;

        let orthographic_projection = UIOrthographicProjection {
            far,
            window_origin: Vec2::new(0.0, 0.0),
            ..Default::default()
        };

        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);

        let view_projection =
            orthographic_projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection(&view_projection);
        UICameraBundle {
            camera: Camera {
                order: isize::MAX - 1,
                ..Default::default()
            },
            camera_render_graph: CameraRenderGraph::new(bevy::core_pipeline::core_2d::graph::NAME),
            orthographic_projection,
            frustum,
            visible_entities: VisibleEntities::default(),
            transform,
            // camera_2d: Camera2d::default(),
            global_transform: Default::default(),
            camera_ui: CameraUIKayak {
                clear_color: ClearColorConfig::None,
            },
            context: kayak_root_context,
            event_disaptcher: EventDispatcher::new(),
        }
    }
}
