use macroquad::prelude::*;

use super::{confirm_pressed, Ctx, Overworld, Scene};
use crate::canvas;
use crate::data::Data;
use crate::game::quest::{next_grant, Grant};
use crate::game::queue::{Action, Outcome, Queue, TICK_MINUTES};
use crate::game::run::Run;
use crate::ui;

enum Phase {
    /// A line from the clerk, then leave. `end_day` says whether leaving ends the day.
    Say {
        line: String,
        end_day: bool,
    },
    Greet(String),
    Queue {
        queue: Queue,
        last: String,
    },
    Issue {
        grant: Grant,
        line: String,
    },
}

pub struct Office {
    run: Run,
    office_id: String,
    return_pos: Vec2,
    phase: Phase,
}

impl Office {
    pub fn new(mut run: Run, office_id: String, return_pos: Vec2, d: &Data) -> Self {
        let office = d.office(&office_id);
        let mut rng = run.rng(&format!("greet:{office_id}"));
        let phase = if !run.clock.numbers_open() {
            run.end_day();
            Phase::Say {
                line: d.s("numbers_closed", &[]),
                end_day: false,
            }
        } else {
            Phase::Greet(rng.pick(&office.greet).clone())
        };
        Self {
            run,
            office_id,
            return_pos,
            phase,
        }
    }

    fn leave(&self, toast: Option<String>) -> Option<Scene> {
        Some(Scene::Overworld(Overworld::at(
            self.run.clone(),
            self.return_pos,
            toast,
        )))
    }

    fn after_greet(&mut self, d: &Data) {
        let office = d.office(&self.office_id);
        let mut rng = self.run.rng(&format!("lines:{}", self.office_id));
        self.phase = match next_grant(&office.grants, &self.run.inv) {
            None => Phase::Say {
                line: rng.pick(&office.nothing).clone(),
                end_day: false,
            },
            Some(g) => {
                let missing = g.missing(&self.run.inv);
                if let Some(doc) = missing.first() {
                    Phase::Say {
                        line: rng.pick(&office.missing).replace("{doc}", d.doc_name(doc)),
                        end_day: false,
                    }
                } else {
                    Phase::Queue {
                        queue: Queue::new(self.run.rng(&format!("queue:{}", self.office_id))),
                        last: String::new(),
                    }
                }
            }
        };
    }

