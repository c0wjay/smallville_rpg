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
    components::{Coordinate, FaceDirection, Facing, MoveLock, Player, NPC},
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

#[derive(Default, Resource)]
pub struct ConsoleData {
    pub enter_command: String,
    pub is_opening: bool,
    pub fully_opened: bool,
    pub messages: Vec<String>,
}
pub struct PrintConsoleEvent(pub String);
pub struct EnteredConsoleCommandEvent(pub String);

pub fn add_message_events_to_console(
    mut data: ResMut<ConsoleData>,
    mut ev_console_message: EventReader<PrintConsoleEvent>,
) {
    for PrintConsoleEvent(message) in ev_console_message.iter() {
        data.messages.push(message.clone());
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

// TODO: Add Scroll. Resize height of command box.
pub fn build_ui(
    mut commands: Commands,
    mut anim_data: ResMut<ConsoleAnimation>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut console_writer: EventWriter<PrintConsoleEvent>,
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
    let mut sys = System::new();
    sys.refresh_all();
    console_writer.send(PrintConsoleEvent(print_motd(&mut sys, false)));
}

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

pub fn interact_with_npc(
    entity_map: Res<EntityMap>,
    player: Query<(&Facing, &Coordinate), With<Player>>,
    npc: Query<Entity, With<NPC>>,
    input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // TODO: under code should be separate systems. Maybe `fn find_entities_in_range`, and add entities in player range into player entitiy's children entities.
    for (facing, coordinate) in player.iter() {
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

        let mut flag = false;
        for x in range_x {
            if flag {
                break;
            }
            for y in range_y.clone() {
                if flag {
                    break;
                }
                if let Some(hit_range) = entity_map.get((x, y)) {
                    for npc_entity in hit_range {
                        if npc.contains(*npc_entity) {
                            flag = true;
                            break;
                        }
                    }
                }
            }
        }

        if flag && input.just_pressed(KeyCode::E) && app_state.0 == AppState::MainGame {
            next_state.set(AppState::ConsoleOpenedState);
            #[cfg(debug_assertions)]
            info!("Console opened {:?}", app_state);
        }
    }

    if (app_state.0 == AppState::ConsoleOpenedState) && input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainGame);
        #[cfg(debug_assertions)]
        info!("Console closed {:?}", app_state);
    }
}

pub fn open_console(
    mut anim_data: ResMut<ConsoleAnimation>,
    mut data: ResMut<ConsoleData>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    info!("Opening console");
    let Ok(current_window) = window.get_single() else {
        return;
    };

    data.is_opening = true;

    anim_data.start_position = Vec2::new(0.0, current_window.height());
    anim_data.end_position = Vec2::new(0.0, (1. - CONSOLE_HEIGHT) * current_window.height());
    anim_data.start_time = time.elapsed_seconds_f64();
}

pub fn close_console(
    mut anim_data: ResMut<ConsoleAnimation>,
    mut data: ResMut<ConsoleData>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    info!("Closing console");
    let Ok(current_window) = window.get_single() else {
        return;
    };

    data.fully_opened = false;
    data.is_opening = false;

    anim_data.end_position = Vec2::new(0.0, current_window.height());
    anim_data.start_position = Vec2::new(0.0, (1. - CONSOLE_HEIGHT) * current_window.height());
    anim_data.start_time = time.elapsed_seconds_f64();
}

pub fn apply_animation(
    mut console_query: Query<(&ConsoleUI, &mut Style)>,
    anim_data: Res<ConsoleAnimation>,
    mut data: ResMut<ConsoleData>,
    time: Res<Time>,
) {
    let delta_t = time.elapsed_seconds_f64() - anim_data.start_time;
    let value = 1.0 - (-(delta_t * anim_data.moving_speed)).exp();
    let new_position = anim_data
        .start_position
        .lerp(anim_data.end_position, value as f32);

    if data.is_opening && new_position.abs_diff_eq(anim_data.end_position, 1.0) {
        data.fully_opened = true;
    }

    if let Ok((_, mut style)) = console_query.get_single_mut() {
        style.position.top = Val::Px(new_position.y);
        style.position.left = Val::Px(new_position.x);
    }
}

