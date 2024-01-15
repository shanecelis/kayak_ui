use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    asset::{AssetServer, embedded_asset, embedded_path},
};
use kayak_ui::{
    prelude::{widgets::*, *},
    CameraUIKayak,
};

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct MyUIMaterial {}
impl MaterialUI for MyUIMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        // "rainbow_shader.wgsl".into()
        // "embedded://custom_shader/rainbow_embed.wgsl".into()
        // "embedded://custom_shader/examples/rainbow_embed.wgsl".into()
        // "embedded://kayak_ui/examples/rainbow_embed.wgsl".into()
        // "embedded://kayak_ui/examples/rainbow_embed.wgsl".into()
        "embedded://custom_shader/rainbow_embed.wgsl".into()
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MyUIMaterial>>,
) {
    let camera_entity = commands
        .spawn(Camera2dBundle::default())
        .insert(CameraUIKayak)
        .id();

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let my_material = MyUIMaterial {};
    let my_material_handle = materials.add(my_material);

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <TextWidgetBundle
                styles={KStyle {
                    position_type: KPositionType::SelfDirected.into(),
                    left: Units::Pixels(20.0).into(),
                    top: Units::Pixels(5.0).into(),
                    material: MaterialHandle::new(move |commands, entity| {
                        commands.entity(entity).insert(my_material_handle.clone_weak());
                        // commands.entity(entity).insert(my_material_handle.clone());
                    }).into(),
                    ..Default::default()
                }}
                text={TextProps {
                    content: "Hello Shader!".into(),
                    size: 100.0,
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut asset_server: ResMut<AssetServer>,
) {
    if keys.just_pressed(KeyCode::Space) {
        // Space was pressed
        eprintln!("reload sample quad");

        asset_server.reload("embedded://custom_shader/rainbow_embed.wgsl");
        // asset_server.reload("embedded://kayak_ui/render/unified/shaders/sample_quad.wgsl");
        // asset_server.reload("rainbow_shader.wgsl");
    }
}

fn main() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins);
    // BUG
    // embedded_asset!(app, "examples", "rainbow_embed.wgsl");
    embedded_asset!(app, "examples/", "rainbow_embed.wgsl");
    // dbg!(embedded_path!("examples/", "rainbow_embed.wgsl"));

    app
        .add_plugins((
            KayakContextPlugin,
            KayakWidgets,
            MaterialUIPlugin::<MyUIMaterial>::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, keyboard_input);

    app
        .run()
}
