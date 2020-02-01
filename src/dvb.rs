/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2018, 2019  Russel Winder
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::fmt;
use std::slice::Iter;

use serde_derive::{Deserialize, Serialize};

/// The various options for DVB delivery system the user is using.
#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum DeliverySystem {
    ATSC,
    DVBC_ANNEX_A,
    DVBC_ANNEX_B,
    DVBT,
    DVBT2,
    ISDBT,
}

// Rust doesn't allow iteration of enum variants directly so we
// hack it up using a slice. Unpleasant but it does the job.
static DELIVERY_SYSTEMS: [DeliverySystem;  6] = [
    DeliverySystem::ATSC,
    DeliverySystem::DVBC_ANNEX_A,
    DeliverySystem::DVBC_ANNEX_B,
    DeliverySystem::DVBT,
    DeliverySystem::DVBT2,
    DeliverySystem::ISDBT,
];

impl DeliverySystem {
    /// Iterate over the `DeliverySystem` variants.
    pub fn iterator() -> Iter<'static, DeliverySystem> {
        DELIVERY_SYSTEMS.iter()
    }

    /// Return the position of the `DeliverySystem` in the sequence of all possibilities.
    pub fn get_index(&self) -> u32 {
        // TODO This is hideous code.
        let mut i = 0;
        for d_s in DELIVERY_SYSTEMS.iter() {
            if d_s == self {
                return i
            }
            i += 1;
        };
        panic!("Failure of preferences::DeliverySystem.");
    }
}

impl fmt::Display for DeliverySystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl std::convert::From<&str> for DeliverySystem {
    fn from(s: &str) -> DeliverySystem {
        match s {
            "ATSC" => DeliverySystem::ATSC,
            "DVBC_ANNEX_A" => DeliverySystem::DVBC_ANNEX_A,
            "DVBC_ANNEX_B" => DeliverySystem::DVBC_ANNEX_B,
            "DVBT" => DeliverySystem::DVBT,
            "DVBT2" => DeliverySystem::DVBT2,
            "ISDBT" => DeliverySystem::ISDBT,
            _ => panic!("unknown DeliverySystem variant."),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn construct_from_string() {
        assert_eq!(DeliverySystem::from("DVBT2"), DeliverySystem::DVBT2);
    }

    #[test]
    fn render_to_string() {
        assert_eq!(DeliverySystem::DVBT2.to_string(), String::from("DVBT2"));
    }

    #[test]
    fn index_of_dvbt2() {
        assert_eq!(DeliverySystem::DVBT2.get_index(), 4);
    }

    #[test]
    fn iterator_sequence() {
        let mut i = DeliverySystem::iterator();
        assert_eq!(*i.next().unwrap(), DeliverySystem::ATSC);
        assert_eq!(*i.next().unwrap(), DeliverySystem::DVBC_ANNEX_A);
    }

    #[test]
    fn format_output() {
        assert_eq!(format!("{}", DeliverySystem::DVBT2), "DVBT2");
    }
}