pub fn update_logs_area(
    data: Res<ConsoleData>,
    asset_server: Res<AssetServer>,
    mut logs_area_query: Query<&mut Text, With<LogsArea>>,
) {
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

pub fn update_enter_command(
    mut enter_command_text: Query<&mut Text, With<CommandLineText>>,
    mut state: ResMut<ConsoleData>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    let mut text = enter_command_text.single_mut();
    text.sections = vec![];

    if state.enter_command.len() > 144 {
        let trimmed_command = state.enter_command[..144].to_string();
        state.enter_command = trimmed_command;
    }

    let mut to_show = String::from(">  ");
    to_show.push_str(&state.enter_command);

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

pub fn handle_input_keys(
    mut data: ResMut<ConsoleData>,
    mut evr_keys: EventReader<KeyboardInput>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ev_writer: EventWriter<EnteredConsoleCommandEvent>,
    // asset_server: Res<AssetServer>,
    // audio: Res<Audio>,
) {
    // if the console is not open yet
    if !data.fully_opened {
        return;
    }

    for ev in evr_keys.iter() {
        if ev.state.is_pressed() {
            // let random_key = rand::thread_rng().gen_range(1..10);
            // audio.play(asset_server.load(format!("audio/keys/key-{}.mp3", random_key).as_str()));

            if let Some(key_code) = ev.key_code {
                match key_code {
                    KeyCode::Back => {
                        if !data.enter_command.is_empty() {
                            data.enter_command.pop();
                        }
                    }
                    KeyCode::Space => data.enter_command.push(' '),
                    KeyCode::Tab => data.enter_command.push_str("  "),
                    KeyCode::Comma => data.enter_command.push(','),
                    KeyCode::Colon => data.enter_command.push(':'),
                    KeyCode::Semicolon => data.enter_command.push(';'),
                    KeyCode::Apostrophe => data.enter_command.push('\''),
                    KeyCode::At => data.enter_command.push('@'),
                    KeyCode::LBracket => data.enter_command.push('['),
                    KeyCode::RBracket => data.enter_command.push(']'),
                    KeyCode::Minus | KeyCode::NumpadSubtract => data.enter_command.push('-'),
                    KeyCode::Period | KeyCode::NumpadDecimal => data.enter_command.push('.'),
                    KeyCode::Asterisk | KeyCode::NumpadMultiply => data.enter_command.push('*'),
                    KeyCode::Slash | KeyCode::NumpadDivide => data.enter_command.push('/'),
                    KeyCode::Plus | KeyCode::NumpadAdd => data.enter_command.push('+'),
                    KeyCode::Key0 | KeyCode::Numpad0 => data.enter_command.push('0'),
                    KeyCode::Key1 | KeyCode::Numpad1 => data.enter_command.push('1'),
                    KeyCode::Key2 | KeyCode::Numpad2 => data.enter_command.push('2'),
                    KeyCode::Key3 | KeyCode::Numpad3 => data.enter_command.push('3'),
                    KeyCode::Key4 | KeyCode::Numpad4 => data.enter_command.push('4'),
                    KeyCode::Key5 | KeyCode::Numpad5 => data.enter_command.push('5'),
                    KeyCode::Key6 | KeyCode::Numpad6 => data.enter_command.push('6'),
                    KeyCode::Key7 | KeyCode::Numpad7 => data.enter_command.push('7'),
                    KeyCode::Key8 | KeyCode::Numpad8 => data.enter_command.push('8'),
                    KeyCode::Key9 | KeyCode::Numpad9 => data.enter_command.push('9'),

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
                        ev_writer.send(EnteredConsoleCommandEvent(data.enter_command.clone()));
                        // clearing the input
                        data.enter_command.clear();
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
                        data.enter_command.push_str(&key_code_str);
                    }
                }
            }
        }
    }
}

// TODO
pub fn commands_handler(
    mut cmd_reader: EventReader<EnteredConsoleCommandEvent>,
    mut console_writer: EventWriter<PrintConsoleEvent>,
    mut data: ResMut<ConsoleData>,
) {
    for EnteredConsoleCommandEvent(cmd) in cmd_reader.iter() {
        // Don't do anything if the string is empty
        if cmd.is_empty() {
            return;
        }

        let args: Vec<&str> = cmd.trim().split(' ').collect();

        if args[0] != "clear" {
            // first send what the user typed
            let mut user_input = String::from("> ");
            user_input.push_str(cmd.clone().trim());
            console_writer.send(PrintConsoleEvent(user_input));
        }

        match args[0] {
            "clear" => data.messages.clear(),
            "help" => console_writer.send(PrintConsoleEvent(display_help())),
            "motd" => console_writer.send(PrintConsoleEvent(print_motd(&mut System::new(), true))),

            _ => {
                console_writer.send(PrintConsoleEvent(format!(
                    "I didn't understand the command: \"{}\"",
                    args[0]
                )));
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
