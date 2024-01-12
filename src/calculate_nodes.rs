use bevy::{
    prelude::{Assets, Commands, Entity, In, Query, Res, With},
    utils::HashMap,
};
use kayak_font::{KayakFont, TextProperties};
use morphorm::Hierarchy;

use crate::{
    layout::{DataCache, Rect},
    node::{DirtyNode, Node, NodeBuilder, WrappedIndex},
    prelude::{KStyle, KayakRootContext, Tree},
    render::font::FontMapping,
    styles::{ComputedStyles, RenderCommand, StyleProp, Units},
};

pub fn calculate_nodes(
    In(mut context): In<KayakRootContext>,
    mut commands: Commands,
    fonts: Res<Assets<KayakFont>>,
    font_mapping: Res<FontMapping>,
    query: Query<Entity, With<DirtyNode>>,
    all_styles_query: Query<&ComputedStyles>,
    node_query: Query<(Entity, &Node)>,
    // widget_names: Query<&WidgetName>,
) -> KayakRootContext {
    let mut new_nodes = HashMap::<Entity, (Node, bool)>::default();

    context.current_z = 0.0;

    let initial_styles = KStyle::initial();
    let default_styles = KStyle {
        opacity: StyleProp::Value(1.0),
        ..KStyle::new_default()
    };

    if let Ok(tree) = context.tree.clone().try_read() {
        if tree.root_node.is_none() {
            return context;
        }

        let mut dirty_entities = query.iter().collect::<Vec<_>>();
        dirty_entities.sort_unstable_by(|a, b| a.index().partial_cmp(&b.index()).unwrap());

        for dirty_entity in dirty_entities {
            let dirty_entity = WrappedIndex(dirty_entity);
            if !tree.contains(dirty_entity) {
                continue;
            }

            let styles = all_styles_query.get(dirty_entity.0).map(|cs| &cs.0);

            let styles = styles.unwrap_or(&default_styles);

            // Get the parent styles. Will be one of the following:
            // 1. Already-resolved node styles (best)
            // 2. Unresolved widget prop styles
            // 3. Unresolved default styles
            let parent_styles = if let Some(parent_widget_id) = tree.parents.get(&dirty_entity) {
                if let Some(parent_node) = new_nodes.get(&parent_widget_id.0) {
                    parent_node.0.resolved_styles.clone()
                } else if let Ok((_, parent_node)) = node_query.get(parent_widget_id.0) {
                    parent_node.resolved_styles.clone()
                } else if let Ok(parent_styles) = all_styles_query.get(parent_widget_id.0) {
                    parent_styles.0.clone()
                } else {
                    default_styles.clone()
                }
            } else {
                default_styles.clone()
            };

            // let parent_z = if let Some(parent_widget_id) = tree.parents.get(&dirty_entity) {
            //     if let Some(parent_node) = new_nodes.get(&parent_widget_id.0) {
            //         parent_node.0.z
            //     } else if let Ok((_, parent_node)) = node_query.get(parent_widget_id.0) {
            //         parent_node.z
            //     } else {
            //         if let Ok(parent_styles) = all_styles_query.get(parent_widget_id.0) {
            //             parent_styles.z_index.resolve() as f32
            //         } else {
            //             -1.0
            //         }
            //     }
            // } else {
            //     -1.0
            // };

            let raw_styles = styles.clone();
            let mut styles = raw_styles.clone();
            // Fill in all `initial` values for any unset property
            styles.apply(&initial_styles);
            // Fill in all `inherited` values for any `inherit` property
            styles.inherit(&parent_styles);

            // Lock opacity so the max opacity for a child is the opacity of the parent.
            // if let StyleProp::Value(opacity) = &mut styles.opacity {
            //     if let StyleProp::Value(parent_opacity) = &parent_styles.opacity {
            //         *opacity = opacity.min(*parent_opacity);
            //     }
            // } else {
            //     styles.opacity = parent_styles.opacity.clone();
            // }

            // if let StyleProp::Value(opacity) = &mut styles.opacity {
            //     if let StyleProp::Value(background_color) = &mut styles.background_color {
            //         // Apply opacity to background_color
            //         background_color.set_a(opacity.min(background_color.a()));
            //     } else {
            //         styles.background_color = Color::rgba(1.0, 1.0, 1.0, *opacity).into();
            //     }

            //     if let StyleProp::Value(color) = &mut styles.color {
            //         // Apply opacity to color
            //         color.set_a(opacity.min(color.a()));
            //     } else {
            //         styles.color = Color::rgba(1.0, 1.0, 1.0, *opacity).into();
            //     }

            //     if let StyleProp::Value(border_color) = &mut styles.border_color {
            //         // Apply opacity to border_color
            //         border_color.set_a(opacity.min(border_color.a()));
            //     } else {
            //         styles.border_color = Color::rgba(1.0, 1.0, 1.0, *opacity).into();
            //     }
            // }

            // let mut current_z = {
            //     if parent_z > -1.0 {
            //         parent_z + 1.0
            //     } else {
            //         let z = context.current_z;
            //         context.current_z += 1.0;
            //         z
            //     }
            // };

            let current_z = if matches!(styles.z_index, StyleProp::Value(..)) {
                styles.z_index.resolve() as f32
            } else {
                -1.0
            };

            let needs_layout = create_primitive(
                &mut commands,
                &context,
                &fonts,
                &font_mapping,
                &query,
                // &node_query,
                dirty_entity,
                &mut styles,
                node_query
                    .get(dirty_entity.0)
                    .map(|(_, node)| node.raw_styles.clone().unwrap_or_default())
                    .unwrap_or_default(),
                &all_styles_query,
            );

            let children = tree
                .children
                .get(&dirty_entity)
                .cloned()
                .unwrap_or_default();

            // If a parent updates the children need to as well.
            // for child in children.iter() {
            //     commands.entity(child.0).insert(DirtyNode);
            // }

            let width = styles.width.resolve().value_or(0.0, 0.0);
            let height = styles.height.resolve().value_or(0.0, 0.0);

            let opacity = styles.opacity.resolve_or(1.0);

            let mut node: Node = NodeBuilder::empty()
                .with_id(dirty_entity)
                .with_styles(styles, Some(raw_styles))
                .with_children(children)
                .with_opacity(opacity)
                .build();

            if dirty_entity == tree.root_node.unwrap() {
                if let Ok(mut cache) = context.layout_cache.try_write() {
                    cache.rect.insert(
                        dirty_entity,
                        Rect {
                            posx: 0.0,
                            posy: 0.0,
                            width,
                            height,
                            z_index: None,
                        },
                    );
                }
            }
            node.old_z = node_query
                .get(dirty_entity.0)
                .map(|old_node| old_node.1.z)
                .unwrap_or(0.0);
            node.z = current_z;
            new_nodes.insert(dirty_entity.0, (node, needs_layout));
        }

        for (entity, (node, needs_layout)) in new_nodes.drain() {
            if !needs_layout {
                commands.entity(entity).remove::<DirtyNode>();
            } else {
                log::trace!("{:?} needs layout!", entity.index());
            }

            commands.entity(entity).insert(node);
        }
    }

    context
}

