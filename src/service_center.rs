use std::{io, rc::Rc};
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};
use std::fs;

use super::Action;


pub mod items;
pub use items::*;

pub mod vehicles;
pub use vehicles::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct KWh(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wh(pub f32);


// VEHICLES

type RVeh = Rc<dyn Vehicle>;
type RIt = Rc<dyn Item>;
type AreaVec = Vec<(String, TextArea<'static>)>;

#[derive(Debug, Clone, PartialEq)]
pub struct FormatError {
    message: String,
}

impl FormatError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FormatError {}

fn field(areas: &AreaVec, ind: usize) -> &str {
    areas[ind].0.as_str()
}

fn required_text(areas: &AreaVec, ind: usize) -> Result<String, FormatError> {
    let text = areas[ind].1.lines().join("").trim().to_string();
    if text.is_empty() {
        return Err(FormatError::new(format!(
            "поле «{}» не должно быть пустым", field(areas, ind)
        )));
    }
    Ok(text)
}

const MAX_MODEL_LEN: usize = 64;

fn model_text(areas: &AreaVec, ind: usize) -> Result<String, FormatError> {
    let text = required_text(areas, ind)?;
    if text.chars().count() > MAX_MODEL_LEN {
        return Err(FormatError::new(format!(
            "поле «{}»: длина не должна превышать {} символов", field(areas, ind), MAX_MODEL_LEN
        )));
    }
    Ok(text)
}

fn parse_f32(areas: &AreaVec, ind: usize) -> Result<f32, FormatError> {
    let text = required_text(areas, ind)?;
    let value: f32 = text.parse().map_err(|_| FormatError::new(format!(
        "поле «{}»: «{}» не является числом", field(areas, ind), text
    )))?;
    if !value.is_finite() {
        return Err(FormatError::new(format!(
            "поле «{}»: «{}» должно быть конечным числом", field(areas, ind), text
        )));
    }
    if value.is_sign_negative() {
        return Err(FormatError::new(format!(
            "поле «{}»: «{}» не может быть отрицательным", field(areas, ind), text
        )));
    }
    Ok(value)
}

fn parse_u8(areas: &AreaVec, ind: usize) -> Result<u8, FormatError> {
    let text = required_text(areas, ind)?;
    text.parse::<u8>().map_err(|_| FormatError::new(format!(
        "поле «{}»: «{}» не является целым неотрицательным числом", field(areas, ind), text
    )))
}

fn parse_bool(areas: &AreaVec, ind: usize) -> Result<bool, FormatError> {
    let text = required_text(areas, ind)?.to_lowercase();
    match text.as_str() {
        "y" | "yes" | "1" | "true" | "да" => Ok(true),
        "n" | "no" | "0" | "false" | "нет" => Ok(false),
        _ => Err(FormatError::new(format!(
            "поле «{}»: «{}» неверно, ожидалось y/n", field(areas, ind), text
        ))),
    }
}


pub struct ServiceCenter {
    techs: Vec<RVeh>,
    items: Vec<RIt>
}

impl ServiceCenter {
    pub fn new() -> Self {
        Self { techs: Vec::new(), items: Vec::new() }
    }

    pub fn add_transport(&mut self, v: RVeh) {
        self.techs.push(v);
    }

    pub fn add_item(&mut self, i: RIt) {
        self.items.push(i);
    }

    pub fn delete_item(&mut self, ind: usize) {
        if ind < self.items.len() {
            self.items.remove(ind);
        }
    }

    pub fn delete_transport(&mut self, ind: usize) {
        if ind < self.techs.len() {
            self.techs.remove(ind);
        }
    }

    pub fn save(&mut self) -> Result<(), io::Error> {
        let filename = "date.json";
        let json = serde_json::to_string_pretty(&(&self.techs, &self.items)).unwrap();
        fs::write(filename, json)?;

        return Ok(());
    }

    pub fn load(&mut self) -> Result<(), io::Error> {
        self.load_from("date.json")
    }

    pub fn load_example(&mut self) -> Result<(), io::Error> {
        self.load_from("example.json")
    }

    fn load_from(&mut self, filename: &str) -> Result<(), io::Error> {
        let content = fs::read_to_string(filename)?;
        (self.techs, self.items) = serde_json::from_str(&content)?;

        return Ok(());
    }

    pub fn next_item_id(&self) -> u32 {
        self.items.len() as u32
    }

    pub fn next_transport_id(&self) -> u32 {
        self.techs.len() as u32
    }

    pub fn energy_statistic_options(&self, ind: usize) -> Option<(String, Action)> {
        if let Some(t) = self.techs.get(ind) {
            return Some((
                format!("У {} осталось {}/{} kWh", t.name(), t.get_energy().0, t.get_capacity().0),
                Action::None
            ));
        }
        if ind == self.techs.len() {
            let energy: f32 = self.techs.iter().map(|t| t.get_energy().0).sum();
            let capacity: f32 = self.techs.iter().map(|t| t.get_capacity().0).sum();
            return Some((format!("Итого: {}/{} kWh", energy, capacity), Action::None));
        }
        None
    }

    pub fn nublist_options(&self, ind: usize) -> Option<(String, Action)> {
        self.techs.iter()
            .filter(|t| t.for_beginners())
            .nth(ind)
            .map(|t| (
                format!("`{}` с простотой: {}", t.name(), t.simplicity()),
                Action::None
            ))
    }

    pub fn transport_options(&self, ind: usize) -> Option<(String, Action)> {
        self.techs.get(ind).map(|t| (t.to_string(), Action::None))
    }

    pub fn item_options(&self, ind: usize) -> Option<(String, Action)> {
        self.items.get(ind).map(|i| (i.to_string(), Action::None))
    }

}

#[cfg(test)]
mod parse_tests {
    use super::*;

    fn areas(values: &[&str]) -> AreaVec {
        values.iter()
            .map(|v| ("field".to_string(), TextArea::new(vec![v.to_string()])))
            .collect()
    }

    #[test]
    fn parse_f32_ok() {
        assert_eq!(parse_f32(&areas(&["1.5"]), 0).unwrap(), 1.5);
        assert_eq!(parse_f32(&areas(&["0"]), 0).unwrap(), 0.0);
        assert_eq!(parse_f32(&areas(&["  2.25  "]), 0).unwrap(), 2.25);
    }

    #[test]
    fn parse_f32_rejects_negative() {
        assert!(parse_f32(&areas(&["-1"]), 0).is_err());
        assert!(parse_f32(&areas(&["-0.5"]), 0).is_err());
        assert!(parse_f32(&areas(&["-0"]), 0).is_err());
        assert!(parse_f32(&areas(&["-0.0"]), 0).is_err());
    }

    #[test]
    fn parse_f32_rejects_infinite() {
        assert!(parse_f32(&areas(&["inf"]), 0).is_err());
        assert!(parse_f32(&areas(&["-inf"]), 0).is_err());
        assert!(parse_f32(&areas(&["1e999"]), 0).is_err());
    }

    #[test]
    fn parse_f32_rejects_nan() {
        assert!(parse_f32(&areas(&["nan"]), 0).is_err());
    }

    #[test]
    fn parse_f32_rejects_garbage() {
        assert!(parse_f32(&areas(&["abc"]), 0).is_err());
        assert!(parse_f32(&areas(&[""]), 0).is_err());
    }

    #[test]
    fn parse_u8_ok() {
        assert_eq!(parse_u8(&areas(&["0"]), 0).unwrap(), 0);
        assert_eq!(parse_u8(&areas(&["255"]), 0).unwrap(), 255);
    }

    #[test]
    fn parse_u8_rejects_overflow() {
        assert!(parse_u8(&areas(&["256"]), 0).is_err());
        assert!(parse_u8(&areas(&["300"]), 0).is_err());
        assert!(parse_u8(&areas(&["99999999999999999999"]), 0).is_err());
    }

    #[test]
    fn parse_u8_rejects_garbage() {
        assert!(parse_u8(&areas(&["-1"]), 0).is_err());
        assert!(parse_u8(&areas(&["1.5"]), 0).is_err());
        assert!(parse_u8(&areas(&[""]), 0).is_err());
    }
}