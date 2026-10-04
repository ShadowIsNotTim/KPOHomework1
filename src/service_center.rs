use std::{io, rc::Rc};
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};
use std::fs;


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

    pub fn save(&mut self) -> Result<(), io::Error> {
        let filename = "date.json";
        let json = serde_json::to_string_pretty(&(&self.techs, &self.items)).unwrap();
        fs::write(filename, json)?;

        return Ok(());
    }

    pub fn load(&mut self) -> Result<(), io::Error> {
        let filename = "date.json";
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

    pub fn energy_statistic(&self) -> Vec<String> {
        self.techs.iter()
            .map(|t| format!(
                "У {} осталось {}/{} kWh", t.name(), t.get_energy().0, t.get_capacity().0))
            .collect()
    }

    pub fn nublist(&self) -> Vec<String> {
        self.techs.iter()
            .filter(|t| t.for_beginners())
            .map(|t| format!("`{}` с простотой: {}", t.name(), t.simplicity()))
            .collect()
    }

    pub fn transport_list(&self) -> Vec<String> {
        self.techs.iter()
            .map(|t| t.to_string())
            .collect()
    }

    pub fn item_list(&self) -> Vec<String> {
        self.items.iter()
            .map(|i| i.to_string())
            .collect()
    }

}