pub fn calculate_layout(
    In(context): In<KayakRootContext>,
    mut commands: Commands,
    nodes_no_entity_query: Query<&'static Node>,
) -> KayakRootContext {
    if let Ok(tree) = context.tree.try_read() {
        // tree.dump();
        let node_tree = &*tree;
        if let Ok(mut cache) = context.layout_cache.try_write() {
            let mut data_cache = DataCache {
                cache: &mut cache,
                query: &nodes_no_entity_query,
            };
            morphorm::layout(&mut data_cache, node_tree, &nodes_no_entity_query);

            for (entity, change) in cache.geometry_changed.iter() {
                if !change.is_empty() {
                    for child in tree.child_iter(*entity) {
                        // log::info!("Layout changed for: {:?}", child.0.id());
                        if let Some(mut entity_commands) = commands.get_entity(child.0) {
                            entity_commands.insert(DirtyNode);
                        }
                    }
                }
            }
        }
    }

    context
}

fn create_primitive(
    commands: &mut Commands,
    context: &KayakRootContext,
    fonts: &Assets<KayakFont>,
    font_mapping: &FontMapping,
    // query: &Query<(Entity, &Node)>,
    dirty: &Query<Entity, With<DirtyNode>>,
    id: WrappedIndex,
    styles: &mut KStyle,
    _prev_styles: KStyle,
    all_styles_query: &Query<&ComputedStyles>,
) -> bool {
    let mut needs_layout = true;
    if let StyleProp::Value(render_command) = &mut styles.render_command {
        match render_command {
            RenderCommand::Text {
                alignment,
                content,
                word_wrap,
                text_layout,
                properties,
                ..
            } => {
                let font = styles
                    .font
                    .resolve_or_else(|| String::from(crate::DEFAULT_FONT));
                // --- Bind to Font Asset --- //
                let font_handle = font_mapping.get_handle(font).unwrap();
                if let Some(font) = fonts.get(&font_handle) {
                    if let Ok(node_tree) = context.tree.try_read() {
                        if let Some(parent_id) =
                            find_not_empty_parent(&node_tree, all_styles_query, &id)
                        {
                            if let Some(parent_layout) = context.get_layout(&parent_id) {
                                let border_x = if let Ok(style) = all_styles_query.get(parent_id.0)
                                {
                                    let border = style.0.border.resolve();
                                    border.left + border.right
                                } else {
                                    0.0
                                };
                                let border_y = if let Ok(style) = all_styles_query.get(parent_id.0)
                                {
                                    let border = style.0.border.resolve();
                                    border.top + border.bottom
                                } else {
                                    0.0
                                };

                                let font_size = styles.font_size.resolve_or(14.0);
                                *properties = TextProperties {
                                    font_size,
                                    line_height: styles.line_height.resolve_or(font_size * 1.2),
                                    alignment: *alignment,
                                    ..*properties
                                };

                                properties.max_size = (
                                    parent_layout.width - border_x,
                                    parent_layout.height - border_y,
                                );

                                // TODO: Fix this hack.
                                if !*word_wrap {
                                    properties.max_size.0 = 100000.0;
                                }

                                needs_layout = false;

                                if properties.max_size.0 == 0.0 || properties.max_size.1 == 0.0 {
                                    needs_layout = true;
                                }

                                if context.get_geometry_changed(&parent_id) {
                                    needs_layout = true;
                                }

                                if dirty.contains(parent_id.0) {
                                    needs_layout = true;
                                }

                                // --- Calculate Text Layout --- //
                                *text_layout = font.measure(content, *properties);
                                let measurement = text_layout.size();

                                log::trace!(
                                    "Text Node: {}, has a measurement of: {:?}, it's parent takes up: {:?}",
                                    &content,
                                    measurement,
                                    properties.max_size
                                );

                                // --- Apply Layout --- //
                                if matches!(styles.width, StyleProp::Default) {
                                    styles.width = StyleProp::Value(Units::Pixels(measurement.0));
                                }
                                if matches!(styles.height, StyleProp::Default) {
                                    styles.height = StyleProp::Value(Units::Pixels(measurement.1));
                                }
                            } else {
                                log::trace!("no layout for: {:?}", parent_id.0.index());
                            }
                        } else {
                            log::trace!("No parent found for: {:?}", id.0.index());
                        }
                    }
                }
            }
            _ => {
                needs_layout = false;
            }
        }
    }

    if needs_layout {
        commands.entity(id.0).insert(DirtyNode);
    }

    // If we have data from the previous frame no need to do anything here!
    // if matches!(prev_styles.width, StyleProp::Value(..)) {
    //     styles.width = prev_styles.width;
    //     styles.height = prev_styles.height;
    //     needs_layout = false;
    // }

    needs_layout
}

