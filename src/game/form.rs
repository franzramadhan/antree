//! Form Filling, GDD 5.1. Each field offers three spellings of the player's own
//! data. Exactly one matches the format rules printed on this visit's form.

use serde::Deserialize;

use super::rng::Rng;

pub const TIME_LIMIT: f32 = 60.0;
pub const MAX_REJECTIONS: u32 = 3;
pub const REJECT_MINUTES: u32 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Kind {
    /// Free text, subject to the capitals rule.
    Text,
    /// Stored as YYYY-MM-DD, rendered per the date rule.
    Date,
    /// Stored with dashes, rendered with or without.
    Number,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FieldDef {
    pub labels: Vec<String>,
    pub value: String,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateFmt {
    DdMmYyyy,
    YyyyMmDd,
    DdMmYy,
}

const DATE_FMTS: [DateFmt; 3] = [DateFmt::DdMmYyyy, DateFmt::YyyyMmDd, DateFmt::DdMmYy];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    pub capitals: bool,
    pub date: DateFmt,
    pub dashes: bool,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub label: String,
    pub options: [String; 3],
    pub answer: usize,
}

#[derive(Clone, Debug)]
pub struct Form {
    pub rules: Rules,
    pub fields: Vec<Field>,
}

fn render(def: &FieldDef, rules: Rules) -> String {
    match def.kind {
        Kind::Text if rules.capitals => def.value.to_uppercase(),
        Kind::Text => def.value.clone(),
        Kind::Number if rules.dashes => def.value.clone(),
        Kind::Number => def.value.replace('-', ""),
        Kind::Date => {
            let (y, rest) = def.value.split_at(4);
            let (m, d) = (&rest[1..3], &rest[4..6]);
            match rules.date {
                DateFmt::DdMmYyyy => format!("{d}/{m}/{y}"),
                DateFmt::YyyyMmDd => format!("{y}-{m}-{d}"),
                DateFmt::DdMmYy => format!("{d}-{m}-{}", &y[2..]),
            }
        }
    }
}

/// The same data, formatted against the rules.
fn misformat(def: &FieldDef, rules: Rules, rng: &mut Rng) -> String {
    let mut wrong = rules;
    match def.kind {
        Kind::Text => wrong.capitals = !rules.capitals,
        Kind::Number => wrong.dashes = !rules.dashes,
        Kind::Date => {
            let others: Vec<DateFmt> = DATE_FMTS.into_iter().filter(|&f| f != rules.date).collect();
            wrong.date = *rng.pick(&others);
        }
    }
    render(def, wrong)
}

/// Correct format, one character off.
fn typo(s: &str, rng: &mut Rng) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    let spots: Vec<usize> = (0..chars.len())
        .filter(|&i| chars[i].is_ascii_alphanumeric())
        .collect();
    let i = *rng.pick(&spots);
    chars[i] = match chars[i] {
        '9' => '8',
        'z' => 'y',
        'Z' => 'Y',
        c => (c as u8 + 1) as char,
    };
    chars.into_iter().collect()
}

impl Form {
    pub fn new(rng: &mut Rng, defs: &[FieldDef]) -> Self {
        let rules = Rules {
            capitals: rng.chance(50),
            date: *rng.pick(&DATE_FMTS),
            dashes: rng.chance(50),
        };
        let mut picked: Vec<&FieldDef> = defs.iter().collect();
        rng.shuffle(&mut picked);
        let n = rng.range(6.min(defs.len() as u32), 10.min(defs.len() as u32) + 1) as usize;
        let fields = picked[..n]
            .iter()
            .map(|def| {
                let right = render(def, rules);
                let mut options = [right.clone(), misformat(def, rules, rng), typo(&right, rng)];
                rng.shuffle(&mut options);
                Field {
                    label: rng.pick(&def.labels).clone(),
                    answer: options.iter().position(|o| *o == right).unwrap(),
                    options,
                }
            })
            .collect();
        Self { rules, fields }
    }

    /// Number of fields where the pick is not the answer.
    pub fn wrong(&self, picks: &[usize]) -> usize {
        self.fields
            .iter()
            .zip(picks)
            .filter(|(f, &p)| f.answer != p)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defs() -> Vec<FieldDef> {
        let f = |v: &str, kind| FieldDef {
            labels: vec!["Label".into(), "LABEL".into()],
            value: v.into(),
            kind,
        };
        vec![
            f("Warga Biasa", Kind::Text),
            f("Kota Antri Raya", Kind::Text),
            f("1987-03-09", Kind::Date),
            f("2026-11-02", Kind::Date),
            f("9901-0909-0003", Kind::Number),
            f("0999-1234-5678", Kind::Number),
            f("Private employee", Kind::Text),
            f("Jl. Sabar No. 3", Kind::Text),
        ]
    }

    #[test]
    fn renders_dates_in_every_format() {
        let d = &defs()[2];
        let r = |date| Rules {
            capitals: false,
            date,
            dashes: true,
        };
        assert_eq!(render(d, r(DateFmt::DdMmYyyy)), "09/03/1987");
        assert_eq!(render(d, r(DateFmt::YyyyMmDd)), "1987-03-09");
        assert_eq!(render(d, r(DateFmt::DdMmYy)), "09-03-87");
    }

    #[test]
    fn answers_pass_and_every_option_is_distinct() {
        for seed in 0..200 {
            let form = Form::new(&mut Rng::new(seed), &defs());
            assert!((6..=8).contains(&form.fields.len()));
            let answers: Vec<usize> = form.fields.iter().map(|f| f.answer).collect();
            assert_eq!(form.wrong(&answers), 0);
            for f in &form.fields {
                let [a, b, c] = &f.options;
                assert!(a != b && b != c && a != c, "seed {seed}: {:?}", f.options);
            }
        }
    }

    #[test]
    fn one_wrong_pick_is_counted() {
        let form = Form::new(&mut Rng::new(4), &defs());
        let mut picks: Vec<usize> = form.fields.iter().map(|f| f.answer).collect();
        picks[0] = (picks[0] + 1) % 3;
        assert_eq!(form.wrong(&picks), 1);
    }

    #[test]
    fn format_changes_between_visits() {
        let rules: Vec<Rules> = (0..10)
            .map(|v| Form::new(&mut Rng::derive(1, &format!("form:x:{v}")), &defs()).rules)
            .collect();
        assert!(rules.windows(2).any(|w| w[0] != w[1]));
    }
}