    fn queue_action(&mut self, action: Action, d: &Data) {
        let office = d.office(&self.office_id);
        let Phase::Queue { queue, last } = &mut self.phase else {
            return;
        };
        if !queue.can(action) {
            return;
        }
        if let Some(price) = queue.fixer_price() {
            if action == Action::PayFixer && !self.run.meters.can_afford(price) {
                *last = d.s("cannot_pay", &[("fee", &price.to_string())]);
                return;
            }
        }
        let step = queue.step(action);
        self.run.meters.spend(step.money);
        self.run.meters.shortcut(step.conscience);
        *last = step.message.to_owned();
        let day_over = self.run.advance(TICK_MINUTES);

        match step.outcome {
            Outcome::Served => {
                let grant = next_grant(&office.grants, &self.run.inv)
                    .expect("queue only runs with a grant pending")
                    .clone();
                let mut rng = self.run.rng(&format!("issued:{}", self.office_id));
                if !self.run.meters.spend(grant.fee) {
                    self.phase = Phase::Say {
                        line: d.s("cannot_pay", &[("fee", &grant.fee.to_string())]),
                        end_day: false,
                    };
                    return;
                }
                let mut line = rng.pick(&office.issued).clone();
                if grant.fee > 0 {
                    line.push(' ');
                    line.push_str(&d.s("issued_fee", &[("fee", &grant.fee.to_string())]));
                }
                self.phase = Phase::Issue { grant, line };
            }
            Outcome::Fight => {
                self.phase = Phase::Say {
                    line: step.message.to_owned(),
                    end_day: true,
                };
            }
            Outcome::Waiting if day_over => {
                self.phase = Phase::Say {
                    line: d.s("queue_closed", &[]),
                    end_day: true,
                };
            }
            Outcome::Waiting => {}
        }
    }

    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        let d = &ctx.assets.data;
        match &self.phase {
            Phase::Say { end_day, .. } => {
                if confirm_pressed() {
                    let toast = if *end_day {
                        self.run.end_day();
                        Some(d.s("day_ended", &[]))
                    } else {
                        None
                    };
                    return self.leave(toast);
                }
            }
            Phase::Greet(_) => {
                if confirm_pressed() {
                    self.after_greet(d);
                }
            }
            Phase::Queue { queue, .. } => {
                let keys = [
                    (KeyCode::Key1, Action::Wait),
                    (KeyCode::Key2, Action::Confront),
                    (KeyCode::Key3, Action::Cut),
                    (KeyCode::Key4, Action::PayFixer),
                ];
                let mut chosen = keys
                    .iter()
                    .find(|(k, _)| is_key_pressed(*k))
                    .map(|(_, a)| *a);
                for (i, (_, action)) in keys.iter().enumerate() {
                    if is_mouse_button_pressed(MouseButton::Left)
                        && action_rect(i).contains(ctx.mouse)
                        && queue.can(*action)
                    {
                        chosen = Some(*action);
                    }
                }
                if let Some(a) = chosen {
                    self.queue_action(a, d);
                }
            }
            Phase::Issue { grant, .. } => {
                if confirm_pressed() {
                    let doc = grant.doc.clone();
                    self.run.inv.add(&doc);
                    return self.leave(None);
                }
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let office = d.office(&self.office_id);
        clear_background(ui::WALL);
        draw_rectangle(0.0, 200.0, canvas::WIDTH, 50.0, ui::WOOD);
        draw_rectangle(0.0, 196.0, canvas::WIDTH, 4.0, ui::INK);
        let name_w = ui::text_width(font, &office.name, 16) + 24.0;
        ui::panel((canvas::WIDTH - name_w) / 2.0, 36.0, name_w, 26.0);
        ui::text_centered(font, &office.name, canvas::WIDTH / 2.0, 54.0, 16, ui::INK);

        match &self.phase {
            Phase::Say { line, .. } => {
                ui::dialog(font, &office.short, line);
                ui::hint(font, &d.s("hint_leave", &[]));
            }
            Phase::Greet(line) => {
                ui::dialog(font, &office.short, line);
                ui::hint(font, &d.s("hint_counter", &[]));
            }
            Phase::Issue { line, .. } => {
                draw_texture_ex(
                    &ctx.assets.stamp,
                    canvas::WIDTH / 2.0 - 32.0,
                    100.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(64.0, 64.0)),
                        ..Default::default()
                    },
                );
                ui::dialog(font, &office.short, line);
                ui::hint(font, &d.s("hint_take", &[]));
            }
            Phase::Queue { queue, last } => self.draw_queue(ctx, queue, last),
        }
        ui::hud(font, d, &self.run);
    }

    fn draw_queue(&self, ctx: &Ctx, queue: &Queue, last: &str) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        // People ahead, drawn as a snake toward the counter.
        for i in 0..queue.ahead.min(30) {
            let x = 40.0 + (i % 15) as f32 * 38.0;
            let y = 140.0 - (i / 15) as f32 * 36.0;
            draw_rectangle(x, y, 10.0, 16.0, ui::INK);
            draw_rectangle(x + 2.0, y - 5.0, 6.0, 6.0, Color::new(0.85, 0.65, 0.5, 1.0));
        }
        let me_x = 40.0 + (queue.ahead.min(30) % 15) as f32 * 38.0;
        let me_y = 140.0 - (queue.ahead.min(30) / 15) as f32 * 36.0;
        draw_rectangle(me_x, me_y, 10.0, 16.0, ui::STAMP);
        draw_rectangle(
            me_x + 2.0,
            me_y - 5.0,
            6.0,
            6.0,
            Color::new(0.85, 0.65, 0.5, 1.0),
        );
        ui::text(font, "you", me_x - 6.0, me_y + 30.0, 16, ui::PAPER);

        let ahead = d.s("queue_ahead", &[("n", &queue.ahead.to_string())]);
        ui::text(font, &ahead, 16.0, 190.0, 16, ui::PAPER);

        ui::panel(16.0, 206.0, canvas::WIDTH - 32.0, 36.0);
        ui::text_wrapped(font, last, 24.0, 228.0, canvas::WIDTH - 48.0, 16, ui::INK);

        let labels = [
            d.s("act_wait", &[]),
            d.s("act_confront", &[]),
            d.s("act_cut", &[]),
            match queue.fixer_price() {
                Some(p) => d.s("act_fixer", &[("price", &p.to_string())]),
                None => d.s("act_fixer_none", &[]),
            },
        ];
        let actions = [
            Action::Wait,
            Action::Confront,
            Action::Cut,
            Action::PayFixer,
        ];
        for (i, (label, action)) in labels.iter().zip(actions).enumerate() {
            ui::button(font, label, action_rect(i), ctx.mouse, queue.can(action));
        }
    }
}

fn action_rect(i: usize) -> Rect {
    let col = (i % 2) as f32;
    let row = (i / 2) as f32;
    Rect::new(16.0 + col * 308.0, 252.0 + row * 44.0, 300.0, 36.0)
}