pub fn find_not_empty_parent(
    tree: &Tree,
    all_styles_query: &Query<&ComputedStyles>,
    node: &WrappedIndex,
) -> Option<WrappedIndex> {
    if let Some(parent) = tree.parent(*node) {
        if let Ok(styles) = all_styles_query.get(parent.0) {
            if matches!(styles.0.render_command.resolve(), RenderCommand::Empty)
                || matches!(styles.0.render_command.resolve(), RenderCommand::Layout)
            {
                find_not_empty_parent(tree, all_styles_query, &parent)
            } else {
                Some(parent)
            }
        } else {
            find_not_empty_parent(tree, all_styles_query, &parent)
        }
    } else {
        None
    }
}

// pub fn build_nodes_tree(context: &mut Context, tree: &Tree, node_query: &Query<(Entity, &Node)>) {
//     if tree.root_node.is_none() {
//         return;
//     }
//     let mut node_tree = Tree::default();
//     node_tree.root_node = tree.root_node;
//     node_tree.children.insert(
//         tree.root_node.unwrap(),
//         get_valid_node_children(&tree, &node_query, tree.root_node.unwrap()),
//     );

//     // let old_focus = self.focus_tree.current();
//     // self.focus_tree.clear();
//     // self.focus_tree.add(root_node_id, &self.tree);

