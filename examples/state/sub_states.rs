//! This example illustrates the use of [`SubStates`] for more complex state handling patterns.
//!
//! [`SubStates`] are [`States`] that only exist while the App is in another [`State`]. They can
//! be used to create more complex patterns while relying on simple enums, or to de-couple certain
//! elements of complex state objects.
//!
//! In this case, we're transitioning from a `Menu` state to an `InGame` state, at which point we create
//! a substate called `IsPaused` to track whether the game is paused or not.

use bevy::{dev_tools::states::*, prelude::*};

use ui::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

// In this case, instead of deriving `States`, we derive `SubStates`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
// And we need to add an attribute to let us know what the source state is
// and what value it needs to have. This will ensure that unless we're
// in [`AppState::InGame`], the [`IsPaused`] state resource
// will not exist.
#[source(AppState = AppState::InGame)]
#[states(scoped_entities)]
enum IsPaused {
    #[default]
    Running,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_sub_state::<IsPaused>() // We set the substate up here.
        // Most of these remain the same
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(OnEnter(IsPaused::Paused), setup_paused_screen)
        .add_systems(
            Update,
            (
                // Instead of relying on [`AppState::InGame`] here, we're relying on
                // [`IsPaused::Running`], since we don't want movement or color changes
                // if we're paused
                (movement, change_color).run_if(in_state(IsPaused::Running)),
                // The pause toggle, on the other hand, needs to work whether we're
                // paused or not, so it uses [`AppState::InGame`] instead.
                toggle_pause.run_if(in_state(AppState::InGame)),
            ),
        )
        .add_systems(Update, log_transitions::<AppState>)
        .run();
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn();
}

const SPEED: f32 = 100.0;
fn movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Sprite>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        if input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }

        if direction != Vec3::ZERO {
            transform.translation += direction.normalize() * SPEED * time.delta_secs();
        }
    }
}

fn change_color(time: Res<Time>, mut query: Query<&mut Sprite>) {
    for mut sprite in &mut query {
        let new_color = LinearRgba {
            blue: ops::sin(time.elapsed_secs() * 0.5) + 2.0,
            ..LinearRgba::from(sprite.color)
        };

        sprite.color = new_color.into();
    }
}

fn toggle_pause(
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<IsPaused>>,
    mut next_state: ResMut<NextState<IsPaused>>,
) {
    if input.just_pressed(KeyCode::Space) {
        next_state.set(match current_state.get() {
            IsPaused::Running => IsPaused::Paused,
            IsPaused::Paused => IsPaused::Running,
        });
    }
}

mod ui {
    use crate::*;

    #[derive(Resource)]
    pub struct MenuData {
        pub button_entity: Entity,
    }

    pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    pub fn setup(mut commands: Commands) {
        commands.spawn(Camera2d);
    }

    pub fn setup_menu(mut commands: Commands) {
        let button_entity = commands
            .spawn((
                Node {
                    // center button
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    Button,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    children![(
                        Text::new("Play"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    )]
                )],
            ))
            .id();
        commands.insert_resource(MenuData { button_entity });
    }

    pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.spawn(Sprite::from_image(asset_server.load("branding/icon.png")));
    }

    pub fn setup_paused_screen(mut commands: Commands) {
        commands.spawn((
            DespawnOnExitState(IsPaused::Paused),
            Node {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.),
                ..default()
            },
            children![(
                Node {
                    width: Val::Px(400.),
                    height: Val::Px(400.),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                children![(
                    Text::new("Paused"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                )]
            )],
        ));
    }
}
