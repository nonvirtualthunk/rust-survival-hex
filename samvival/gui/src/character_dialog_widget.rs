use gui::*;
use control_events::TacticalEvents;
use state::ControlContext;
use game::logic::combat::*;
use common::prelude::*;
use common::string::*;
use game::prelude::*;
use common::color::Color;
use state::GameState;

pub struct CharacterSpeechWidget {
    pub body : Widget,
    pub text : Widget,
    pub dismiss_button: Widget,
    pub character : Entity,
    pub removed : bool,
    // todo: record created at time and duration for auto-closing speech widgets in the future
}

impl DelegateToWidget for CharacterSpeechWidget {
    fn as_widget(&mut self) -> &mut Widget { self.body.as_widget() }
    fn as_widget_immut(&self) -> &Widget { self.body.as_widget_immut() }
}
impl WidgetContainer for CharacterSpeechWidget {
    fn for_each_widget<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(&mut self.body);
        (func)(&mut self.text);
        (func)(&mut self.dismiss_button);
    }
}

impl CharacterSpeechWidget {
    pub fn new(character : Entity, text : RichString, confirmation_required : bool) -> CharacterSpeechWidget {
        let body = Widget::segmented_window("ui/window/speech_dialog")
            .named("Speech dialog body")
            .width(20.ux())
            .centered()
            .surround_children_v();

        let text = Widget::wrapped_text(text.as_plain_string(), FontSize::HeadingMajor, TextWrap::WithinParent)
            .named("Speech dialog text")
            .parent(&body)
            .centered_horizontally();

        let dismiss_button = Button::image_button(strf("ui/window/checkbox_on"))
            .named("Speech dialog dismiss button")
            .parent(&body)
            .below(&text, 1.ux())
            .showing(confirmation_required)
        ;

        CharacterSpeechWidget {
            body,
            text,
            dismiss_button,
            character,
            removed : false,
        }
    }

    pub fn update(&mut self, view: &WorldView, gui : &mut GUI, game_state : &GameState, control : &mut ControlContext) {
        if ! self.removed {
            let character_pos = view.character(self.character).effective_graphical_pos();

            let screen_pos = ::piston_window::math::transform_pos(game_state.view_matrix, [character_pos.x as f64, character_pos.y as f64]);
            info!("Screen pos {:?}", screen_pos);
            self.body.set_position((screen_pos[0] * game_state.viewport.window_size[0] as f64).px(), (screen_pos[1] * game_state.viewport.window_size[1] as f64).px());

            self.reapply_all(gui);

            let mut remove = false;
            for evt in gui.events_for(&self.dismiss_button) {
                if let UIEvent::WidgetEvent { event : WidgetEvent::ButtonClicked(_), .. } = evt {
                    control.trigger_event(TacticalEvents::SpeechDialogDismissed(self.character, self.body.id()));
                    remove = true;
                }
            }
            if remove {
                self.removed = true;
                gui.remove_widget(&mut self.body);
            }
        }
    }
}