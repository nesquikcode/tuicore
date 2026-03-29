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
        let label = Label::new(
            format!(
                "{}x{}", app.buffer.size.width, app.buffer.size.height).to_string(),
            Positions::RelPosition(
                RelPosition { relx: 50, rely: 50 }
            ),
            Some(true),
            Some(Color::black()), Some(Color::white()),
            None, None
        );
        let fpsLabel = Label::new(
            "calculating...".to_string(),
            Positions::RelPosition(
                RelPosition { relx: 50, rely: 80 }
            ),
            Some(true), None, None,
            None, None
        );

        let img = Image::new(
            Positions::RelPosition(
                RelPosition { relx: 25, rely: 0 }
            ),
            Sizes::RelSize(
                RelSize { relwidth: 50, relheight: 100 }
            ),
            "D:\\tuicore\\photo_2026-03-29_03-57-03.jpg".to_string(),
            None
        );

        let redid: u32 = app.register_renderer(Box::new(img));
        let lid: u32 = app.register_renderer(Box::new(label));
        let fpslid: u32 = app.register_renderer(Box::new(fpsLabel));
        app.register_event(
            AppEvent {
                evtype: EventType::OnSizeChange,
                exec: Box::new(move |mut renderers, mut buff, mut context| {
                    if let Some(x) = renderers.get_mut(&lid) {
                        let x = x.as_mut();

                        if let Some(label) = x.as_any_mut().downcast_mut::<Label>() {
                            label.change_content(format!("{}x{}", buff.size.width, buff.size.height));
                        }
                    }
                })
            }
        );
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
                                    label.change_content(format!("{} fps (ft {}s)", 1.0/t2.duration_since(*t1).as_secs_f32(), t2.duration_since(*t1).as_secs_f32()));
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