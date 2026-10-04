use std::rc::Rc;
use ratatui_textarea::TextArea;
use strum::Display;


mod items;
use items::*;

mod vehicles;
use vehicles::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Default)]
pub enum PageOption {
    #[strum(to_string = "главное меню")]
    #[default]
    MainMenu,
    #[strum(to_string = "добавить новый транспорт")]
    AddNewTransport,
    #[strum(to_string = "провести техосмотр")]
    TechReview,
    #[strum(to_string = "статистика потребления энергии")]
    EnergyStatistic,
    #[strum(to_string = "список устройств для новичков")]
    NubList,
    #[strum(to_string = "инвентарь")]
    Inventory,
    #[strum(to_string = "о нас")]
    AboutUs,
}


#[derive(Debug, Clone, Copy)]
pub enum Action {
    None,
    Back,
    Quit,
    Go(PageOption),
}

pub struct KWh(pub f32);
pub struct Wh(pub f32);


// VEHICLES

type RVeh = Rc<dyn Vehicle>;
type RIt = Rc<dyn Item>;
type AreaVec = Vec<(String, TextArea<'static>)>;



pub enum ItemTypes {
    ChargingStation(ChargingStation),
    Helmet(Helmet)
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

    pub fn next_item_id(&self) -> u32 {
        self.items.len() as u32
    }

    pub fn energy_statistic(&self) -> Vec<String> {
        self.techs.iter()
            .map(|t| format!(
                "У {} осталось {}/{} kWh", t.name(), t.get_energy().0, t.get_capacity().0))
            .collect()
    }

    pub fn numlist(&self) -> Vec<String> {
        self.techs.iter()
            .filter(|t| t.for_beginners())
            .map(|t| t.to_string())
            .collect()
    }

}