use bevy::{
    prelude::*, ui::RelativeCursorPosition,
};

use bevy_nine_slice_ui::{NineSliceUiMaterialBundle, NineSliceUiTexture};
use leafwing_input_manager::{
    action_state::{ActionState, ActionStateDriver},
    axislike::SingleAxis,
    input_map::InputMap,
    plugin::ToggleActions,
    user_input::InputKind,
    InputManagerBundle,
};

use crate::{
    menu_ui::UIAnchorNode,
    playerinput::CreditsUIActions,
    ui_helpers::{px, ProjectLocalStyle, UICommandsExt, ALL, LARGE, MEDIUM}, UIFocus,
};

pub struct CreditsPlugin;
impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UIFocus::Credits), instantiate_credits_ui)
            .add_systems(OnExit(UIFocus::Credits), teardown_credits)
            .add_systems(
                Update,
                (
                    exit_credits_ui,
                    scroll_node_with_wheel,
                    scroll_node_with_drag,
                )
                    .run_if(in_state(UIFocus::Credits)),
            );
    }
}
#[derive(Component)]
struct CreditsRoot;

#[derive(Component)]
struct ScrollableElement {
    mask: Entity,
    content_container: Entity,
    scroll_bar_element: Entity,
    scroll_bar_handle: Entity,
    scroll_value: f32,
}
impl ScrollableElement {
    fn new(
        mask: Entity,
        content_container: Entity,
        scroll_bar_element: Entity,
        scroll_bar_handle: Entity,
    ) -> Self {
        Self {
            mask,
            content_container,
            scroll_bar_element,
            scroll_bar_handle,
            scroll_value: 0.0,
        }
    }
}

#[derive(Component)]
struct PixelsPerScroll(f32);

fn instantiate_credits_ui(
    mut commands: Commands,
    mut settings_menu_actions: ResMut<ToggleActions<CreditsUIActions>>,
    asset_server: Res<AssetServer>,
    anchor: Res<UIAnchorNode>,
) {
    let thick_bar = asset_server.load("nine_slice/thick_credits_bar.png");
    let thin_bar = asset_server.load("nine_slice/thin_credits_bar.png");
    settings_menu_actions.enabled = true;
    let credits_layout_container = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: Style {
                    width: px(525.),
                    height: px(367.),
                    position_type: PositionType::Absolute,
                    padding: UiRect::all(px(8.)),
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/settings_tile.png"),
                ),
                ..default()
            },
            CreditsRoot,
            Name::new("Credits layout"),
        ))
        .id();
    let credits_window = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: px(503.),
                    height: px(353.),
                    overflow: Overflow::clip(),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Name::new("Credits mask"),
        ))
        .id();
    let credits_container = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Grid,
                grid_template_columns: GridTrack::fr(1.),
                ..default()
            },
            ..default()
        })
        .id();
    let scroll_bar = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: px(8.),
                    height: px(353.),
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            Interaction::None,
            RelativeCursorPosition::default(),
        ))
        .id();
    let scroll_bar_handle = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: px(8.),
                    height: px(15.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::BLACK.into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            Name::new("Scroll Bar Handle"),
        ))
        .id();

    let exit_button = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: Style {
                    width: px(40.),
                    height: px(20.),
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    justify_self: JustifySelf::Center,
                    padding: UiRect::bottom(px(5.)),
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/settings_menu_exit_button.png"),
                ),
                ..default()
            },
            Interaction::None,
        ))
        .insert(ActionStateDriver {
            action: CreditsUIActions::ExitCredits,
            targets: credits_layout_container.into(),
        })
        .id();

    let exit_button_label = commands
        .make_text("Exit", TextStyle::local(MEDIUM, Color::WHITE))
        .id();

    commands
        .entity(anchor.0)
        .add_child(credits_layout_container);
    commands
        .entity(credits_layout_container)
        .push_children(&[credits_window, scroll_bar]);
    commands.entity(credits_window).add_child(credits_container);
    commands.entity(scroll_bar).add_child(scroll_bar_handle);
    for entry in get_entries().iter() {
        let child = entry.make_entry(&mut commands, thick_bar.clone(), thin_bar.clone());
        commands.entity(credits_container).add_child(child);
    }
    commands.entity(credits_container).add_child(exit_button);
    commands.entity(exit_button).add_child(exit_button_label);

    commands
        .entity(credits_layout_container)
        .insert(InputManagerBundle::<CreditsUIActions> {
            input_map: InputMap::default()
                .insert(
                    InputKind::SingleAxis(SingleAxis::mouse_wheel_y()),
                    CreditsUIActions::ScrollCredits,
                )
                .insert(KeyCode::Escape, CreditsUIActions::ExitCredits)
                .build(),
            ..default()
        })
        .insert(ScrollableElement::new(
            credits_window,
            credits_container,
            scroll_bar,
            scroll_bar_handle,
        ))
        .insert(PixelsPerScroll(100.));
}
fn teardown_credits(
    q: Query<Entity, With<CreditsRoot>>,
    mut commands: Commands,
    mut settings_menu_actions: ResMut<ToggleActions<CreditsUIActions>>,
) {
    settings_menu_actions.enabled = false;
    q.iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}