//     for (node_id, node) in node_query.iter() {
//         let node_id = WrappedIndex(node_id);
//         if let Some(widget_styles) = node.raw_styles.as_ref() {
//             // Only add widgets who have renderable nodes.
//             // if widget_styles.render_command.resolve() != RenderCommand::Empty {
//                 let valid_children = get_valid_node_children(&tree, &node_query, node_id);
//                 node_tree.children.insert(node_id, valid_children);
//                 let valid_parent = get_valid_parent(&tree, &node_query, node_id);
//                 if let Some(valid_parent) = valid_parent {
//                     node_tree.parents.insert(node_id, valid_parent);
//                 }
//             // }
//         }

//         // let focusable = self.get_focusable(widget_id).unwrap_or_default();
//         // if focusable {
//         //     self.focus_tree.add(widget_id, &self.tree);
//         // }
//     }

//     // if let Some(old_focus) = old_focus {
//     //     if self.focus_tree.contains(old_focus) {
//     //         self.focus_tree.focus(old_focus);
//     //     }
//     // }

//     // dbg!(&node_tree);

//     // context.node_tree = node_tree;
// }

// pub fn get_valid_node_children(
//     tree: &Tree,
//     query: &Query<(Entity, &Node)>,
//     node_id: WrappedIndex,
// ) -> Vec<WrappedIndex> {
//     let mut children = Vec::new();
//     if let Some(node_children) = tree.children.get(&node_id) {
//         for child_id in node_children {
//             if let Ok((_, _child_node)) = query.get(child_id.0) {
//                 // if child_node.resolved_styles.render_command.resolve() != RenderCommand::Empty {
//                     children.push(*child_id);
//                 // } else {
//                     // children.extend(get_valid_node_children(tree, query, *child_id));
//                 // }
//             } else {
//                 // children.extend(get_valid_node_children(tree, query, *child_id));
//             }
//         }
//     }

//     children
// }

// pub fn get_valid_parent(
//     tree: &Tree,
//     query: &Query<(Entity, &Node)>,
//     node_id: WrappedIndex,
// ) -> Option<WrappedIndex> {
//     if let Some(parent_id) = tree.parents.get(&node_id) {
//         if let Ok((_, parent_node)) = query.get(parent_id.0) {
//             // if parent_node.resolved_styles.render_command.resolve() != RenderCommand::Empty {
//                 return Some(*parent_id);
//             // }
//         }
//         // return get_valid_parent(tree, query, *parent_id);
//     }

//     None
// }
