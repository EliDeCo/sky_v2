use bevy::{prelude::*, window::PresentMode, window::PrimaryWindow};

#[derive(Component)]
struct CoordsText;

#[derive(Component)]
struct DropdownToggleButton;

#[derive(Component)]
struct DropdownLabel;

#[derive(Component)]
struct DropdownList;

#[derive(Component)]
struct DropdownOption {
    mode: PresentMode,
}

#[derive(Component)]
struct DropdownBackdrop;

#[derive(Resource, Default)]
struct DropdownState {
    open: bool,
}

const MODES: &[(PresentMode, &str)] = &[
    (PresentMode::AutoVsync, "AutoVsync"),
    (PresentMode::AutoNoVsync, "AutoNoVsync"),
    (PresentMode::Fifo, "Fifo"),
    (PresentMode::Mailbox, "Mailbox"),
    (PresentMode::Immediate, "Immediate"),
];

fn present_mode_label(mode: PresentMode) -> &'static str {
    MODES
        .iter()
        .find(|(m, _)| *m == mode)
        .map(|(_, s)| *s)
        .unwrap_or("Unknown")
}

pub struct PresentModeDropdownPlugin;

impl Plugin for PresentModeDropdownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DropdownState>()
            .add_systems(Startup, (spawn_dropdown_ui, spawn_coords_text))
            .add_systems(
                Update,
                (
                    dropdown_toggle_system,
                    dropdown_option_system,
                    dropdown_backdrop_system,
                    dropdown_hover_highlight,
                    update_coords_text,
                ),
            );
    }
}

fn spawn_dropdown_ui(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    let initial_label = present_mode_label(window.present_mode);

    // Full-screen backdrop — catches clicks outside the dropdown to dismiss it
    commands.spawn((
        Button,
        DropdownBackdrop,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(9),
    ));

    // Dropdown widget anchored to top-right corner
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(8.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            // Toggle button — shows current mode, opens/closes the list
            parent
                .spawn((
                    Button,
                    DropdownToggleButton,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(28.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
                    GlobalZIndex(11),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(initial_label),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        DropdownLabel,
                    ));
                });

            // Options list — hidden until toggle is clicked
            parent
                .spawn((
                    DropdownList,
                    Node {
                        width: Val::Px(160.0),
                        flex_direction: FlexDirection::Column,
                        display: Display::None,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.95)),
                    GlobalZIndex(10),
                ))
                .with_children(|list| {
                    for (mode, label) in MODES {
                        list.spawn((
                            Button,
                            DropdownOption { mode: *mode },
                            Node {
                                width: Val::Px(160.0),
                                height: Val::Px(26.0),
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ))
                        .with_children(|opt| {
                            opt.spawn((
                                Text::new(*label),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
        });
}

fn set_dropdown_open(
    open: bool,
    list_node: &mut Node,
    backdrop_node: &mut Node,
    state: &mut DropdownState,
) {
    state.open = open;
    list_node.display = if open { Display::Flex } else { Display::None };
    backdrop_node.display = if open { Display::Flex } else { Display::None };
}

fn dropdown_toggle_system(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<DropdownToggleButton>)>,
    mut list_q: Query<&mut Node, With<DropdownList>>,
    mut backdrop_q: Query<&mut Node, (With<DropdownBackdrop>, Without<DropdownList>)>,
    mut state: ResMut<DropdownState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            let new_open = !state.open;
            if let (Ok(mut list), Ok(mut backdrop)) =
                (list_q.single_mut(), backdrop_q.single_mut())
            {
                set_dropdown_open(new_open, &mut list, &mut backdrop, &mut state);
            }
        }
    }
}

fn dropdown_option_system(
    interaction_q: Query<(&Interaction, &DropdownOption), Changed<Interaction>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut label_q: Query<&mut Text, With<DropdownLabel>>,
    mut list_q: Query<&mut Node, With<DropdownList>>,
    mut backdrop_q: Query<&mut Node, (With<DropdownBackdrop>, Without<DropdownList>)>,
    mut state: ResMut<DropdownState>,
) {
    for (interaction, option) in &interaction_q {
        if *interaction == Interaction::Pressed {
            window.present_mode = option.mode;
            info!("PresentMode changed: {:?}", option.mode);
            if let Ok(mut label) = label_q.single_mut() {
                *label = Text::new(present_mode_label(option.mode));
            }
            if let (Ok(mut list), Ok(mut backdrop)) =
                (list_q.single_mut(), backdrop_q.single_mut())
            {
                set_dropdown_open(false, &mut list, &mut backdrop, &mut state);
            }
        }
    }
}

fn dropdown_backdrop_system(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<DropdownBackdrop>)>,
    mut list_q: Query<&mut Node, With<DropdownList>>,
    mut backdrop_q: Query<&mut Node, (With<DropdownBackdrop>, Without<DropdownList>)>,
    mut state: ResMut<DropdownState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            if let (Ok(mut list), Ok(mut backdrop)) =
                (list_q.single_mut(), backdrop_q.single_mut())
            {
                set_dropdown_open(false, &mut list, &mut backdrop, &mut state);
            }
        }
    }
}

fn dropdown_hover_highlight(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DropdownOption>),
    >,
) {
    for (interaction, mut bg) in &mut interaction_q {
        bg.0 = match interaction {
            Interaction::Hovered => Color::srgba(0.25, 0.25, 0.25, 1.0),
            _ => Color::srgba(0.0, 0.0, 0.0, 0.0),
        };
    }
}

fn spawn_coords_text(mut commands: Commands) {
    commands.spawn((
        CoordsText,
        Text::new("x: 0.00  y: 0.00  z: 0.00"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(0.0),
            ..default()
        },
    ));
}

fn update_coords_text(
    camera_q: Single<&GlobalTransform, With<Camera3d>>,
    mut text_q: Query<&mut Text, With<CoordsText>>,
) {
    let pos = camera_q.translation();
    if let Ok(mut text) = text_q.single_mut() {
        *text = Text::new(format!(
            "x: {:.2}  y: {:.2}  z: {:.2}",
            pos.x, pos.y, pos.z
        ));
    }
}
