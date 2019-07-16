/*
Copyright (c) 2018 Pierre Marijon <pierre.marijon@inria.fr>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/* local use */

/* crates use */
use serde::ser::SerializeStruct;
use serde_json;

/* standard use */
use std;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/* begin of type declaration */

#[derive(Debug, PartialEq, Serialize)]
pub enum BadReadType {
    Chimeric,
    NotCovered,
    NotBad,
}

impl Eq for BadReadType {}

impl BadReadType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BadReadType::Chimeric => "Chimeric",
            BadReadType::NotCovered => "Not_covered",
            BadReadType::NotBad => "NotBad",
        }
    }
}

#[derive(Debug)]
pub struct NameLen {
    pub name: String,
    pub len: u64,
}

impl PartialEq for NameLen {
    fn eq(&self, other: &NameLen) -> bool {
        self.name == other.name
    }
}

impl Eq for NameLen {}

impl Hash for NameLen {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum IntervalType {
    Sure,
    Check,
}

#[derive(Debug, Clone)]
pub struct Interval {
    pub begin: u64,
    pub end: u64,
    pub int_type: IntervalType,
}

impl serde::ser::Serialize for Interval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Interval", 2)?;
        state.serialize_field("begin", &self.begin)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl Ord for Interval {
    fn cmp(&self, other: &Interval) -> Ordering {
        let r = self.begin.cmp(&other.begin);

        return match r {
            Ordering::Equal => self.end.cmp(&other.end),
            _ => r,
        };
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Interval) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Interval) -> bool {
        self.begin == other.begin && self.end == other.end
    }
}

impl Eq for Interval {}

pub type BadReadMap = HashMap<String, (BadReadType, u64, Vec<Interval>)>;

pub fn write<W: std::io::Write>(mut output: &mut W, remove_reads: &BadReadMap, json: bool) {
    if json {
        let mut map = serde_json::map::Map::new();
        for (id, (label, len, gaps)) in remove_reads {
            map.insert(
                id.to_string(),
                json!({
                    "type": label.as_str(),
                    "length": len,
                    "gaps": gaps
                }),
            );
        }
        output
            .write(&json!(map).to_string().into_bytes())
            .expect("Error durring write result in json format");
    } else {
        for (id, (label, len, gaps)) in remove_reads {
            write_result(&mut output, &label, &id, &len, &gaps);
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn formating_gap() {
        let mut writer: Vec<u8> = Vec::new();
        let input = vec![
            Interval {
                begin: 0,
                end: 10,
                int_type: IntervalType::Sure,
            },
            Interval {
                begin: 50,
                end: 100,
                int_type: IntervalType::Sure,
            },
            Interval {
                begin: 150,
                end: 200,
                int_type: IntervalType::Sure,
            },
        ];

        for (i, gaps) in input.iter().enumerate() {
            write_gap(gaps, &mut writer, input.len() - i);
        }

        assert_eq!(writer, b"10,0,10;50,50,100;50,150,200");
    }
}
