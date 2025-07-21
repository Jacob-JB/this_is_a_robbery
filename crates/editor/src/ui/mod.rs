use bevy::{platform::collections::HashMap, prelude::*};
use bevy_egui::{EguiContext, EguiPostUpdateSet, egui::Ui};
use egui_dock::{DockArea, DockState};

pub mod bevy_inspector;
pub mod element_selection;

pub fn build(app: &mut App) {
    app.add_plugins(bevy_egui::EguiPlugin::default());
    app.add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin);

    app.init_resource::<UiDockState>();

    bevy_inspector::build(app);
    element_selection::build(app);

    app.add_systems(PostUpdate, show_ui.before(EguiPostUpdateSet::ProcessOutput));
}

#[derive(Resource)]
struct UiDockState(DockState<BoxedTab>);

type BoxedTab = Box<dyn Tab>;

pub trait Tab: Send + Sync {
    fn title(&self) -> &'static str;

    fn ui(&mut self, world: &mut World, ui: &mut Ui);

    fn clear_background(&self) -> bool {
        true
    }

    fn closeable(&self) -> bool {
        true
    }
}

impl FromWorld for UiDockState {
    fn from_world(world: &mut World) -> Self {
        let mut dock_state: DockState<BoxedTab> = DockState::new(vec![Box::new(
            element_selection::ElementSelectionTab::from_world(world),
        )]);

        UiDockState(dock_state)
    }
}

/// responsible for calling the methods on tabs using dynamic dispatch to draw windows
struct TabViewer<'a> {
    world: &'a mut World,
    seen_names: HashMap<&'static str, u16>,
}

impl<'a> TabViewer<'a> {
    fn new(world: &'a mut World) -> Self {
        TabViewer {
            world,
            seen_names: HashMap::new(),
        }
    }
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = BoxedTab;

    fn ui(&mut self, ui: &mut Ui, window: &mut Self::Tab) {
        window.ui(self.world, ui);
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        let title = window.title();

        let count = if let Some(count) = self.seen_names.get_mut(&title) {
            *count += 1;
            *count
        } else {
            self.seen_names.insert(title, 0);
            1
        };

        format!("{} ({})", title, count).into()
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        tab.clear_background()
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        tab.closeable()
    }
}

fn show_ui(
    world: &mut World,

    mut context_q: Local<QueryState<&mut EguiContext, With<bevy::window::PrimaryWindow>>>,
) {
    let Ok(context) = context_q.single(world) else {
        return;
    };
    let mut context = context.clone();
    let context = context.get_mut();

    world.resource_scope(|world: &mut World, mut dock_state: Mut<UiDockState>| {
        DockArea::new(&mut dock_state.0)
            .style(egui_dock::Style::from_egui(context.style().as_ref()))
            .show(context, &mut TabViewer::new(world));
    });
}
