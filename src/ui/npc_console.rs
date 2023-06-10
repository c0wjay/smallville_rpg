use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
    window::PrimaryWindow,
};
use rand::Rng;
use sysinfo::{ProcessorExt, System, SystemExt, UserExt};

use crate::{
    components::{Coordinate, CurrentInteractingNPC, FaceDirection, Facing, MoveLock, Player, NPC},
    map::EntityMap,
    state::AppState,
};

const CONSOLE_HEIGHT: f32 = 0.4;

#[derive(Component)]
pub struct LogsArea;
#[derive(Component)]
pub struct CommandLineText;
#[derive(Component)]
pub struct ConsoleUI;

// TODO: `ConsoleData`, `PrintConsoleEvent`, `EnteredConsoleCommandEvent` should contains info about NPC (which interacts with player), to separate response by each NPCs.
// Maybe `ConsoleData` will be child of NPC entity, not direct Component of it.
// `ConsoleData` should be a component that NPC entities take. `PrintConsoleEvent` and `EnteredConsoleCommandEvent` should have field that describes which NPC entities emit them.
#[derive(Default, Component, Debug, Reflect)]
pub struct ConsoleData {
    pub typed_command: String,
    pub is_opening: bool,
    pub fully_opened: bool,
    pub messages: Vec<String>,
}
pub struct PrintConsoleEvent {
    pub npc: Entity,
    pub message: String,
}
pub struct EnteredConsoleCommandEvent {
    pub npc: Entity,
    pub message: String,
}

// pushes messages in event `PrintConsoleEvent` to `ConsoleData.messages`.
pub fn push_message_events_to_console(
    mut npc_query: Query<Entity, With<NPC>>,
    mut data_query: Query<(&Parent, &mut ConsoleData)>,
    mut ev_console_message: EventReader<PrintConsoleEvent>,
) {
    for PrintConsoleEvent { npc, message } in ev_console_message.iter() {
        for (parent, mut data) in data_query.iter_mut() {
            if let Ok(parent_npc) = npc_query.get(parent.get()) {
                if parent_npc.eq(npc) {
                    data.messages.push(message.clone());
                }
            }
        }
    }
}

#[derive(Default, Resource)]
pub struct ConsoleAnimation {
    pub start_position: Vec2,
    pub end_position: Vec2,
    pub moving_speed: f64,
    pub time_to_move: f64,
    pub start_time: f64,
}

#[derive(Component, Default, Debug)]
pub struct ScrollingList {
    position: f32,
}

pub fn spawn_console_data_in_npc(
    mut commands: Commands,
    mut npc_query: Query<Entity, Added<NPC>>,
    mut console_writer: EventWriter<PrintConsoleEvent>,
) {
    for npc in npc_query.iter_mut() {
        let child = commands.spawn(ConsoleData::default()).id();
        commands.entity(npc).push_children(&[child]);

        // TODO: This would be modified. motd should contains about NPC info.
        console_writer.send(PrintConsoleEvent {
            npc,
            message: print_motd(&mut System::new(), true),
        });
    }
}