fn scroll_node_with_wheel(
    mut q: Query<(
        &mut ScrollableElement,
        &ActionState<CreditsUIActions>,
        &PixelsPerScroll,
    )>,
    mut nodes_and_styles: Query<(&Node, &mut Style), Without<ScrollableElement>>,
) {
    q.iter_mut()
        .for_each(|(mut scroll_element, action_state, pps)| {
            if action_state.just_pressed(CreditsUIActions::ScrollCredits) {
                let state = action_state.value(CreditsUIActions::ScrollCredits);

                if let Ok([(window, _), (content, mut content_style), (handle, mut handle_style)]) =
                    nodes_and_styles.get_many_mut([
                        scroll_element.mask,
                        scroll_element.content_container,
                        scroll_element.scroll_bar_handle,
                    ])
                {
                    let max_offset = (content.size().y - window.size().y).max(0.);
                    let handle_max_offset = (window.size().y - handle.size().y).max(0.);

                    scroll_element.scroll_value += pps.0 * state.signum();
                    scroll_element.scroll_value =
                        scroll_element.scroll_value.clamp(-max_offset, 0.0);
                    let handle_scroll =
                        (scroll_element.scroll_value.abs() / max_offset) * handle_max_offset;

                    content_style.top = px(scroll_element.scroll_value);
                    handle_style.top = px(handle_scroll);
                }
            }
        })
}

fn scroll_node_with_drag(
    mut q: Query<&mut ScrollableElement>,
    mut nodes_and_styles: Query<(&Node, &mut Style), Without<ScrollableElement>>,
    interaction: Query<(&Interaction, &RelativeCursorPosition)>,
) {
    q.iter_mut().for_each(|mut scroll_element| {
        if let Ok((int, pos)) = interaction.get(scroll_element.scroll_bar_element) {
            match int {
                Interaction::Pressed => {
                    if let Ok(
                        [(window, _), (content, mut content_style), (handle, mut handle_style)],
                    ) = nodes_and_styles.get_many_mut([
                        scroll_element.mask,
                        scroll_element.content_container,
                        scroll_element.scroll_bar_handle,
                    ]) {
                        let max_offset = (content.size().y - window.size().y).max(0.);
                        let handle_max_offset = (window.size().y - handle.size().y).max(0.);
                        if let Some(position) = pos.normalized {
                            scroll_element.scroll_value = -1.0 * (max_offset * position.y);
                            scroll_element.scroll_value =
                                scroll_element.scroll_value.clamp(-max_offset, 0.0);

                            let handle_scroll =
                                (handle_max_offset * position.y).clamp(0., handle_max_offset);
                            content_style.top = px(scroll_element.scroll_value);
                            handle_style.top = px(handle_scroll);
                        }
                    }
                }
                _ => {}
            }
        }
    })
}

fn exit_credits_ui(
    q: Query<&ActionState<CreditsUIActions>>,
    mut next_state: ResMut<NextState<UIFocus>>,
) {
    for n in q.iter() {
        if n.just_pressed(CreditsUIActions::ExitCredits) {
            next_state.set(UIFocus::MainMenu);
        }
    }
}

fn section_header(text: impl Into<String>) -> Text {
    Text::from_section(text, TextStyle::local(LARGE, Color::BLACK))
}
fn section_content(text: impl Into<String>) -> Text {
    Text::from_section(text, TextStyle::local(MEDIUM, Color::WHITE))
}

struct CreditEntry {
    title: &'static str,
    content: &'static str,
}
impl CreditEntry {
    fn new(title: &'static str, content: &'static str) -> Self {
        Self { title, content }
    }
    fn make_entry(
        &self,
        commands: &mut Commands,
        thick_bar: Handle<Image>,
        thin_bar: Handle<Image>,
    ) -> Entity {
        let credit_entry_layout = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    margin: UiRect::all(px(2.)),
                    ..default()
                },
                ..default()
            })
            .id();
        let header_layout = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ..default()
            })
            .id();
        let header = commands
            .spawn(TextBundle {
                text: section_header(self.title),
                style: Style {
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ..default()
            })
            .id();
        let header_underline = commands
            .spawn(NineSliceUiMaterialBundle {
                style: Style {
                    height: px(10.),
                    width: ALL,
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(thick_bar),
                ..default()
            })
            .id();
        commands
            .entity(header_layout)
            .push_children(&[header, header_underline]);
        let section_layout = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    min_width: px(140.),
                    ..default()
                },
                ..default()
            })
            .id();
        let section = commands
            .spawn(TextBundle {
                text: section_content(self.content),
                style: Style {
                    align_self: AlignSelf::Center,
                    margin: UiRect::vertical(px(3.)),
                    ..default()
                },
                ..default()
            })
            .id();
        let section_underline = commands
            .spawn(NineSliceUiMaterialBundle {
                style: Style {
                    height: px(5.),
                    width: ALL,
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(thin_bar),
                ..default()
            })
            .id();
        commands
            .entity(section_layout)
            .push_children(&[section, section_underline]);
        commands
            .entity(credit_entry_layout)
            .push_children(&[header_layout, section_layout]);

        credit_entry_layout
    }
}

fn get_entries() -> Vec<CreditEntry> {
    vec![
        CreditEntry::new(
            "
Sound
",
            "
Stephen Ancona   Justin Johnson
 
",
        ),
        CreditEntry::new(
            "
Music
",
            "
\"Limit 70\" -  Kevin MacLeod (incompetech.com)
Licensed under Creative Commons: By Attribution 4.0 License
http://creativecommons.org/licenses/by/4.0/
 
",
        ),
        CreditEntry::new(
            "
Special Thanks
",
            "
Adrienne Pasta    Stephen Ancona    Jeff Crowl
A. Kerr

The wonderful humans over at the Bevy Discord Server
 
",
        ),
        CreditEntry::new(
            "
The Whole Rest of It
",
            "
Justin Johnson
 
",
        ),
    ]
}
