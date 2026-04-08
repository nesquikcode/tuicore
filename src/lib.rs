pub mod core;
pub mod base;
pub mod elements;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::base::{RelSize, Sizes};

    #[test]
    fn test() {
        use std::cell::Cell;
        use std::time::{Instant};
        use std::collections::{HashMap};
        use crate::base::{Color, Positions, RelPosition};
        use crate::core::{App, AppEvent, EventType};
        use crate::elements::{Label, ComposedLayer, Rectangle, Image};
        let mut app = App::init(60, Some(16));
        let fpsLabel = Label::new(
            "Идёт подсчёт...".to_string(),
            Positions::RelPosition(
                RelPosition { relx: 50, rely: 45 }
            ),
            None, None, None,
            None, None
        );
        let author = Label::new(
            "Максимов Василий 9 \"б\" класс".to_string(),
            Positions::RelPosition(
                RelPosition { relx: 50, rely: 30 }
            ),
            None, None, None,
            None, None
        );
        let part = Label::new(
            "Практическая часть проекта".to_string(),
            Positions::RelPosition(
                RelPosition { relx: 50, rely: 35 }
            ),
            None, None, None,
            None, None
        );
        let lang = Label::new(
            "Язык: Rust".to_string(),
            Positions::RelPosition(
                RelPosition { relx: 50, rely: 40 }
            ),
            None, None, None,
            None, None
        );


        let img = Image::new(
            Positions::RelPosition(
                RelPosition { relx: 5, rely: 10 }
            ),
            Sizes::RelSize(
                RelSize { relwidth: 40, relheight: 80 }
            ),
            "D:\\tuicore\\logo.jpg".to_string(),
            None, None
        );

        let redid: u32 = app.register_renderer(Box::new(img));
        let fpslid: u32 = app.register_renderer(Box::new(fpsLabel));
        let authorid: u32 = app.register_renderer(Box::new(author));
        let partid: u32 = app.register_renderer(Box::new(part));
        let langif: u32 = app.register_renderer(Box::new(lang));
        app.register_event(
            AppEvent {
                evtype: EventType::PreRender,
                exec: Box::new(move |mut renderers, mut buff, mut context| {
                    context.insert("t1", Instant::now());
                })
            }
        );
        app.register_event(
            AppEvent {
                evtype: EventType::PostRender,
                exec: Box::new(move |mut renderers, mut buff, mut context| {
                    let t2 = Instant::now();
                    let t1 = context.get::<Instant>("t1");
                    match t1 {
                        Some(t1) => {
                            if let Some(x) = renderers.get_mut(&fpslid) {
                                let x = x.as_mut();

                                if let Some(label) = x.as_any_mut().downcast_mut::<Label>() {
                                    label.change_content(format!("Кадров в секунду: {}", (1.0/t2.duration_since(*t1).as_secs_f32()).round()));
                                }
                            }
                        }
                        None => {}
                    }
                })
            }
        );
        app.run();
    }
}