pub fn build_ui(
    mut commands: Commands,
    mut anim_data: ResMut<ConsoleAnimation>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(current_window) = window.get_single() else {
        return;
    };

    // move away the window
    anim_data.start_position = Vec2::new(0.0, current_window.height());
    anim_data.end_position = anim_data.start_position;

    // building the background color
    let background_component = NodeBundle {
        style: Style {
            size: Size::new(
                Val::Px(current_window.width()),
                Val::Px(CONSOLE_HEIGHT * current_window.height()),
            ),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect {
                left: Val::Percent(2.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
        background_color: Color::rgba_u8(5, 17, 0, 255).into(),
        ..Default::default()
    };

    let transparent_col = Color::rgba_u8(0, 0, 0, 0);

    commands
        .spawn(background_component)
        .insert(ConsoleUI {})
        .with_children(|parent| {
            // command textbox container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(0.75 * current_window.width()), Val::Percent(10.0)),
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::ColumnReverse,
                        flex_wrap: FlexWrap::Wrap,
                        ..Default::default()
                    },
                    background_color: transparent_col.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // command textbox area
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                // size: Size::new(
                                //     Val::Px(0.75 * current_window.width()),
                                //     Val::Percent(100.0),
                                // ),
                                flex_wrap: FlexWrap::Wrap,
                                position: UiRect {
                                    bottom: Val::Percent(0.0),
                                    ..default()
                                },
                                ..Default::default()
                            },
                            background_color: transparent_col.into(),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle {
                                    style: Style {
                                        // size: Size::new(
                                        //     Val::Px(0.75 * current_window.width()),
                                        //     Val::Percent(10.0),
                                        // ),
                                        flex_wrap: FlexWrap::Wrap,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .insert(CommandLineText);
                        });
                });

            // command logs container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(0.75 * current_window.width()), Val::Percent(85.0)),
                        align_self: AlignSelf::Stretch,
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::ColumnReverse,
                        flex_wrap: FlexWrap::Wrap,
                        overflow: Overflow::Hidden,
                        ..Default::default()
                    },
                    background_color: transparent_col.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // logs area
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    // size: Size::new(
                                    //     Val::Px(0.75 * current_window.width()),
                                    //     Val::Percent(90.0),
                                    // ),
                                    justify_content: JustifyContent::FlexEnd,
                                    flex_direction: FlexDirection::ColumnReverse,
                                    flex_wrap: FlexWrap::Wrap,
                                    align_items: AlignItems::FlexEnd,
                                    max_size: Size::UNDEFINED,
                                    ..Default::default()
                                },
                                background_color: transparent_col.into(),
                                ..Default::default()
                            },
                            ScrollingList::default(),
                            AccessibilityNode(NodeBuilder::new(Role::List)),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    TextBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Px(0.75 * current_window.width()),
                                                Val::Percent(100.0),
                                            ),
                                            flex_wrap: FlexWrap::Wrap,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    AccessibilityNode(NodeBuilder::new(Role::ListItem)),
                                ))
                                .insert(LogsArea);
                        });
                });
        });
}

// TODO: add npc argument, and push npc info into motd.
fn print_motd(sys: &mut System, should_refresh: bool) -> String {
    if should_refresh {
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_system();
        sys.refresh_users_list();
    }

    let mut res = String::from("Welcome to Android Console\n");
    res.push_str("--------------------------\n");

    if let Some(user) = sys.users().last() {
        res.push_str(&format!("Username: {}\n\n", user.name()));
    }

    res.push_str(&format!(
        "System name:             {:?}\n",
        sys.name().get_or_insert("Random system".to_string())
    ));
    res.push_str(&format!(
        "System kernel version:   {:?}\n",
        sys.kernel_version()
            .get_or_insert("Kernel alpha".to_string())
    ));
    res.push_str(&format!(
        "System OS version:       {:?}\n",
        sys.os_version().get_or_insert("1.0".to_string())
    ));
    res.push_str(&format!(
        "System host name:        {:?}\n\n",
        sys.host_name().get_or_insert("localhost".to_string())
    ));

    res.push_str(&format!(
        "Processors: {} at {:.2}GHz\n",
        sys.processors().len(),
        sys.processors()[0].frequency() as f64 / 1000.0
    ));

    res.push_str(&format!(
        "RAM: {} Gb\n",
        display_bar(
            60,
            sys.used_memory() as f64 / 1000000.0,
            sys.total_memory() as f64 / 1000000.0
        )
    ));

    res
}

