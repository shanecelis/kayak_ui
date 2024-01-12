use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

#[derive(Debug, Component, Default, Clone, PartialEq, Eq)]
pub struct MyWidget {
    pub foo: u32,
}

fn my_widget_1_render(
    In(entity): In<Entity>,
    mut _commands: Commands,
    query: Query<&MyWidget>,
) -> bool {
    if let Ok(my_widget) = query.get(entity) {
        dbg!(my_widget.foo);
    }

    true
}

impl Widget for MyWidget {}

fn startup(mut commands: Commands) {
    let camera_entity = commands
        .spawn((Camera2dBundle::default(), CameraUIKayak))
        .id();

    let mut context = KayakRootContext::new(camera_entity);
    context.add_plugin(KayakWidgetsContextPlugin);
    context.add_widget_system(
        MyWidget::default().get_name(),
        widget_update::<MyWidget, EmptyState>,
        my_widget_1_render,
    );
    context.add_widget_data::<MyWidget, EmptyState>();

    let app_entity = commands.spawn_empty().id();
    let mut children = KChildren::default();
    let entity = commands
        .spawn((
            MyWidget { foo: 0 },
            kayak_ui::prelude::KStyle::default(),
            MyWidget::default().get_name(),
        ))
        .id();
    children.add(entity);

    commands.entity(app_entity).insert(KayakAppBundle {
        children,
        ..KayakAppBundle::default()
    });
    context.add_widget(None, app_entity);

    commands.spawn((context, EventDispatcher::default()));
}

// Note this example shows prop changing not state changing which is quite different.
// For state changes please see simple_state example.
fn update_resource(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut MyWidget, Without<PreviousWidget>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut my_widget in query.iter_mut() {
            my_widget.foo += 1;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .add_systems(Update, update_resource)
        .run()
}
