//! Document inventory and chain resolution.

use serde::Deserialize;

#[derive(Clone, Debug, Default)]
pub struct Inventory(Vec<String>);

impl Inventory {
    pub fn has(&self, doc: &str) -> bool {
        self.0.iter().any(|d| d == doc)
    }

    pub fn add(&mut self, doc: &str) {
        if !self.has(doc) {
            self.0.push(doc.to_owned());
        }
    }
}

/// One document an office can issue and what it asks for in return.
#[derive(Clone, Debug, Deserialize)]
pub struct Grant {
    pub doc: String,
    pub requires: Vec<String>,
    #[serde(default)]
    pub fee: i64,
}

impl Grant {
    pub fn missing<'a>(&'a self, inv: &Inventory) -> Vec<&'a str> {
        self.requires
            .iter()
            .filter(|d| !inv.has(d))
            .map(String::as_str)
            .collect()
    }
}

/// First grant at an office the player does not hold yet, or None when the
/// office has nothing left to give.
pub fn next_grant<'a>(grants: &'a [Grant], inv: &Inventory) -> Option<&'a Grant> {
    grants.iter().find(|g| !inv.has(&g.doc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(doc: &str, req: &[&str]) -> Grant {
        Grant {
            doc: doc.into(),
            requires: req.iter().map(|s| s.to_string()).collect(),
            fee: 0,
        }
    }

    #[test]
    fn chain_resolves_in_order() {
        let polsek = [g("loss_report", &[])];
        let dukcatan = [g("ktpss", &["loss_report", "ksk_copy"])];
        let mut inv = Inventory::default();

        let grant = next_grant(&dukcatan, &inv).unwrap();
        assert_eq!(grant.missing(&inv), vec!["loss_report", "ksk_copy"]);

        inv.add(next_grant(&polsek, &inv).unwrap().doc.as_str());
        assert!(next_grant(&polsek, &inv).is_none());
        assert_eq!(grant.missing(&inv), vec!["ksk_copy"]);

        inv.add("ksk_copy");
        assert!(grant.missing(&inv).is_empty());
        inv.add("ktpss");
        assert!(inv.has("ktpss"));
    }
}
