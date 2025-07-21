use bevy::{ecs::system::SystemState, prelude::*};
use common::editor::{ElementGroup, ParentGroup};

use crate::level::LevelElement;

use super::Tab;

pub fn build(app: &mut App) {
    // add_openable_tab::<ElementSelectionTab>(app, "Elements");

    // let inspector = GroupComponentInspector::from_world(&mut app.world);
    // add_component_inspector(app, inspector);
}

/// entities with this component are "selected"
#[derive(Component)]
pub struct SelectedElement;

#[derive(Component)]
pub struct LastSelectedElement;

// struct AddToGroupCommand {
//     group: Entity,
// }

// impl EntityCommand for AddToGroupCommand {
//     fn apply(self, id: Entity, world: &mut World) {
//         // make sure it isn't in a group
//         let Some(child_entity) = world.get_entity(id) else {
//             error!(
//                 "entity {:?} did not exist when trying to add it to group {:?}",
//                 id, self.group
//             );
//             return;
//         };

//         if child_entity.contains::<ParentGroup>() {
//             RemoveFromGroupCommand.apply(id, world);
//         }

//         let Some(mut group_entity) = world.get_entity_mut(self.group) else {
//             error!(
//                 "entity {:?} did not exist when trying to add {:?} to it's group",
//                 self.group, id
//             );
//             return;
//         };

//         if self.group == id {
//             warn!("added entity {:?} to it's own group", id);
//         }

//         if let Some(mut group) = group_entity.get_mut::<ElementGroup>() {
//             group.add(id);
//         } else {
//             group_entity.insert(ElementGroup::new([id]));
//         }

//         // already checked it exists
//         world
//             .entity_mut(id)
//             .insert(ParentGroup { group: self.group });
//     }
// }

// struct RemoveFromGroupCommand;

// impl EntityCommand for RemoveFromGroupCommand {
//     fn apply(self, id: Entity, world: &mut World) {
//         let Some(mut entity) = world.get_entity_mut(id) else {
//             error!(
//                 "entity {:?} didn't exist when trying to remove it from it's group",
//                 id
//             );
//             return;
//         };

//         let Some(ParentGroup {
//             group: group_entity,
//         }) = entity.take()
//         else {
//             warn!(
//                 "entity {:?} wasn't in a group when trying to remove it from one",
//                 id
//             );
//             return;
//         };

//         let Some(mut group) = world.get_entity_mut(group_entity) else {
//             error!(
//                 "when trying to remove entiy {:?} from it's group, it's parent group {:?} did not exist",
//                 id, group_entity
//             );
//             return;
//         };

//         let Some(mut group_component) = group.get_mut::<ElementGroup>() else {
//             error!(
//                 "when trying to remove entiy {:?} from it's group, it's parent group {:?} wasn't a group",
//                 id, group_entity
//             );
//             return;
//         };

//         group_component.remove(id);

//         if group_component.len() == 0 {
//             group.remove::<ElementGroup>();
//         }
//     }
// }

// pub trait GroupEntityCommands {
//     fn add_to_group(&mut self, new_group: Entity) -> &mut Self;

//     fn remove_from_group(&mut self) -> &mut Self;
// }

// impl<'a> GroupEntityCommands for EntityCommands<'a> {
//     fn add_to_group(&mut self, group: Entity) -> &mut Self {
//         self.add(AddToGroupCommand { group })
//     }

//     fn remove_from_group(&mut self) -> &mut Self {
//         self.add(RemoveFromGroupCommand)
//     }
// }

pub struct ElementSelectionTab {
    selection_state: SystemState<(
        Commands<'static, 'static>,
        Query<'static, 'static, Entity, (With<LevelElement>, Without<ParentGroup>)>,
        Query<
            'static,
            'static,
            (
                Option<&'static Name>,
                Has<SelectedElement>,
                Option<&'static ElementGroup>,
            ),
            With<LevelElement>,
        >,
        Query<'static, 'static, Entity, With<SelectedElement>>,
        Query<'static, 'static, Entity, With<LastSelectedElement>>,
        Res<'static, ButtonInput<KeyCode>>,
    )>,

    group_state: SystemState<(
        Commands<'static, 'static>,
        Query<'static, 'static, (Entity, Option<&'static ParentGroup>), With<SelectedElement>>,
    )>,

    ungroup_state: SystemState<(
        Commands<'static, 'static>,
        Query<
            'static,
            'static,
            (Entity, &'static ElementGroup, Option<&'static ParentGroup>),
            With<SelectedElement>,
        >,
    )>,
}

impl FromWorld for ElementSelectionTab {
    fn from_world(world: &mut World) -> Self {
        ElementSelectionTab {
            selection_state: SystemState::new(world),
            group_state: SystemState::new(world),
            ungroup_state: SystemState::new(world),
        }
    }
}

impl Tab for ElementSelectionTab {
    fn title(&self) -> &'static str {
        "Elements"
    }

