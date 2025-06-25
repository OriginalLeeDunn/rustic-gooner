use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::keyboard::{KeyCode, KeyboardInput};
use reqwest::blocking::get;
use bevy::app::AppExit;


fn fetch_from_server() {
    let response = get("http://localhost:8000").unwrap().text().unwrap();
    println!("Server says: {}", response);
}

#[derive(Component)]
struct MenuUI;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    MainMenu,
    InGame,
    Settings,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>() // ✅ Bevy 0.13 uses `add_state_machine`
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        .add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(OnExit(AppState::InGame), cleanup_game)
        .add_systems(Update, (
            menu_action_system.run_if(in_state(AppState::MainMenu)),
            fetch_from_server.run_if(in_state(AppState::MainMenu)),
            main_menu_controls.run_if(in_state(AppState::MainMenu)),
            menu_keyboard_system.run_if(in_state(AppState::MainMenu)),
        ))
        .run();
}

fn setup_main_menu(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // UI camera
    commands.spawn(Camera2dBundle::default());

    // Menu UI
    commands.spawn((NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    }, MenuUI))
    .with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                column_gap: Val::Px(20.0),
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            for label in &["Play", "Quit", "Settings"] {
                parent.spawn((ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                }, MenuUI))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        *label,
                        TextStyle {
                            font: Default::default(),
                            font_size: 40.0,
                            color: Color::RED,
                        },
                    ));
                });
            }
        });
    });
}

fn menu_action_system(
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                let button_text = &text.sections[0].value;
                if button_text == "Play" {
                    next_state.set(AppState::InGame);
                } else if button_text == "Quit" {
                    exit.send(AppExit);
                } else if button_text == "Settings" {
                    next_state.set(AppState::Settings);
                }
            }
            Interaction::Hovered => {
                text.sections[0].style.color = Color::YELLOW;
            }
            Interaction::None => {
                text.sections[0].style.color = Color::RED;
            }
        }
    }
}

fn menu_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for ent in query.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

fn setup_game() {
    println!("Game setup");
}

fn cleanup_game() {
    println!("Game cleanup");
}

fn settings_menu() {
    println!("Settings menu");
}

fn cleanup_settings_menu() {
    println!("Settings menu cleanup");
}

fn main_menu_controls(
    mut next_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::InGame);
    }
}