fn display_bar(width: usize, value: f64, total_value: f64) -> String {
    let percent = value / total_value;
    let nb_full_tiles = (percent * (width - 2) as f64) as usize;
    let rest_tiles = width - 2 - nb_full_tiles;

    let mut res = String::from("[");
    res.push_str(&String::from("=").repeat(nb_full_tiles));
    res.push_str(&String::from(" ").repeat(rest_tiles));
    res.push_str(&format!("] {:.2}/{:.2}", value, total_value));

    res
}

// TODO: Make a link with this system and open_npc_console, that game should know which npc this console is interact with.
// Maybe CurrentInteractingNPC component will be need in PlayerBundle?
pub fn interact_with_npc(
    entity_map: Res<EntityMap>,
    mut player: Query<(&mut CurrentInteractingNPC, &Facing, &Coordinate), With<Player>>,
    npc: Query<Entity, With<NPC>>,
    input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // TODO: under code should be separate systems. Maybe `fn find_entities_in_range`, and add entities in player range into player entitiy's children entities.
    for (mut interacting_npc, facing, coordinate) in player.iter_mut() {
        let (range_x, range_y) = match facing.direction {
            FaceDirection::Down => (
                (coordinate.min_x..=coordinate.max_x),
                (coordinate.min_y - 1..=coordinate.min_y - 1), // Maybe have to range in min_y-1..=min_y, because of very small & adjoined objects
            ),
            FaceDirection::Left => (
                (coordinate.min_x - 1..=coordinate.min_x - 1),
                (coordinate.min_y..=coordinate.max_y),
            ),
            FaceDirection::Right => (
                (coordinate.max_x + 1..=coordinate.max_x + 1),
                (coordinate.min_y..=coordinate.max_y),
            ),
            FaceDirection::Up => (
                (coordinate.min_x..=coordinate.max_x),
                (coordinate.max_y + 1..=coordinate.max_y + 1),
            ),
        };

        // TODO: need to be refactored.
        for x in range_x {
            for y in range_y.clone() {
                if let Some(hit_range) = entity_map.get((x, y)) {
                    for npc_entity in hit_range {
                        if npc.contains(*npc_entity) {
                            if input.just_pressed(KeyCode::E) && app_state.0 == AppState::MainGame {
                                next_state.set(AppState::ConsoleOpenedState);
                                interacting_npc.0 = Some(*npc_entity);

                                #[cfg(debug_assertions)]
                                info!("Console opened {:?}", app_state);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    if (app_state.0 == AppState::ConsoleOpenedState) && input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainGame);
        #[cfg(debug_assertions)]
        info!("Console closed {:?}", app_state);
    }
}

pub fn open_npc_console(
    mut interacting_npc_query: Query<&mut CurrentInteractingNPC, With<Player>>,
    mut anim_data: ResMut<ConsoleAnimation>,
    mut data_query: Query<(&Parent, &mut ConsoleData)>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    info!("Opening console");
    let Ok(current_window) = window.get_single() else {
        return;
    };
    for (parent, mut data) in data_query.iter_mut() {
        // TODO: Maybe checking parent entity of ConsoleData is not necessary.
        if interacting_npc_query
            .get_single()
            .is_ok_and(|x| x.0.is_some_and(|y| y.eq(parent)))
        {
            data.is_opening = true;

            anim_data.start_position = Vec2::new(0.0, current_window.height());
            anim_data.end_position =
                Vec2::new(0.0, (1. - CONSOLE_HEIGHT) * current_window.height());
            anim_data.start_time = time.elapsed_seconds_f64();
            break;
        }
    }
}

pub fn close_npc_console(
    mut anim_data: ResMut<ConsoleAnimation>,
    mut interacting_npc_query: Query<&mut CurrentInteractingNPC, With<Player>>,
    npc_query: Query<&Children, With<NPC>>,
    mut data_query: Query<&mut ConsoleData>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    info!("Closing console");
    let Ok(current_window) = window.get_single() else {
        return;
    };

    let Ok(mut interacting_npc) = interacting_npc_query.get_single_mut() else {
        return;
    };

    if let Some(mut interacting_npc_entity) = interacting_npc.0 {
        if let Ok(npc_children) = npc_query.get(interacting_npc_entity) {
            for child in npc_children.iter() {
                if let Ok(mut child_data) = data_query.get_mut(*child) {
                    child_data.fully_opened = false;
                    child_data.is_opening = false;

                    anim_data.end_position = Vec2::new(0.0, current_window.height());
                    anim_data.start_position =
                        Vec2::new(0.0, (1. - CONSOLE_HEIGHT) * current_window.height());
                    anim_data.start_time = time.elapsed_seconds_f64();
                    interacting_npc.0 = None;
                    break;
                }
            }
        }
    }
}

pub fn apply_animation(
    mut console_query: Query<(&ConsoleUI, &mut Style)>,
    anim_data: Res<ConsoleAnimation>,
    mut data_query: Query<&mut ConsoleData>,
    time: Res<Time>,
) {
    let delta_t = time.elapsed_seconds_f64() - anim_data.start_time;
    let value = 1.0 - (-(delta_t * anim_data.moving_speed)).exp();
    let new_position = anim_data
        .start_position
        .lerp(anim_data.end_position, value as f32);

    for mut data in data_query.iter_mut() {
        if data.is_opening && new_position.abs_diff_eq(anim_data.end_position, 1.0) {
            data.fully_opened = true;
        }
    }

    if let Ok((_, mut style)) = console_query.get_single_mut() {
        style.position.top = Val::Px(new_position.y);
        style.position.left = Val::Px(new_position.x);
    }
}

// shows logs that stored in `ConsoleData.messages` to console.
pub fn update_logs_area(
    interacting_npc_query: Query<&CurrentInteractingNPC, With<Player>>,
    npc_query: Query<&Children, With<NPC>>,
    mut data_query: Query<&mut ConsoleData>,
    asset_server: Res<AssetServer>,
    mut logs_area_query: Query<&mut Text, With<LogsArea>>,
) {
    let Ok(CurrentInteractingNPC(interacting_npc)) = interacting_npc_query.get_single() else {
        return;
    };

    if let Some(interacting_npc_entity) = interacting_npc {
        let children_entity = npc_query.get(*interacting_npc_entity).unwrap();
        for &child in children_entity.iter() {
            // TODO: data not need to be mutable.
            if let Ok(mut data) = data_query.get_mut(child) {
                let sections = data
                    .messages
                    .iter()
                    .flat_map(|msg| {
                        let mut msg = msg.clone();
                        msg.push('\n');

                        IntoIterator::into_iter([TextSection {
                            value: msg.clone(),
                            style: TextStyle {
                                font: asset_server.load("fonts/VT323-Regular.ttf"),
                                font_size: 16.,
                                color: Color::rgba_u8(76, 207, 76, 255),
                            },
                        }])
                    })
                    .collect::<Vec<_>>();

                let mut text = logs_area_query.single_mut();
                text.sections = sections;
            }
        }
    }
}

pub fn update_enter_command(
    mut enter_command_text: Query<&mut Text, With<CommandLineText>>,
    interacting_npc_query: Query<&CurrentInteractingNPC, With<Player>>,
    npc_query: Query<&Children, With<NPC>>,
    mut data_query: Query<&mut ConsoleData>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    // TODO: need to be refactored. functionize this checking CurrentInteractingNPC -> ConsoleData logic.
    let Ok(CurrentInteractingNPC(interacting_npc)) = interacting_npc_query.get_single() else {
        return;
    };

    if let Some(interacting_npc_entity) = interacting_npc {
        let children_entity = npc_query.get(*interacting_npc_entity).unwrap();
        for &child in children_entity.iter() {
            if let Ok(mut state) = data_query.get_mut(child) {
                let mut text = enter_command_text.single_mut();
                text.sections = vec![];

                if state.typed_command.len() > 144 {
                    let trimmed_command = state.typed_command[..144].to_string();
                    state.typed_command = trimmed_command;
                }

                let mut to_show = String::from(">  ");
                to_show.push_str(&state.typed_command);

                if (time.elapsed_seconds_f64() * 3.0) as u64 % 2 == 0 {
                    to_show.push('_');
                }

                text.sections.push(TextSection {
                    value: to_show,
                    style: TextStyle {
                        font: asset_server.load("fonts/VT323-Regular.ttf"),
                        font_size: 20.,
                        color: Color::rgba_u8(102, 255, 102, 255),
                    },
                });
            }
        }
    }
}

pub fn handle_input_keys(
    mut data_query: Query<(&Parent, &mut ConsoleData)>,
    mut evr_keys: EventReader<KeyboardInput>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ev_writer: EventWriter<EnteredConsoleCommandEvent>,
    npc_query: Query<Entity, With<NPC>>,
    // asset_server: Res<AssetServer>,
    // audio: Res<Audio>,
) {
    for (parent, mut data) in data_query.iter_mut() {
        // if the console is not open yet
        if !data.fully_opened {
            continue;
        }

        // for opened npc console
        if let Ok(npc) = npc_query.get(parent.get()) {
            for ev in evr_keys.iter() {
                if ev.state.is_pressed() {
                    // let random_key = rand::thread_rng().gen_range(1..10);
                    // audio.play(asset_server.load(format!("audio/keys/key-{}.mp3", random_key).as_str()));

                    if let Some(key_code) = ev.key_code {
                        match key_code {
                            KeyCode::Back => {
                                if !data.typed_command.is_empty() {
                                    data.typed_command.pop();
                                }
                            }
                            KeyCode::Space => data.typed_command.push(' '),
                            KeyCode::Tab => data.typed_command.push_str("  "),
                            KeyCode::Comma => data.typed_command.push(','),
                            KeyCode::Colon => data.typed_command.push(':'),
                            KeyCode::Semicolon => data.typed_command.push(';'),
                            KeyCode::Apostrophe => data.typed_command.push('\''),
                            KeyCode::At => data.typed_command.push('@'),
                            KeyCode::LBracket => data.typed_command.push('['),
                            KeyCode::RBracket => data.typed_command.push(']'),
                            KeyCode::Minus | KeyCode::NumpadSubtract => {
                                data.typed_command.push('-')
                            }
                            KeyCode::Period | KeyCode::NumpadDecimal => {
                                data.typed_command.push('.')
                            }
                            KeyCode::Asterisk | KeyCode::NumpadMultiply => {
                                data.typed_command.push('*')
                            }
                            KeyCode::Slash | KeyCode::NumpadDivide => data.typed_command.push('/'),
                            KeyCode::Plus | KeyCode::NumpadAdd => data.typed_command.push('+'),
                            KeyCode::Key0 | KeyCode::Numpad0 => data.typed_command.push('0'),
                            KeyCode::Key1 | KeyCode::Numpad1 => data.typed_command.push('1'),
                            KeyCode::Key2 | KeyCode::Numpad2 => data.typed_command.push('2'),
                            KeyCode::Key3 | KeyCode::Numpad3 => data.typed_command.push('3'),
                            KeyCode::Key4 | KeyCode::Numpad4 => data.typed_command.push('4'),
                            KeyCode::Key5 | KeyCode::Numpad5 => data.typed_command.push('5'),
                            KeyCode::Key6 | KeyCode::Numpad6 => data.typed_command.push('6'),
                            KeyCode::Key7 | KeyCode::Numpad7 => data.typed_command.push('7'),
                            KeyCode::Key8 | KeyCode::Numpad8 => data.typed_command.push('8'),
                            KeyCode::Key9 | KeyCode::Numpad9 => data.typed_command.push('9'),

                            KeyCode::LShift
                            | KeyCode::RShift
                            | KeyCode::Escape
                            | KeyCode::LAlt
                            | KeyCode::RAlt
                            | KeyCode::LControl
                            | KeyCode::RControl
                            | KeyCode::F1
                            | KeyCode::Up
                            | KeyCode::Down
                            | KeyCode::Right
                            | KeyCode::Left
                            | KeyCode::F2
                            | KeyCode::F3
                            | KeyCode::F4
                            | KeyCode::F5
                            | KeyCode::F6
                            | KeyCode::F7
                            | KeyCode::F8
                            | KeyCode::F9
                            | KeyCode::F10
                            | KeyCode::F11
                            | KeyCode::F12
                            | KeyCode::Insert
                            | KeyCode::Delete
                            | KeyCode::Grave
                            | KeyCode::Backslash => {}

                            KeyCode::Return => {
                                // sending the command

                                ev_writer.send(EnteredConsoleCommandEvent {
                                    npc,
                                    message: data.typed_command.clone(),
                                });
                                // clearing the input
                                data.typed_command.clear();
                            }
                            _ => {
                                let key_code_str = if keyboard_input.pressed(KeyCode::LShift)
                                    || keyboard_input.pressed(KeyCode::RShift)
                                {
                                    format!("{:?}", key_code).to_uppercase()
                                } else {
                                    format!("{:?}", key_code).to_lowercase()
                                };

                                trace!("Pressed key: {:?}", key_code_str);
                                data.typed_command.push_str(&key_code_str);
                            }
                        }

                        info!("Console Typing: {:?}", data.typed_command);
                    }
                }
            }
        }
    }
}

// TODO: modify this code, [`EnteredConsoleCommandEvent`], to send command input to the server (that interacts with chatGPT)
// and returns the output as form of [`PrintConsoleEvent`], and display this to the console.
pub fn commands_handler(
    mut cmd_reader: EventReader<EnteredConsoleCommandEvent>,
    mut console_writer: EventWriter<PrintConsoleEvent>,
    npc_query: Query<&Children, With<NPC>>,
    mut data_query: Query<&mut ConsoleData>,
) {
    for EnteredConsoleCommandEvent { npc, message: cmd } in cmd_reader.iter() {
        // Don't do anything if the string is empty
        if cmd.is_empty() {
            return;
        }

        let args: Vec<&str> = cmd.trim().split(' ').collect();

        if args[0] != "clear" {
            // first send what the user typed
            let mut user_input = String::from("> ");
            user_input.push_str(cmd.clone().trim());
            console_writer.send(PrintConsoleEvent {
                npc: *npc,
                message: user_input,
            });
        }

        if let Ok(children) = npc_query.get(*npc) {
            for &child in children.iter() {
                if let Ok(mut data) = data_query.get_mut(child) {
                    match args[0] {
                        "clear" => data.messages.clear(),
                        "help" => console_writer.send(PrintConsoleEvent {
                            npc: *npc,
                            message: display_help(),
                        }),
                        "motd" => console_writer.send(PrintConsoleEvent {
                            npc: *npc,
                            message: print_motd(&mut System::new(), true),
                        }),

                        _ => {
                            console_writer.send(PrintConsoleEvent {
                                npc: *npc,
                                message: format!(
                                    "I didn't understand the command: \"{}\"",
                                    args[0]
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
}

fn display_help() -> String {
    let mut res = String::from("\nSHOWING AVAILABLE COMMANDS\n");

    let underline = "==========================\n\n";
    res.push_str(underline);

    res.push_str("- help : Displays this message\n");
    res.push_str("- clear : Clears commands on the screen\n");
    res.push_str("- motd : Prints informations about YOUR computer\n");

    res
}

// TODO
pub fn should_run_cmd_handler() -> bool {
    true
}

pub fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.iter() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;

            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(0., max_scroll);
            style.position.top = Val::Px(scrolling_list.position);
        }
    }
}
