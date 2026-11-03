use super::{cannot_pay, Outcome};
use crate::data::Data;
use crate::game::nego::{Choice, Nego, Outcome as Result, Who, REVIEW_DAYS};
use crate::game::run::Run;
use crate::scene::{say, Ctx};
use crate::ui;

const CHOICES: [Choice; 3] = [Choice::Pay, Choice::Decline, Choice::Report];

pub struct NegoGame {
    nego: Nego,
    msg: String,
}

impl NegoGame {
    pub fn new(run: &Run, office: &str, d: &Data, who: Who) -> Self {
        let key = if who == Who::Clerk {
            "ob_nego"
        } else {
            "ob_fixer"
        };
        Self {
            nego: Nego::new(run.visit_rng("nego", office), who),
            msg: say(d, office, key, run),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        let d = &ctx.assets.data;
        let Some(i) = ui::choice_input(3, ctx.mouse, |i| self.nego.can(CHOICES[i])) else {
            return Outcome::Running;
        };
        let choice = CHOICES[i];
        if choice == Choice::Pay && !run.meters.can_afford(self.nego.price) {
            self.msg = cannot_pay(d, office, run, self.nego.price);
            return Outcome::Running;
        }
        match self.nego.resolve(choice) {
            Result::Paid => {
                run.meters.spend(self.nego.price);
                run.meters.shortcut(self.nego.conscience());
                run.grease(office);
                Outcome::Granted(say(d, office, "n_paid", run))
            }
            Result::Declined => {
                run.decline(office);
                let key = if self.nego.who == Who::Clerk {
                    "n_declined"
                } else {
                    "n_fixer_declined"
                };
                Outcome::Passed(Some(say(d, office, key, run)))
            }
            Result::Replaced => Outcome::Granted(say(d, office, "n_replaced", run)),
            Result::UnderReview => {
                run.freeze(office, REVIEW_DAYS);
                Outcome::Failed {
                    line: say(d, office, "n_review", run),
                    end_day: false,
                }
            }
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        ui::message(font, &self.msg);
        let mut labels = vec![
            d.s("nego_pay", &[("price", &self.nego.price.to_string())]),
            d.s("nego_decline", &[]),
        ];
        if self.nego.can(Choice::Report) {
            labels.push(d.s("nego_report", &[]));
        }
        ui::choice_buttons(font, &labels, ctx.mouse, |_| true);
        ui::hint(font, &d.s("hint_walkout", &[]));
    }
}
