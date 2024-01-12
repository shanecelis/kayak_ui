use bevy::prelude::{Bundle, Color, Commands, Component, Entity, In, Query, Res};
use kayak_ui::prelude::{
    rsx, widgets::BackgroundBundle, ComputedStyles, Edge, KChildren, KStyle, KayakWidgetContext,
    StyleProp, Units, Widget, WidgetName,
};

use crate::tab_context::TabContext;

#[derive(Component, Default, PartialEq, Eq, Clone)]
pub struct Tab {
    pub index: usize,
}

impl Widget for Tab {}

#[derive(Bundle)]
pub struct TabBundle {
    pub tab: Tab,
    pub children: KChildren,
    pub styles: ComputedStyles,
    pub widget_name: WidgetName,
}

impl Default for TabBundle {
    fn default() -> Self {
        Self {
            tab: Default::default(),
            children: Default::default(),
            styles: ComputedStyles::default(),
            widget_name: Tab::default().get_name(),
        }
    }
}

pub fn tab_render(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<(&KChildren, &Tab, &mut ComputedStyles)>,
    tab_context_query: Query<&TabContext>,
) -> bool {
    if let Ok((children, tab, mut styles)) = query.get_mut(entity) {
        let context_entity = widget_context
            .get_context_entity::<TabContext>(entity)
            .unwrap();
        if let Ok(tab_context) = tab_context_query.get(context_entity) {
            if tab_context.current_index == tab.index {
                styles.0.height = StyleProp::default();
                styles.0.width = StyleProp::default();
                let parent_id = Some(entity);
                let styles = KStyle {
                    background_color: StyleProp::Value(Color::rgba(0.0781, 0.0898, 0.101, 1.0)),
                    padding: StyleProp::Value(Edge::all(Units::Pixels(15.0))),
                    height: Units::Stretch(1.0).into(),
                    width: Units::Stretch(1.0).into(),
                    ..Default::default()
                };
                rsx! {
                    <BackgroundBundle styles={styles} children={children.clone()} />
                };
            } else {
                styles.0.height = StyleProp::Value(Units::Pixels(0.0));
                styles.0.width = StyleProp::Value(Units::Pixels(0.0));
            }
        }
    }
    true
}
