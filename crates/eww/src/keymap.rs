use std::{collections::HashMap, time::Duration};

use anyhow::anyhow;
use codespan_reporting::diagnostic::Severity;
use eww_shared_util::Span;
use gdk::{
    keys::{constants::VoidSymbol, Key},
    ModifierType,
};
use gtk::prelude::*;
use std::str::FromStr;
use yuck::{config::window_keymap::WindowKeymap, gen_diagnostic};

use crate::{enum_parse, error_handling_ctx, widgets::run_command};

pub fn assign_to_window(map: &WindowKeymap, window: &gtk::Window) {
    window.add_events(gdk::EventMask::KEY_PRESS_MASK);

    let keymap = parse_from_yuck(map);
    let _ = window.connect_key_press_event(move |_, evt| {
        let c = Combination { mods: evt.state(), key: evt.keyval() };

        if let Some((cmd, inhibit)) = keymap.get(&c) {
            run_command(Duration::from_millis(200), cmd, &[] as &[&str]);
            *inhibit
        } else {
            gtk::Inhibit(false)
        }
    });
}

fn parse_from_yuck(wk: &WindowKeymap) -> HashMap<Combination, (String, gtk::Inhibit)> {
    let mut map = HashMap::with_capacity(wk.binds.len());
    for bind in wk.binds.iter() {
        match bind.combination.parse() {
            Ok(c) => drop(map.insert(c, (bind.cmd.clone(), gtk::Inhibit(bind.inhibit.unwrap_or_default())))),
            Err(e) => print_diagnostic(e, bind.combination_span),
        }
    }
    map
}

fn print_diagnostic(e: anyhow::Error, span: Span) {
    let diag = error_handling_ctx::stringify_diagnostic(gen_diagnostic! {
        kind =  Severity::Error,
        msg = format!("Invalid key combination provided: {e}"),
        label = span => "Found in here",
        note = "The valid format is \"Ctrl + Shift + X\" (case insentitive, whitespace tolerant)"
    });

    match diag {
        Ok(d) => eprintln!("{d}"),
        Err(e) => eprintln!("Failed to generate diagnostic: {e}"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Combination {
    mods: ModifierType,
    key: Key,
}

impl FromStr for Combination {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('+').map(str::trim).rev();

        let key_str = iter.next().ok_or_else(|| anyhow!("Missing key"))?;
        let key = Key::from_name(key_str);

        if key == VoidSymbol {
            return Err(anyhow!("Unknown key name: {key}"));
        }

        let mut mods = ModifierType::empty();
        for m in iter {
            mods = mods | {
                enum_parse! { "modifier", m,
                    "shift" => ModifierType::SHIFT_MASK,
                    "ctrl" => ModifierType::CONTROL_MASK,
                    "alt" => ModifierType::MOD2_MASK,
                    "super" => ModifierType::MOD2_MASK,
                }
            }?;
        }

        Ok(Self { mods, key })
    }
}
