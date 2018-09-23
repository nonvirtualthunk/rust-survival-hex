use gui::*;
use game::entities::actions::*;
use common::color::Color;
use common::prelude::*;
use game::Entity;
use std::collections::HashMap;
use state::GameState;
use state::ControlContext;
use control_events::TacticalEvents;
use std::time::Duration;
use std::time::Instant;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Message {
    text : String,
    color : Color,
    duration : Duration,
    started_at : Instant
}
impl Message {
    pub fn new <S : Into<String>> (text : S) -> Message {
        Message {
            text : text.into(),
            color : Color::black(),
            duration : Duration::from_secs(6),
            started_at : Instant::now()
        }
    }
}

#[derive(Default)]
pub struct MessagesDisplay {
    messages_list : ListWidget<MessageWidget>,
    messages : VecDeque<Message>,
}
impl DelegateToWidget for MessagesDisplay {
    fn as_widget(&mut self) -> &mut Widget { self.messages_list.as_widget() }

    fn as_widget_immut(&self) -> &Widget { self.messages_list.as_widget_immut() }
}

#[derive(WidgetContainer)]
struct MessageWidget {
    pub background : Widget,
    pub text : Widget,
}

impl Default for MessageWidget {
    fn default() -> Self {
        let background = Widget::window(Color::greyscale(0.7), 2)
            .surround_children()
            .margin(2.px());
        let text = Widget::text("Message", FontSize::HeadingMajor).parent(&background);
        MessageWidget { background, text }
    }
}

impl MessagesDisplay {
    pub fn new(gui : &mut GUI, parent : &Widget) -> MessagesDisplay {
        let mut messages_list = ListWidget::featherweight()
            .alignment(Alignment::Top, Alignment::Left)
            .x(Positioning::centered())
            .y(Positioning::constant(2.ux()))
            .size(Sizing::surround_children(), Sizing::surround_children())
            .named("messages display list")
            .only_consume(EventConsumption::none())
            .parent(parent)
            .apply(gui);

        messages_list.item_archetype.set_size(Sizing::surround_children(), Sizing::surround_children());

        MessagesDisplay {
            messages : VecDeque::new(),
            messages_list
        }
    }

    pub fn add_message(&mut self, message : Message) {
        self.messages.push_front(message);
    }

    pub fn update(&mut self, gui : &mut GUI) {
        self.messages.retain(|m| Instant::now().duration_since(m.started_at) < m.duration);

        self.messages_list.update(gui, &self.messages.iter().collect_vec(), |widget, message| {
            let base_color = message.color.clone();
            let pcnt_done = Instant::now().duration_since(message.started_at).to_millis() / message.duration.to_millis();
            let alpha = if pcnt_done > 0.7 {
                1.0 - (pcnt_done - 0.7) / 0.3
            } else {
                1.0
            };

            widget.text.set_text(message.text.clone()).set_color(base_color.with_a(alpha as f32));
            widget.background.set_color(Color::new(0.7,0.7,0.7,alpha as f32)).set_border_color(Color::new(0.0,0.0,0.0,alpha as f32));

        });
    }
}