    fn ui(&mut self, world: &mut World, ui: &mut bevy_egui::egui::Ui) {
        let (mut commands, element_q) = self.group_state.get_mut(world);

        if !element_q.is_empty() {
            if ui.button("Group Selected").clicked() {
                // all selected entities get placed into one group
                // if they are all in the same group, the new group gets placed in that group

                enum SharedGroupState {
                    /// haven't found a shared group
                    None,
                    /// found only one group, can place new group in it
                    Single(Entity),
                    /// multiple groups place at the top level
                    Multiple,
                }

                let group_entity = commands
                    .spawn((
                        Name::new("Element Group"),
                        LevelElement,
                        SelectedElement,
                        LastSelectedElement,
                    ))
                    .id();

                let mut state = SharedGroupState::None;
                let mut elements = Vec::new();

                for (element_entity, parent_group) in element_q.iter() {
                    elements.push(element_entity);

                    if let Some(&ParentGroup(group_entity)) = parent_group {
                        match state {
                            SharedGroupState::None => {
                                state = SharedGroupState::Single(group_entity)
                            }
                            SharedGroupState::Single(current) => {
                                if group_entity != current {
                                    state = SharedGroupState::Multiple
                                }
                            }
                            _ => (),
                        }

                        commands.entity(element_entity).remove::<ParentGroup>();
                    }

                    commands
                        .entity(element_entity)
                        .remove::<(SelectedElement, LastSelectedElement)>()
                        .insert(ParentGroup(group_entity));
                }

                if let SharedGroupState::Single(parent_group) = state {
                    commands
                        .entity(group_entity)
                        .insert(ParentGroup(parent_group));
                }
            }
        }

        self.group_state.apply(world);

        let (mut commands, group_q) = self.ungroup_state.get_mut(world);

        if !group_q.is_empty() {
            if ui.button("Ungroup Selected").clicked() {
                for (group_entity, group, parent_group) in group_q.iter() {
                    for child in group.iter() {
                        commands.entity(child).remove::<ParentGroup>();

                        if let Some(parent_group) = parent_group {
                            commands.entity(child).insert(ParentGroup(parent_group.0));
                        }
                    }

                    if parent_group.is_some() {
                        commands.entity(group_entity).remove::<ParentGroup>();
                    }

                    commands.entity(group_entity).despawn();
                }
            }
        }

        self.ungroup_state.apply(world);

        let (mut commands, top_level_q, element_q, selected_q, last_selected_q, input) =
            self.selection_state.get(world);

        for entity in top_level_q.iter() {
            show_entity(
                entity,
                0,
                ui,
                &mut commands,
                &element_q,
                &selected_q,
                &last_selected_q,
                &input,
            );
        }

        self.selection_state.apply(world);
    }
}

fn show_entity(
    element_entity: Entity,
    depth: usize,
    ui: &mut bevy_egui::egui::Ui,
    commands: &mut Commands,
    element_q: &Query<
        (Option<&Name>, Has<SelectedElement>, Option<&ElementGroup>),
        With<LevelElement>,
    >,
    selected_q: &Query<Entity, With<SelectedElement>>,
    last_selected_q: &Query<Entity, With<LastSelectedElement>>,
    input: &Res<ButtonInput<KeyCode>>,
) {
    let Ok((name, is_selected, group)) = element_q.get(element_entity) else {
        warn!(
            "tried to show selection ui for element {:?} which doesn't exist",
            element_entity
        );
        return;
    };

    let mut text = format!("{}{:?}", "  ".repeat(depth), element_entity);

    if let Some(name) = name {
        text.push_str(&format!(" {}", name.as_str()));
    }

    if let Some(group) = group {
        text.push_str(&format!(" ({})", group.len()));
    }

    let label = ui.selectable_label(is_selected, text);

    if label.clicked() {
        if !input.pressed(KeyCode::ControlLeft) {
            for entity in selected_q.iter() {
                commands
                    .entity(entity)
                    .remove::<(SelectedElement, LastSelectedElement)>();
            }
        }

        if is_selected {
            commands
                .entity(element_entity)
                .remove::<(SelectedElement, LastSelectedElement)>();
        } else {
            commands
                .entity(element_entity)
                .insert((SelectedElement, LastSelectedElement));

            for entity in last_selected_q.iter() {
                commands.entity(entity).remove::<LastSelectedElement>();
            }
        }
    }

    if let Some(group) = group {
        if label.secondary_clicked() {
            if !input.pressed(KeyCode::ControlLeft) {
                for entity in selected_q.iter() {
                    commands
                        .entity(entity)
                        .remove::<(SelectedElement, LastSelectedElement)>();
                }
            }

            for entity in group.iter() {
                commands.entity(entity).insert(SelectedElement);
            }
        }

        for entity in group.iter() {
            show_entity(
                entity,
                depth + 1,
                ui,
                commands,
                element_q,
                selected_q,
                last_selected_q,
                input,
            );
        }
    }
}

// struct GroupComponentInspector {
//     group_q_state: QueryState<(Option<&'static ElementGroup>, Option<&'static ParentGroup>)>,
//     name_q_state: QueryState<&'static Name>,
// }

// impl FromWorld for GroupComponentInspector {
//     fn from_world(world: &mut World) -> Self {
//         GroupComponentInspector {
//             group_q_state: world.query(),
//             name_q_state: world.query(),
//         }
//     }
// }

// impl ComponentInspector for GroupComponentInspector {
//     type Filter = Or<(With<ElementGroup>, With<ParentGroup>)>;

//     fn name(&self) -> String {
//         "Group Info".into()
//     }

//     fn ui(&mut self, entity: Entity, world: &mut World, ui: &mut bevy_egui::egui::Ui) {
//         let (group, parent_group) = self.group_q_state.get(world, entity).unwrap();

//         if let Some(group) = group {
//             ui.label(format!("Group with {} elements", group.len()));
//         }

//         if let Some(&ParentGroup { group }) = parent_group {
//             ui.label(if let Ok(name) = self.name_q_state.get(world, group) {
//                 format!("Parent group {:?} {}", group, name.as_str())
//             } else {
//                 format!("Parent group {:?}", group)
//             });
//         }
//     }
// }
